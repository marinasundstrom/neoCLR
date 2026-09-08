use neoclr::{Limits, LoadedProgram, Value, frontend};

fn program(source: &str) -> LoadedProgram {
    let module = frontend::compile(source).unwrap();
    LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap()
}

#[test]
fn sample_searches_with_custom_readonly_equality() {
    let result = program(include_str!("../examples/source/predicate-search.neo"))
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["1", "Grace"]);
}

#[test]
fn zero_match_is_distinct_from_absence_and_search_stops_at_first_match() {
    let result = program(
        r#"
func Main() -> int {
    var values = System.Collections.ArrayList<int>.Allocate(0)
    values.Add(0)
    values.Add(42)
    var calls = 0
    let found = values.Find(value => { calls = calls + 1; return value.Equals(0) })
    if calls != 1 { return -1 }
    if !found.IsSome { return -2 }
    if !values.Find(value => value < 0).IsNone { return -3 }
    if values.FindIndex(value => value.Equals(42)) != 1 { return -4 }
    if values.FindIndex(value => value < 0) != -1 { return -5 }
    return 42
}
"#,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn empty_reference_list_needs_no_default_element_and_calls_no_predicate() {
    let result = program(
        r#"
record Item(Value: int)
func Main() -> int {
    let values = System.Collections.ArrayList<Item&>.Allocate(0)
    var calls = 0
    let predicate: System.Func<Item&, bool> = item => { calls = calls + 1; return true }
    if !values.Find(predicate).IsNone { return -1 }
    if values.Exists(predicate) { return -2 }
    if values.FindIndex(predicate) != -1 { return -3 }
    return 42 - calls
}
"#,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn reference_results_keep_heap_identity_and_survive_collection() {
    let p = program(
        r#"
record Item(Value: int)
func Find() -> Option<Item&> {
    var values = System.Collections.ArrayList<Item&>.Allocate(0)
    values.Add(new Item(40))
    return values.Find(item => { item.Value = item.Value + 2; return true })
}
func Main() -> int {
    let result = Find()
    for i in 0..<30 { let ignored = Find() }
    return result match { Some(let item) => item.Value, None => -1 }
}
"#,
    );
    let result = p
        .run(Limits {
            heap_objects: 16,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn predicate_search_captures_initial_buffer_and_returns_tested_value() {
    let result = program(
        r#"
func Main() -> int {
    var values = System.Collections.ArrayList<int>.Allocate(1)
    values.Add(42)
    let found = values.Find(value => { values[0] = 7; values.Add(99); return true })
    return found match { Some(let value) => value, None => -1 }
}
"#,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn predicates_are_checked_and_faults_keep_guest_trace() {
    let error = program(
        r#"
func Main() -> int {
    var values = System.Collections.ArrayList<int>.Allocate(0)
    values.Add(0)
    return values.FindIndex(value => 1 / value > 0)
}
"#,
    )
    .run(Limits::default())
    .unwrap_err();
    assert!(
        error
            .stack_trace
            .unwrap()
            .frames
            .iter()
            .any(|f| f.function.name.contains("FindIndex"))
    );
    assert!(frontend::compile("func Main() -> int { let values = System.Collections.ArrayList<int>.Allocate(0); return values.FindIndex(value => value) }").is_err());
}

#[test]
fn linked_validation_still_rejects_unknown_union_payload_types() {
    assert!(
        frontend::compile(
            r#"
func Read(value: Option<Missing>) -> int {
    return value match { Some(_) => 1, None => 0 }
}
func Main() -> int { return 0 }
"#
        )
        .is_err()
    );
}
