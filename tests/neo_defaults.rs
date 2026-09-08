use neoclr::{Limits, LoadedProgram, Value, frontend};
fn run(source: &str) -> Result<Value, neoclr::Fault> {
    let module = frontend::compile(source)?;
    Ok(LoadedProgram::new(&module)?.run(Limits::default())?.value)
}
#[test]
fn typed_defaults_do_not_run_constructors_or_field_initializers() {
    assert_eq!(run("class Counter { var Value: int = 42 }\nfunc Main() -> int { let normal = Counter(); let zero = default(Counter); return normal.Value - zero.Value }").unwrap(), Value::Int32(42));
    assert_eq!(
        run("func Main() -> bool { return default(bool) }").unwrap(),
        Value::Boolean(false)
    );
}
#[test]
fn generic_defaults_use_the_runtime_contract() {
    let source =
        "func Zero<T>() -> T { return default(T) }\nfunc Main() -> int { return Zero<int>() }";
    assert_eq!(run(source).unwrap(), Value::Int32(0));
    assert!(
        run(&source.replace(
            "-> int { return Zero<int>()",
            "-> string { return Zero<string>()"
        ))
        .is_err()
    );
}
#[test]
fn missing_defaults_do_not_invent_null_empty_strings_or_empty_arrays() {
    for ty in ["int&", "string", "int[]"] {
        assert!(
            run(&format!(
                "func Main() -> () {{ let value = default({ty}) }}"
            ))
            .is_err()
        );
    }
}
