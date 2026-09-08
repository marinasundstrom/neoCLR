use neoclr::{Limits, LoadedProgram, Value, frontend};

#[test]
fn outputs_initialize_values_forward_references_and_project_conditional_library_outputs() {
    let module = frontend::compile(include_str!("../examples/source/outputs.neo")).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = LoadedProgram::new(&loaded)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.output, ["42", "Counter"]);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn source_rejects_missing_initialization_and_incorrect_output_use() {
    for source in [
        "func Main() -> int { var a: int\nreturn a }",
        "func Main() -> () { let a: int }",
        "func Set(out a: int) -> () {}\nfunc Main() -> () {}",
        "func Set(out a: int&) -> () { a = 1 }\nfunc Main() -> () { var a = 0\nSet(&a) }",
        "func Read(a: int&) -> () {}\nfunc Main() -> () { var a = 0\nRead(out a) }",
        "func Read(a: int&) -> () {}\nfunc Main() -> () { var a: int\nRead(&a) }",
        "func Main() -> System.Type { let o = typeof(int&).GetElementType()\nvar some: System.Option.Some<System.Type>\no.TryGet(out some)\nreturn some.Value }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn runtime_checks_callee_output_obligations_even_for_preinitialized_targets() {
    for body in ["return", "let old: int = a\na = old"] {
        let source = format!(
            "func Bad(out a: int&) -> () {{ {body} }}\nfunc Main() -> int {{ var a = 99\nBad(out a)\nreturn a }}"
        );
        let module = frontend::compile(&source).unwrap();
        let fault = LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap_err();
        assert!(
            fault.message.contains("out") && fault.message.contains("assigned"),
            "{fault:?}"
        );
    }
}

#[test]
fn output_methods_dispatch_through_source_interfaces() {
    let source = r#"
interface Writer { func Write(out value: int&) -> () }
record One(): Writer {
    func Write(out value: int&) -> () { value = 1 }
}
func Main() -> int {
    var writer = One()
    let view: Writer& = &writer
    var value: int
    view.Write(out value)
    return value
}
"#;
    let module = frontend::compile(source).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(1)
    );
}
