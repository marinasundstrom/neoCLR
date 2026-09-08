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
fn generic_records_preserve_values_references_and_closed_field_types() {
    let source = r#"
record Box<T>(Value: T)
record Pair<T,U>(First: T, Second: U)
record Counter(Value: int)
func Wrap<T>(value: T) -> Box<T> { return Box<T>(value) }
func Main() -> int {
    var value = Box<int>(40)
    var copy = value
    copy.Value = 42
    if value.Value != 40 { return -1 }
    let counter = new Counter(40)
    let reference = Wrap(counter)
    reference.Value.Value = 42
    if !ReferenceEquals(reference.Value, counter) { return -2 }
    let heap = new Box<int>(42)
    let field: int& = &heap.Value
    if field != 42 { return -3 }
    let pair = Pair<Box<int>, Box<Counter&>>(copy, reference)
    if pair.First.Value != pair.Second.Value.Value { return -4 }
    let readonlyCounter: readonly Counter& = counter
    let view = Box<readonly Counter&>(readonlyCounter)
    let option: Option<Box<int>> = System.Option.Some(pair.First)
    let Some(payload) = option else { return -5 }
    return payload.Value
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
    let module = frontend::compile(source).unwrap();
    let box_type = module.types.iter().find(|t| t.name == "Box").unwrap();
    assert_eq!(box_type.generic_parameters.len(), 1);
    assert_eq!(
        box_type.fields[0].ty,
        neoclr::metadata::Type::TypeParameter(0)
    );
}

#[test]
fn generic_record_type_parameters_shadow_imported_cases() {
    assert_eq!(
        run(r#"
import System.Option.*
record Box<Some>(Value: Some)
func Main() -> int { return Box<int>(42).Value }
"#)
        .value,
        Value::Int32(42)
    );
}

#[test]
fn invalid_or_unimplemented_generic_record_forms_are_rejected() {
    for source in [
        "record Box<T>(Value: T)\nfunc Main() -> () { let x = Box(42) }",
        "record Box<T>(Value: T)\nfunc Main() -> () { let x = Box<int,string>(42) }",
        "record Box<T>(Value: T)\nfunc Main() -> () { let x = Box<int>(true) }",
        "record Box<T,T>(Value: T)\nfunc Main() -> () {}",
        "record Box<int>(Value: int)\nfunc Main() -> () {}",
        "record Box<T>(Value: T) { func Get() -> T { return this.Value } }\nfunc Main() -> () {}",
        "class Box<T> { var Value: T }\nfunc Main() -> () {}",
        "record Box<T>(Value: T)\nfunc Main() -> () { var x = default(Box<int&>) }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn generic_record_payloads_do_not_extend_frame_lifetimes() {
    let module = frontend::compile("record Box<T>(Value: T)\nfunc Main() -> Box<int&> { var value = 42; return Box<int&>(&value) }").unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn generic_record_example_runs() {
    let result = run(include_str!("../examples/source/generic-records.neo"));
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["40", "42"]);
}

#[test]
fn arrays_void_and_reference_intent_use_existing_runtime_contracts() {
    assert_eq!(
        run(r#"
record Box<T>(Value: T)
record Buffer<T>(Items: T[]&)
func Nothing() -> () {}
func Main() -> int {
    let empty = Box<Void>(Nothing())
    let data = new int[2]
    let buffer = Buffer<int>(data)
    buffer.Items[1] = 42
    if !typeof(Box<Void>).Equals(typeof(Box<Void>)) { return -1 }
    return data[1]
}
"#)
        .value,
        Value::Int32(42)
    );
    for body in [
        "let box = Box<Counter&>(Counter(42))",
        "let counter = new Counter(42); let box = Box<readonly Counter&>(counter); box.Value.Value = 1",
        "let box: Box<string> = Box<int>(42)",
    ] {
        assert!(frontend::compile(&format!("record Box<T>(Value: T)\nrecord Counter(Value: int)\nfunc Main() -> () {{ {body} }}")).is_err(), "{body}");
    }
}
