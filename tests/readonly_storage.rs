use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

#[test]
fn source_storage_return_generic_and_reflection_roundtrip() {
    let module =
        frontend::compile(include_str!("../examples/source/readonly-storage.neo")).unwrap();
    let serialized = serde_json::to_string(&module).unwrap();
    assert!(serialized.contains("ReadOnlyByRef"));
    let restored = serde_json::from_str(&serialized).unwrap();
    let program = LoadedProgram::new(&restored).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn reference_contract_probe_now_fails_verification_and_faults_at_return() {
    let module = assemble(include_str!(
        "../docs/experiments/reference-contracts/return-gap.neoil"
    ))
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().unwrap_err().message.contains("readonly"));
    let fault = program.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("readonly"), "{}", fault.message);
    assert_eq!(fault.function.as_deref(), Some("Observe"));
}

#[test]
fn source_rejects_readonly_to_writable_local_return_call_and_container() {
    for main in [
        "let read: readonly Counter& = x; let write: Counter& = read; return 0",
        "return Escape(x).Age",
        "let read: readonly Counter& = x; Write(read); return 0",
        "var a = System.Collections.ArrayList<readonly Counter&>.Allocate(1); a.Add(x); let b: System.Collections.ArrayList<Counter&> = a; return 0",
    ] {
        let source = format!(
            "record Counter(Age: int)
func Write(x: Counter&) -> () {{ x.Age = 0 }}
func Escape(readonly x: Counter&) -> Counter& {{ return x }}
func Main() -> int {{ let x = new Counter(1); {main} }}"
        );
        // Check the invalid main independently of the intentionally invalid Escape.
        let source = if main.contains("Escape") {
            source
        } else {
            source.replace(
                "func Escape(readonly x: Counter&) -> Counter& { return x }",
                "",
            )
        };
        assert!(frontend::compile(&source).is_err(), "accepted {source}");
    }
}

#[test]
fn unchecked_storage_rejects_readonly_before_later_write() {
    let module = assemble(
        r#"
.module App
.entry Main
.function Read(readonly Int32& value) -> readonly Int32&
 ldarg value
 ret
.end
.function Main() -> Int32
 .local Int32& writable
 ldc.i4 1
 heap.new
 call Read(Int32&)
 stloc writable
 ldc.i4 0
 ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    let fault = program.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("readonly"));
    assert_eq!(fault.instruction, Some(3));
}

#[test]
fn readonly_array_elements_preserve_access_and_heap_roots() {
    let module = assemble(
        r#"
.module App
.entry Main
.function Main() -> Int32
 .local (readonly Int32&[])& values
 ldc.i4 1
 array.alloc readonly Int32&
 stloc values
 ldloc values
 ldc.i4 0
 ldc.i4 42
 heap.new
 stelem readonly Int32&
 ldloc values
 ldc.i4 0
 ldelem readonly Int32&
 ldobj Int32
 ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn readonly_local_can_be_replaced_and_frame_escape_still_faults() {
    let module = assemble(
        r#"
.module App
.entry Main
.function Main() -> Int32
 .local readonly Int32& view
 ldc.i4 1
 heap.new
 stloc view
 ldc.i4 42
 heap.new
 stloc view
 ldloc view
 ldobj Int32
 ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );

    let module = assemble(
        r#"
.module App
.entry Main
.function Main() -> readonly Int32&
 .local Int32 owned
 .local readonly Int32& view
 ldc.i4 1
 stloc owned
 ldloca owned
 stloc view
 ldloc view
 ret
.end
"#,
    )
    .unwrap();
    let fault = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("current frame"), "{}", fault.message);
}

#[test]
fn permission_joins_narrow_without_allowing_writable_returns() {
    let source = r#"
.module App
.entry Main
.function Choose(Boolean first, Int32& write, readonly Int32& read) -> readonly Int32&
 ldarg first
 brfalse Read
 ldarg write
 br End
Read:
 ldarg read
End:
 ret
.end
.function Main() -> Int32
 .local Int32& value
 ldc.i4 42
 heap.new
 stloc value
 ldc.bool true
 ldloc value
 ldloc value
 call Choose(Boolean,Int32&,Int32&)
 ldobj Int32
 ret
.end
"#;
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let module = assemble(&source.replace("-> readonly Int32&", "-> Int32&")).unwrap();
    assert!(LoadedProgram::new(&module).unwrap().verify().is_err());
}

#[test]
fn readonly_metadata_recurses_without_allowing_native_or_nested_managed_addresses() {
    use neoclr::assembler::parse_type;
    use neoclr::metadata::Type;
    let element = Type::ReadOnlyByRef(Box::new(Type::Int32));
    assert_eq!(
        parse_type("readonly Int32&[]").unwrap(),
        Type::Array(Box::new(element.clone()))
    );
    assert_eq!(
        parse_type("(readonly Int32&[])&").unwrap(),
        Type::ByRef(Box::new(Type::Array(Box::new(element))))
    );
    for ty in ["readonly Int32", "(readonly Int32&)&", "(readonly Int32&)*"] {
        let source = format!(
            ".module App
.function Main() -> Void
.local {ty} value
ldvoid
ret
.end"
        );
        assert!(
            assemble(&source)
                .and_then(|m| LoadedProgram::new(&m))
                .is_err(),
            "accepted {ty}"
        );
    }
}
