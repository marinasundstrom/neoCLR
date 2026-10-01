use neoclr::{Limits, Value, assemble, load, run};

#[test]
fn ownerless_namespace_is_retained_and_executes() {
    let mut module = assemble(
        ".module Namespaces\n.entry Main\n.function Main() -> int32\nldc.i4 42\nret\n.end",
    )
    .unwrap();
    module.functions[0].namespace = "System.Math".into();
    let encoded = serde_json::to_string(&module).unwrap();
    let decoded = load(&encoded).unwrap();
    assert_eq!(decoded.functions[0].namespace, "System.Math");
    assert_eq!(
        run(&decoded, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn invalid_namespace_is_rejected_before_execution() {
    for namespace in [
        "System..Math",
        ".Math",
        "System.",
        "System. ",
        "System.\nMath",
    ] {
        let mut module = assemble(
            ".module Namespaces\n.entry Main\n.function Main() -> int32\nldc.i4 42\nret\n.end",
        )
        .unwrap();
        module.functions[0].namespace = namespace.into();
        assert!(
            run(&module, Limits::default())
                .unwrap_err()
                .to_string()
                .contains("function namespace")
        );
    }
}

#[test]
fn type_owned_method_cannot_also_declare_a_function_namespace() {
    let mut module = assemble(
        ".module Namespaces\n.entry Main\n.function Main() -> int32\nldc.i4 42\nret\n.end",
    )
    .unwrap();
    module.functions[0].namespace = "System.Math".into();
    module.functions[0].owner = Some(neoclr::metadata::Type::Int32);
    assert!(
        run(&module, Limits::default())
            .unwrap_err()
            .to_string()
            .contains("function namespace")
    );
}
