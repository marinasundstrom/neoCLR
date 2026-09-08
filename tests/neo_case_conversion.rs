use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

#[test]
fn case_values_convert_in_bindings_returns_arguments_fields_and_collections() {
    let source = r#"
record Holder(Result: Result<int, string>)
func Success() -> Result<int, string> { return System.Result.Ok<int>(42) }
func Read(result: Result<int, string>) -> int {
    let Ok(value) = result else { return -1 }
    return value
}
func Main() -> int {
    let ok = System.Result.Ok<int>(42)
    let result: Result<int, string> = ok
    let holder = Holder(ok)
    var results = System.Collections.ArrayList<Result<int, string>>.Allocate(1)
    results.Add(ok)
    results[0] = System.Result.Error<string>("no")
    if let Error(_) = results[0] {} else { return -2 }
    let array = new Result<int, string>[1] { ok }
    array[0] = System.Result.Ok<int>(42)
    if Read(Success()) != 42 { return -3 }
    if Read(holder.Result) != 42 { return -4 }
    if Read(array[0]) != 42 { return -5 }
    if Read(ok) != 42 { return -6 }
    return Read(result)
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn optional_and_nested_carriers_require_each_declared_case_layer() {
    let source = r#"
func Main() -> int {
    let inner: Result<int, string> = System.Result.Ok<int>(42)
    let outer: Option<Result<int, string>> = System.Option.Some<Result<int, string>>(inner)
    let missing: Option<int> = System.Option.None()
    if let None = missing {} else { return -1 }
    let Some(result) = outer else { return -2 }
    let Ok(value) = result else { return -3 }
    return value
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn generic_carriers_preserve_reference_payloads_and_evaluate_cases_once() {
    let source = r#"
record Counter(Value: int)
func Wrap<T, E>(value: System.Result.Ok<T>) -> Result<T, E> { return value }
func Next(count: int&) -> System.Result.Ok<int> { count = count + 1; return System.Result.Ok<int>(42) }
func Main() -> int {
    let counter = new Counter(40)
    let ok = System.Result.Ok<Counter&>(counter)
    let wrapped = Wrap<Counter&, string>(ok)
    let Ok(alias) = wrapped else { return -1 }
    if !ReferenceEquals(alias, counter) { return -2 }
    alias.Value = 42
    var count = 0
    let result: Result<int, string> = Next(&count)
    if count != 1 { return -3 }
    let Ok(value) = result else { return -4 }
    return value
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn ordinary_constructors_wrong_generic_arguments_and_missing_case_layers_do_not_convert() {
    for source in [
        "func Main() -> () { let r: Result<int, string> = 42 }",
        "func Main() -> () { let r: Result<Void, string> = System.Result.Ok<int>(42) }",
        "func Main() -> () { let r: Result<int, string> = System.Result.Error<int>(42) }",
        "func Main() -> () { let r: Option<int> = System.Result.Ok<int>(42) }",
        "func Main() -> () { let r: System.Option.Some<int> = 42 }",
        "func Main() -> () { let r: Option<Result<int, string>> = System.Result.Ok<int>(42) }",
        "func Main() -> () { let r: Result<int, string>& = System.Result.Ok<int>(42) }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn implicit_carriers_do_not_allow_frame_backed_payloads_to_escape() {
    let module = frontend::compile("func Main() -> Result<int&, string> { var local = 42; return System.Result.Ok<int&>(&local) }").unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn readonly_case_payload_survives_collection_without_implicit_generic_widening() {
    let source = r#"
record Counter(Value: int)
func Make() -> Result<readonly Counter&, string> {
    return System.Result.Ok<readonly Counter&>(new Counter(42))
}
func Discard() -> () { let temporary = new Counter(0) }
func Main() -> int {
    let result = Make()
    Discard()
    Discard()
    Discard()
    let Ok(view) = result else { return -1 }
    return view.Value
}
"#;
    let module = frontend::compile(source).unwrap();
    let result = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits {
            heap_objects: 2,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.reclaimed_objects() >= 2);
    assert!(frontend::compile("record Counter(Value: int)\nfunc Main() -> () { let ok = System.Result.Ok<Counter&>(new Counter(42)); let result: Result<readonly Counter&, string> = ok }").is_err());
}
