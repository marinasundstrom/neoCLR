use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> Value {
    let module = frontend::compile(source).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&loaded)
        .unwrap()
        .run(Limits::default())
        .unwrap()
        .value
}

#[test]
fn imports_resolve_constructors_annotations_and_generic_arguments_file_wide() {
    let source = r#"
import Shape.*
func Measure(circle: Circle) -> int { return circle.radius }
func Identity<Circle>(value: Circle) -> Circle { return value }
union Shape { case Circle(radius: int); case Empty }
func Main() -> int {
    let circle: Circle = Circle(40)
    let shape: Shape = circle
    let Circle(copy) = shape else { return -1 }
    let heap: Circle& = new Circle(2)
    let qualified = new Shape.Circle(2)
    if qualified.radius != heap.radius { return -3 }
    var circles = System.Collections.ArrayList<Circle>.Allocate(1)
    circles.Add(copy)
    if !typeof(Circle).Equals(typeof(Shape.Circle)) { return -2 }
    return Measure(circles[0]) + Identity(heap.radius)
}
"#;
    assert_eq!(run(source), Value::Int32(42));
}

#[test]
fn order_workflow_constructs_imported_error_cases() {
    assert_eq!(
        run(include_str!("../examples/source/order-workflow.neo")),
        Value::Int32(0)
    );
}

#[test]
fn duplicate_imports_are_idempotent_and_local_callables_shadow_cases() {
    let source = r#"
import Shape.*
import Shape.*
union Shape { case Circle(radius: int) }
func Main() -> int {
    let Circle: System.Func<int, int> = value => value + 2
    let explicit = Shape.Circle(40)
    return Circle(explicit.radius)
}
"#;
    assert_eq!(run(source), Value::Int32(42));
}

#[test]
fn ambiguity_is_reported_on_use_and_qualified_names_remain_available() {
    let declarations =
        "import A.*\nimport B.*\nunion A { case Item(n: int) }\nunion B { case Item(n: int) }\n";
    assert_eq!(
        run(&format!(
            "{declarations}func Main() -> int {{ return A.Item(42).n }}"
        )),
        Value::Int32(42)
    );
    for body in ["let value = Item(42)", "let value: Item = A.Item(42)"] {
        let error = frontend::compile(&format!("{declarations}func Main() -> () {{ {body} }}"))
            .unwrap_err();
        assert!(
            error.message.contains("ambiguous imported case"),
            "{error:?}"
        );
    }
    assert!(
        frontend::compile("import Missing.*\nfunc Main() -> () {}")
            .unwrap_err()
            .message
            .contains("declared source union")
    );
}

#[test]
fn file_declarations_take_precedence() {
    assert_eq!(
        run(
            "import U.*\nunion U { case Item(n: int) }\nrecord Item(n: int)\nfunc Main() -> int { let x: Item = Item(42); return x.n }"
        ),
        Value::Int32(42)
    );
    assert_eq!(
        run(
            "import U.*\nunion U { case Item(n: int) }\nfunc Item(n: int) -> int { return n + 2 }\nfunc Main() -> int { return Item(40) }"
        ),
        Value::Int32(42)
    );
}

#[test]
fn imports_do_not_replace_predefined_type_names() {
    assert_eq!(
        run(
            "import U.*\nunion U { case int(n: int) }\nfunc Main() -> int { let n: int = 42; return n }"
        ),
        Value::Int32(42)
    );
}

#[test]
fn bundled_cases_support_inference_explicit_arguments_and_type_positions() {
    let source = r#"
func Wrap<T>(value: T) -> Some<T> { return Some(value) }
import System.Result.*
import System.Option.*
import System.Result.*
record Counter(Value: int)
func Main() -> int {
    let number: Ok<int> = Ok(42)
    let flag: Ok<bool> = Ok(true)
    let message: Error<string> = Error("no")
    let result: Result<int, string> = number
    let failure: Result<int, string> = message
    let explicit = Ok<int>(42)
    let counter = new Counter(40)
    let refCase: Some<Counter&> = Wrap(counter)
    refCase.Value.Value = 42
    if !ReferenceEquals(refCase.Value, counter) { return -1 }
    if !typeof(Ok<int>).Equals(typeof(System.Result.Ok<int>)) { return -2 }
    let absent: Option<int> = None()
    let present: Option<int> = Some(explicit.Value)
    let Some(value) = present else { return -3 }
    if let None = absent { if flag.Value { return value } }
    return -4
}
"#;
    assert_eq!(run(source), Value::Int32(42));
}

#[test]
fn imported_constructor_inference_keeps_stable_expression_identity() {
    let mut expression = "Next(&calls)".to_owned();
    for _ in 0..24 {
        expression = format!("Some({expression})");
    }
    assert_eq!(
        run(&format!(
            "import System.Option.*\nfunc Next(calls: int&) -> int {{ calls = calls + 1; return 42 }} func Main() -> int {{ var calls = 0; let nested = {expression}; let number: Some<int> = Some(42); let flag: Some<bool> = Some(true); return calls }}"
        )),
        Value::Int32(1)
    );
}

#[test]
fn bundled_imports_keep_shadowing_and_ambiguity_rules() {
    assert_eq!(
        run(
            "import System.Result.*\nfunc Ok<T>(value: T) -> T { return value } func Main() -> int { return Ok<int>(42) }"
        ),
        Value::Int32(42)
    );
    assert_eq!(
        run(
            "import System.Result.*\nfunc Main() -> int { let Ok: System.Func<int,int> = value => value + 2; return Ok(40) }"
        ),
        Value::Int32(42)
    );
    assert_eq!(
        run(
            "import System.Result.*\nrecord Ok(Value: int)\nfunc Main() -> int { let value: Ok = Ok(42); return value.Value }"
        ),
        Value::Int32(42)
    );
    let prefix = "import System.Result.*\nimport Local.*\nunion Local { case Ok(Value: int) }\n";
    assert_eq!(
        run(&format!(
            "{prefix}func Main() -> int {{ return System.Result.Ok(42).Value }}"
        )),
        Value::Int32(42)
    );
    for statement in [
        "let value = Ok(42)",
        "let value = Ok<int>(42)",
        "let value: Ok<int> = System.Result.Ok(42)",
    ] {
        assert!(
            frontend::compile(&format!("{prefix}func Main() -> () {{ {statement} }}"))
                .unwrap_err()
                .message
                .contains("ambiguous imported case")
        );
    }
}

#[test]
fn imports_do_not_expand_runtime_or_constructor_contracts() {
    for source in [
        "import System.String.*\nfunc Main() -> () {}",
        "import System.Result.Ok.*\nfunc Main() -> () {}",
        "import System.Result.*\nfunc Main() -> () { let result: Result<Void,string> = Ok(42) }",
        "import System.Option.*\nfunc Main() -> () { let value = new Some(42) }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
    let module = frontend::compile("import System.Option.*\nfunc Main() -> Option<int&> { var value = 42; return Some(&value) }").unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}
