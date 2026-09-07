use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> Value {
    let module = frontend::compile(source).unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap()
        .value
}

#[test]
fn structured_program_and_both_returning_branches() {
    assert_eq!(
        run(include_str!("../examples/source/control-flow.neo")),
        Value::Int32(21)
    );
    for (condition, expected) in [("true", 1), ("false", 2)] {
        assert_eq!(
            run(&format!(
                "func Main() -> int {{ if {condition} {{ return 1 }} else {{ return 2 }} }}"
            )),
            Value::Int32(expected)
        );
    }
    assert_eq!(
        run(
            "func Main() -> int { var n = 0; for i in 0..2 { for j in 0..3 { if j == 1 { continue }; if j == 3 { break }; n = n + 1 } }; return n }"
        ),
        Value::Int32(6)
    );
}

#[test]
fn ranges_evaluate_bounds_once_and_never_wrap() {
    for (range, count) in [
        ("0..2", 3),
        ("0..<2", 2),
        ("2..1", 0),
        ("2..<2", 0),
        ("2147483647..2147483647", 1),
        ("2147483646..<2147483647", 1),
        ("-2147483648..-2147483648", 1),
    ] {
        assert_eq!(
            run(&format!(
                "func Main() -> int {{ var n = 0; for i in {range} {{ n = n + 1; continue }}; return n }}"
            )),
            Value::Int32(count),
            "{range}"
        );
    }
    assert_eq!(
        run(
            "func Bound(n: int&) -> int { n = n + 1; return 3 }\nfunc Main() -> int { var n = 0; for i in 0..<Bound(&n) { }; return n }"
        ),
        Value::Int32(1)
    );
    assert_eq!(
        run("func Main() -> int { var n = 0; while false { n = 1 }; return n }"),
        Value::Int32(0)
    );
}

#[test]
fn comparisons_and_boolean_short_circuit_preserve_effects() {
    for expression in [
        "1 < 2",
        "2 > 1",
        "2 <= 2",
        "2 >= 2",
        "1 != 2",
        "true == true",
        "!false",
        "false || true && true",
        "false && (1 / 0 == 0) || true",
        "true || (1 / 0 == 0)",
    ] {
        assert_eq!(
            run(&format!("func Main() -> bool {{ return {expression} }}")),
            Value::Boolean(true),
            "{expression}"
        );
    }
    assert_eq!(
        run(
            "func Touch(x: int&) -> bool { x = x + 1; return true }\nfunc Main() -> int { var x = 0; let a = false && Touch(&x); let b = true || Touch(&x); let c = true && Touch(&x); return x }"
        ),
        Value::Int32(1)
    );
}

#[test]
fn scopes_and_loop_control_have_explicit_diagnostics() {
    for source in [
        "func Main() -> () { break }",
        "func Main() -> () { continue }",
        "func Main() -> int { if true { return 1 } }",
        "func Main() -> () { while 1 {} }",
        "func Main() -> int { if true { let x = 1 }; return x }",
        "func Main() -> () { for i in 0..2 { i = 3 } }",
        "func Main() -> int { for i in 0..2 {}; return i }",
        "func Main() -> () { loop { break; Console.WriteLine(1) } }",
        "func Main() -> () { var x = 0; var r = &x; loop { var y = 1; r = &y; break } }",
        "record R(X: int)\nfunc Main() -> () { loop { var r = R(1); let p = &r.X; break } }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
    let nested = format!(
        "func Main() -> () {{ {}{} }}",
        "loop {".repeat(140),
        "}".repeat(140)
    );
    assert!(
        frontend::compile(&nested)
            .unwrap_err()
            .message
            .contains("nesting limit")
    );
    assert_eq!(
        run(
            "record R(X: int)\nfunc Main() -> int { var outer = 0; if true { var local = R(1); local.X = 2; let heap = new R(40); let r = &heap.X; r = r + local.X; outer = r }; if true { let local = 3 }; return outer }"
        ),
        Value::Int32(42)
    );
}

#[test]
fn infinite_loop_obeys_runtime_budget() {
    let module = frontend::compile("func Main() -> () { loop {} }").unwrap();
    let error = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(error.message.contains("instruction"), "{error}");
}
