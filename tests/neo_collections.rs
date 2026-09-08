use neoclr::{Limits, LoadedProgram, Value, frontend};

#[test]
fn complete_collection_program_preserves_references_and_descriptor_copy_contract() {
    let module = frontend::compile(include_str!("../examples/source/collections.neo")).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.output, ["42", "1", "2", "2"]);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn generic_owner_parsing_preserves_comparisons_and_rejects_bad_conversions() {
    assert!(
        frontend::compile("func Main() -> bool { let a = 1\nlet b = 2\nreturn a < b }").is_ok()
    );
    for source in [
        "func Main() -> () { let a = System.Collections.ArrayList<int,string>.Allocate(0) }",
        "func Main() -> () { var a = System.Collections.ArrayList<int>.Allocate(0)\nlet b: System.Collections.List<string>& = &a }",
        "func Main() -> () { var a = System.Collections.ArrayList<int>.Allocate(0)\nlet b: System.Collections.List<int>& = a }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn indexers_dispatch_getters_and_setters_with_reference_element_contracts() {
    let source = r#"
record Counter(Age: int)
func Main() -> int {
    var list = System.Collections.ArrayList<Counter&>.Allocate(0)
    let view: System.Collections.List<Counter&>& = &list
    let original = new Counter(10)
    list.Add(original)
    let alias = view[0]
    view[0].Age = 12
    let replacement = new Counter(30)
    view[0] = replacement
    let age = &view[0].Age
    age = 32
    if !ReferenceEquals(list[0], replacement) { return -1 }
    return alias.Age + list[0].Age
}
"#;
    let module = frontend::compile(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(44)
    );
}

#[test]
fn value_indexer_setters_evaluate_arguments_once_and_reject_invalid_access() {
    let source = r#"
func Next(value: int&) -> int { value = value + 1; return value }
func Receiver(list: System.Collections.ArrayList<int>&, calls: int&) -> System.Collections.ArrayList<int>& {
    calls = calls + 1
    return list
}
func Main() -> int {
    var list = System.Collections.ArrayList<int>.Allocate(0)
    list.Add(0)
    var calls = -2
    Receiver(&list, &calls)[Next(&calls)] = Next(&calls)
    return list[0] + calls
}
"#;
    let module = frontend::compile(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(2)
    );
    for source in [
        "func Main() -> int { var list = System.Collections.ArrayList<int>.Allocate(0); list.Add(1); list[0] = true; return 0 }",
        "func Main() -> int { var list = System.Collections.ArrayList<int>.Allocate(0); list.Add(1); let x = &list[0]; return 0 }",
        "func Main() -> int { let list = System.Collections.ArrayList<int>.Allocate(0); list[0] = 1; return 0 }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
}
