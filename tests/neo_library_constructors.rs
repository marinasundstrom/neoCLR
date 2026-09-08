use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&loaded)
        .unwrap()
        .run(Limits::default())
        .unwrap()
}

#[test]
fn independent_cases_construct_carriers_through_metadata_overloads() {
    let result = run(include_str!("../examples/source/case-constructors.neo"));
    assert_eq!(result.output, ["42", "42"]);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn constructor_context_preserves_reference_readonly_delegate_and_void_payloads() {
    let source = r#"
record Counter(Value: int)
func Nothing() -> () {}
func Add(value: int) -> int { return value + 1 }
func Main() -> int {
    let counter = new Counter(40)
    let ok = System.Result.Ok<Counter&>(counter)
    let result = Result<Counter&, string>(ok)
    let Ok(alias) = result else { return -1 }
    if !ReferenceEquals(alias, counter) { return -2 }
    alias.Value = 41
    let view = System.Option.Some<readonly Counter&>(counter)
    let callback = System.Option.Some<System.Func<int, int>>(Add)
    let empty = System.Result.Ok<Void>(Nothing())
    let voidResult = Result<Void, string>(empty)
    if let Ok(_) = voidResult { return callback.Value(view.Value.Value) }
    return -3
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn constructor_arguments_execute_once_and_errors_are_distinct_variants() {
    let source = r#"
func Next(counter: int&) -> int { counter = counter + 1; return 42 }
func Main() -> int {
    var calls = 0
    let some = Option<int>(System.Option.Some<int>(Next(&calls)))
    let error = Result<int, string>(System.Result.Error<string>("no"))
    let Some(value) = some else { return -1 }
    if calls != 1 { return -2 }
    if let Error(message) = error { if message.Equals("no") { return value } }
    return -3
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn invalid_constructors_and_unclosed_generic_cases_are_rejected() {
    for expression in [
        "System.Result.Ok<int>(true)",
        "System.Result.Ok<int>()",
        "System.Result.Ok<int, string>(42)",
        "System.Result.Ok()",
        "Result<int, string>(42)",
        "System.Collections.ArrayListState<int>()",
        "System.Option.Some<int>(42, 43)",
    ] {
        assert!(
            frontend::compile(&format!("func Main() -> () {{ let value = {expression} }}"))
                .is_err(),
            "{expression}"
        );
    }
}

#[test]
fn case_constructors_do_not_hide_frame_references() {
    let source = "func Main() -> System.Option.Some<int&> { var value = 42; return System.Option.Some<int&>(&value) }";
    let module = frontend::compile(source).unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn enclosing_generic_parameters_close_case_constructors_without_losing_reference_mode() {
    let source = r#"
record Counter(Value: int)
func Wrap<T>(value: T) -> System.Option.Some<T> {
    return System.Option.Some<T>(value)
}
func Main() -> int {
    let counter = new Counter(40)
    let wrapped = Wrap(counter)
    if !ReferenceEquals(wrapped.Value, counter) { return -1 }
    wrapped.Value.Value = 42
    return counter.Value
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn inferred_cases_preserve_argument_types_before_carrier_conversion() {
    let source = r#"
record Counter(Value: int)
func Wrap<T>(value: T) -> System.Option.Some<T> { return System.Option.Some(value) }
func Nothing() -> () {}
func Main() -> int {
    let counter = new Counter(40)
    let ok = System.Result.Ok(counter)
    let result: Result<Counter&, string> = ok
    let Ok(alias) = result else { return -1 }
    if !ReferenceEquals(alias, counter) { return -2 }
    let wrapped = Wrap(alias)
    wrapped.Value.Value = 42
    let view: readonly Counter& = counter
    let some = System.Option.Some(view)
    let readonlyOption: Option<readonly Counter&> = some
    let empty: Result<Void, string> = System.Result.Ok(Nothing())
    if let Ok(_) = empty { return wrapped.Value.Value }
    return -3
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn constructor_inference_is_bounded_and_evaluates_arguments_once() {
    let mut expression = "Next(&calls)".to_owned();
    for _ in 0..24 {
        expression = format!("System.Option.Some({expression})");
    }
    let source = format!(
        "func Next(calls: int&) -> int {{ calls = calls + 1; return 42 }} func Main() -> int {{ var calls = 0; let value = {expression}; return calls }}"
    );
    assert_eq!(run(&source).value, Value::Int32(1));
}

#[test]
fn carrier_target_does_not_supply_or_override_constructor_evidence() {
    for statement in [
        "let result: Result<Void, string> = System.Result.Ok(42)",
        "let result: Result<int, string> = Result(System.Result.Ok(42))",
        "let value = System.Result.Ok<int>(true)",
        "let value = System.Option.Some()",
        "let value = Ok(42)",
    ] {
        assert!(
            frontend::compile(&format!("func Main() -> () {{ {statement} }}")).is_err(),
            "{statement}"
        );
    }
}
