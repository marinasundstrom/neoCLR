use neoclr::{Limits, LoadedProgram, Value, frontend};
use std::process::Command;

fn execute(source: &str) -> neoclr::Execution {
    LoadedProgram::new(&frontend::compile(source).unwrap())
        .unwrap()
        .run(Limits::default())
        .unwrap()
}

#[test]
fn source_type_inspection_uses_existing_type_handles_without_boxing() {
    let source = include_str!("../examples/source/typeof.neo");
    let il = frontend::lower_to_il(source).unwrap();
    assert!(il.contains("ldtoken Int32"));
    assert!(il.contains("call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)"));
    assert!(!il.contains("heap.new"));
    assert!(!il.contains("value.pack"));
    let result = execute(source);
    assert_eq!(result.value, Value::Void);
    assert_eq!(
        result.output,
        [
            "System.Int32",
            "Counter",
            "Counter&",
            "System.Result",
            "2",
            "System.Int32",
            "Same type"
        ]
    );
    assert_eq!(result.heap.statistics().allocated_objects, 0);
}

#[test]
fn aliases_closed_generic_arguments_and_address_modes_preserve_identity() {
    for expression in [
        "typeof(int).Equals(typeof(System.Int32))",
        "typeof(Result).Equals(typeof(System.Result))",
        "typeof(()).Equals(typeof(unit))",
        "typeof(Option<Result<int,System.Int32ParseError>>).GetGenericArgument(0).Equals(typeof(System.Result<System.Int32,System.Int32ParseError>))",
        "!typeof(int).Equals(typeof(int&))",
        "typeof(int&).GenericArgumentCount == 0",
    ] {
        assert_eq!(
            execute(&format!("func Main() -> bool {{ return {expression} }}")).value,
            Value::Boolean(true),
            "{expression}"
        );
    }
    assert_eq!(
        execute("func Main() -> string { return typeof(\nint\n).Name }").value,
        Value::String("System.Int32".into())
    );
    let result =
        execute("record Counter(Age: int)\nfunc Main() -> System.Type { return typeof(Counter) }");
    let Value::Object { fields, .. } = result.value else {
        panic!("expected Type value")
    };
    assert_eq!(fields.len(), 1);
    let Value::RuntimeTypeHandle(descriptor) = &fields[0] else {
        panic!("expected type handle")
    };
    assert_eq!(descriptor.name, "Counter");
}

#[test]
fn typeof_takes_a_type_operand_and_respects_visibility_and_shape() {
    for source in [
        "func Main() -> System.Type { return typeof(Unknown) }",
        "func Main() -> System.Type { return typeof(Result<int>) }",
        "func Main() -> System.Type { return typeof(Option<int&>) }",
        "func Main() -> System.Type { return typeof(int&&) }",
        "func Main() -> System.Type { var value = 42; return typeof(value) }",
        "func Make() -> int { return 42 }\nfunc Main() -> System.Type { return typeof(Make()) }",
        "func Main() -> () { let typeof = 1 }",
        "func typeof() -> () {}\nfunc Main() -> () {}",
        "func Main() -> () { let t = typeof(int); t.Name = \"changed\" }",
        "func Main() -> System.RuntimeTypeHandle { return typeof(int).Handle }",
        "func Main() -> string& { var t = typeof(int); return &t.Name }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
    let module = frontend::compile(
        "func Main() -> System.Type { return typeof(int).GetGenericArgument(0) }",
    )
    .unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn property_reads_use_metadata_getters_for_closed_library_values() {
    assert_eq!(execute("func Main() -> int { let parsed = Int32.Parse(\"42\"); if parsed.IsOk { return parsed.GetOkCase().Value }; return 0 }").value, Value::Int32(42));
    assert_eq!(
        execute("func Main() -> string { var t = typeof(int); let r = &t; return r.Name }").value,
        Value::String("System.Int32".into())
    );
}

#[test]
fn documented_cli_example_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["run", "examples/source/typeof.neo"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Same type\n=> Void\n"));
}
