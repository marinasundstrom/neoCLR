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
