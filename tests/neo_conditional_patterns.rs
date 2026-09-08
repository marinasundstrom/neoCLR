use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

#[test]
fn order_workflow_uses_typed_errors_and_conditional_bindings() {
    let execution = run(include_str!("../examples/source/order-workflow.neo"));
    assert_eq!(execution.value, Value::Int32(0));
    assert_eq!(
        execution.output,
        [
            "Purchased: Coffee",
            "24",
            "7",
            "24",
            "Out of stock",
            "Orders complete"
        ]
    );
}

#[test]
fn conditions_evaluate_once_and_else_if_has_its_own_binding() {
    let execution = run(r#"
func Read(calls: int&, ok: bool) -> Result<int,string> {
    calls = calls + 1
    if ok { return Result<int,string>.Ok(20) }
    return Result<int,string>.Error("missing")
}
func Main() -> int {
    var calls = 0
    var total = 0
    if let Ok(value) = Read(&calls, false) { total = value }
    else if let Error(error) = Read(&calls, false) { total = 2 }
    let Ok(value) = Read(&calls, true) else { return -1 }
    if calls != 3 { return -2 }
    return total + value + value
}
"#);
    assert_eq!(execution.value, Value::Int32(42));
}

#[test]
fn guard_failure_exits_and_loop_control_preserves_definite_assignment() {
    let execution = run(r#"
union Choice { case Number(value: int)
case Empty }
func Make(n: int) -> Choice {
    if n < 2 { return Choice.Empty() }
    return Choice.Number(n)
}
func Read(n: int) -> int {
    let Number(value) = Make(n) else { return 0 }
    return value.value
}
func Main() -> int {
    var total = 0
    for i in 0..<5 {
        let Number(value) = Make(i) else { continue }
        total = total + value.value
    }
    for i in 0..<5 {
        let Number(value) = Make(i) else { break }
        return -1
    }
    let Empty = Make(0) else { return -2 }
    if let Empty = Make(1) { total = total + Read(33) }
    return total
}
"#);
    assert_eq!(execution.value, Value::Int32(42));
}

#[test]
fn successful_pattern_bindings_can_be_captured_by_returned_closures() {
    let execution = run(r#"
func Guard(text: string) -> System.Func<int> {
    let Ok(value) = Int32.Parse(text) else { return () => 0 }
    return () => value
}
func Conditional(text: string) -> System.Func<int> {
    if let Ok(value) = Int32.Parse(text) { return () => value }
    else { return () => 0 }
}
func Main() -> int { return Guard("20")() + Conditional("22")() + Guard("bad")() }
"#);
    assert_eq!(execution.value, Value::Int32(42));
}

#[test]
fn unavailable_bindings_fallthrough_bad_cases_and_mutation_are_rejected() {
    for source in [
        "func Main() -> int { if let Ok(v) = Int32.Parse(\"1\") {} return v }",
        "func Main() -> int { if let Ok(v) = Int32.Parse(\"1\") {} else { return v }; return 0 }",
        "func Main() -> int { let Ok(v) = Int32.Parse(\"1\") else {}; return v }",
        "func Main() -> int { let Ok(v) = Int32.Parse(\"1\") else { return v }; return v }",
        "func Main() -> int { let Missing(v) = Int32.Parse(\"1\") else { return 0 }; return v }",
        "func Main() -> int { if let Ok(v) = 42 {}; return 0 }",
        "func Main() -> int { let Ok(v) = Int32.Parse(\"1\") else { continue }; return v }",
        "func Main() -> int { let Ok(v) = Int32.Parse(\"1\") else { return 0 }; v = 2; return v }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
}
