use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

#[test]
fn factories_support_explicit_types_nested_results_void_and_reference_identity() {
    let result = run(r#"
record Item(Value: int)
func Main() -> int {
    let explicit = Result<int,string>.Ok(42)
    let nested = Result<Result<int,string>,string>.Ok(Result<int,string>.Error("inner"))
    if !(nested match { Ok(let inner) => inner.IsErr, Error(_) => false }) { return -1 }
    let done: Result<void,string> = Result<void,string>.Ok(default(void))
    if !done.IsOk { return -2 }
    let item = new Item(40)
    let reference: Result<Item&,string> = Result<Item&,string>.Ok(item)
    reference match { Ok(let shared) => { shared.Value = 42 }, Error(_) => { return -3 } }
    if item.Value != 42 { return -4 }
    let readOnly: readonly Item& = item
    let readOnlyResult = Result<readonly Item&,string>.Ok(readOnly)
    if (readOnlyResult match { Ok(let view) => view.Value, Error(_) => 0 }) != 42 { return -6 }
    let callback = Result<System.Func<int>,string>.Ok(() => 42)
    if (callback match { Ok(let call) => call(), Error(_) => 0 }) != 42 { return -7 }
    return explicit match { Ok(let n) => n, Error(_) => -5 }
}
"#);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn factories_reject_unqualified_owners_bad_payloads_and_wrong_arity() {
    for source in [
        "func Main() -> int { let x = Result.Ok(1); return 0 }",
        "func Main() -> int { return Result.Ok(1) }",
        "func Main() -> Result<int,string> { return Result<int,string>.Ok(true) }",
        "func Main() -> Result<int,string> { return Result<int,string>.Error(1) }",
        "func Main() -> Result<int,string> { return Result<int,string>.Ok() }",
        "func Main() -> Result<int,string> { return Result<int,string>.Ok(1,2) }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
}

#[test]
fn result_factories_do_not_weaken_frame_escape_checks() {
    let module = frontend::compile(
        r#"
func Main() -> Result<int&,string> {
    var local = 42
    return Result<int&,string>.Ok(&local)
}
"#,
    )
    .unwrap();
    let error = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(error.message.contains("frame-backed"), "{error}");
}

#[test]
fn local_named_result_keeps_ordinary_method_resolution() {
    let result = run(r#"
record Factory(Value: int) { func Ok(n: int) -> int { return this.Value + n } }
func Main() -> int { var Result = Factory(40); return Result.Ok(2) }
"#);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn generic_factory_calls_and_payload_evaluation_are_preserved() {
    let result = run(r#"
func Wrap<T>(value: T) -> Result<T,string> { return Result<T,string>.Ok(value) }
func Next(calls: int&) -> int { calls = calls + 1; return 42 }
func Main() -> int {
    var calls = 0
    let value: Result<int,string> = Result<int,string>.Ok(Next(&calls))
    if calls != 1 { return -1 }
    return Wrap<int>(value match { Ok(let n) => n, Error(_) => 0 }) match {
        Ok(let n) => n, Error(_) => -2
    }
}
"#);
    assert_eq!(result.value, Value::Int32(42));
}
