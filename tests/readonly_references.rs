use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

fn raw(body: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = assemble(&format!(".module App\n.entry Main\n{body}"))?;
    LoadedProgram::new(&module)?.run(Limits::default())
}

#[test]
fn neo_readonly_parameters_cover_frame_heap_arrays_and_artifacts() {
    let module = frontend::compile(include_str!("../examples/source/readonly.neo")).unwrap();
    let json = serde_json::to_string(&module).unwrap();
    assert!(json.contains("readonly_parameters"));
    let restored = serde_json::from_str(&json).unwrap();
    let program = LoadedProgram::new(&restored).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["20", "22", "23"]);
}

#[test]
fn unchecked_il_cannot_write_or_forward_restricted_aliases() {
    for body in [
        "ldarg value\nldc.i4 9\nstobj Int32",
        "ldarg value\ninitobj Int32",
        "ldarg value\ncall Write(Int32&)\npop",
        "ldarg value\nstloc alias\nldloc alias\nldc.i4 9\nstobj Int32",
    ] {
        let source = format!(
            r#"
.function Write(Int32& value) -> Void
 ldarg value
 ldc.i4 9
 stobj Int32
 ldvoid
 ret
.end
.function Read(readonly Int32& value) -> Void
 .local Int32& alias
 {body}
 ldvoid
 ret
.end
.function Main() -> Void
 .local Int32 value
 ldc.i4 1
 stloc value
 ldloca value
 call Read(Int32&)
 ret
.end
"#
        );
        let error = raw(&source).unwrap_err();
        assert!(error.message.contains("readonly"), "{}", error.message);
    }
}

#[test]
fn readonly_propagates_into_owned_fields_and_elements_without_deep_freezing() {
    for source in [
        "record Counter(Age: int)\nfunc Bad(readonly x: Counter&) -> () { x.Age = 9 }\nfunc Main() -> () { let x = new Counter(1); Bad(x) }",
        "func Bad(readonly x: int[]&) -> () { x[0] = 9 }\nfunc Main() -> () { let x = new int[1]; Bad(x) }",
        "record Counter(Age: int) { func Change() -> () { this.Age = 9 } }\nfunc Bad(readonly x: Counter&) -> () { x.Change() }\nfunc Main() -> () { let x = new Counter(1); Bad(x) }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
    let source = "record Counter(Age: int)\nrecord Holder(Value: Counter&)\nfunc Change(readonly h: Holder&) -> () { h.Value.Age = 42 }\nfunc Main() -> int { let x = new Counter(1); var h = Holder(x); Change(&h); return x.Age }";
    let module = frontend::compile(source).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn metadata_rejects_invalid_readonly_contracts() {
    for signature in ["readonly Int32 x", "readonly Void* x"] {
        let source = format!(
            ".function Bad({signature}) -> Void\nldvoid\nret\n.end\n.function Main() -> Void\nldvoid\nret\n.end"
        );
        assert!(raw(&source).is_err());
    }
    let mut module = assemble(".module App\n.entry Main\n.function Read(readonly Int32& x) -> Void\nldvoid\nret\n.end\n.function Main() -> Void\nldvoid\nret\n.end").unwrap();
    let index = module
        .functions
        .iter()
        .position(|f| f.name == "Read")
        .unwrap();
    module.functions[index].out_parameters = vec![0];
    assert!(LoadedProgram::new(&module).is_err());
}

#[test]
fn returned_heap_capabilities_survive_storage_and_cannot_be_upgraded() {
    let source = r#"
.function View(readonly Int32& value) -> Int32&
 ldarg value
 ret
.end
.function Main() -> Void
 .local Int32& alias
 ldc.i4 1
 heap.new
 call View(Int32&)
 stloc alias
 ldloc alias
 ldc.i4 9
 stobj Int32
 ldvoid
 ret
.end
"#;
    assert!(raw(source).unwrap_err().message.contains("readonly"));
}

#[test]
fn interface_parameter_contracts_and_reflection_preserve_readonly_metadata() {
    let source = r#"
interface Reader { func Read(readonly value: int&) -> int }
record ReaderImpl() : Reader {
    func Read(readonly value: int&) -> int { return value }
}
func Main() -> int {
    var reader = ReaderImpl()
    let view: Reader& = &reader
    var value = 42
    let methods = typeof(ReaderImpl).GetMethods()
    let parameters = methods[0].GetParameters()
    if !parameters[0].IsReadOnly { return -1 }
    return view.Read(&value)
}
"#;
    let module = frontend::compile(source).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    assert!(
        frontend::compile(&source.replace(
            "func Read(readonly value: int&) -> int { return value }",
            "func Read(value: int&) -> int { return value }"
        ))
        .is_err()
    );
}

#[test]
fn readonly_view_observes_mutation_through_a_distinct_writable_alias() {
    let source = "func Observe(readonly view: int&, writable: int&) -> int { writable = 42; return view }\nfunc Main() -> int { var x = 1; return Observe(&x, &x) }";
    let module = frontend::compile(source).unwrap();
    let result = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
}
