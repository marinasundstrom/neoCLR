use neoclr::{Limits, LoadedProgram, Value, frontend, load};

fn execute(source: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = frontend::compile(source)?;
    let module = load(&serde_json::to_string(&module).unwrap())?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    program.run(Limits::default())
}

const DECLARATIONS: &str = r#"
interface Readable { readonly func Read() -> int }
interface Counter: Readable { func Add(n: int) -> () }
record Base(Value: int): Counter {
    readonly virtual func Read() -> int { return this.Value }
    func Add(n: int) -> () { this.Value = this.Value + n }
}
record Derived(Extra: int): Base {
    readonly override func Read() -> int { return this.Value + this.Extra }
}
record Selection(Current: readonly Readable&)
func Select<T>(selection: Selection&, value: T&) -> () where T: Base { selection.Current = value }
func Add(value: Counter&, n: int) -> () { value.Add(n) }
func View<T>(value: T&) -> Base& where T: Base { return value }
func Observe<T>(readonly value: T&) -> readonly Readable& where T: Base { return value }
func Update<T>(value: T&, n: int) -> () where T: Base {
    Add(value, n)
    let view: Base& = value
    view.Value = view.Value + 1
}
"#;

#[test]
fn assignments_arguments_and_returns_keep_original_objects_and_dispatch() {
    let source = format!(
        r#"{DECLARATIONS}
func Main() -> int {{
    var local = Derived(18, 1)
    Update(&local, 1)
    let heap = new Derived(18, 1)
    Update(heap, 1)
    let localView = View(&local)
    let heapView = Observe(heap)
    let selection = new Selection(new Derived(0, 0))
    Select(selection, heap)
    if !ReferenceEquals(selection.Current, heap) {{ return -3 }}
    if !ReferenceEquals(localView, &local) {{ return -1 }}
    if !ReferenceEquals(heapView, heap) {{ return -2 }}
    return Observe(&local).Read() + heapView.Read()
}}
"#
    );
    assert_eq!(execute(&source).unwrap().value, Value::Int32(42));
}

#[test]
fn inherited_interface_bounds_and_explicit_casts_prove_only_reachable_views() {
    let source = format!(
        r#"{DECLARATIONS}
func Narrow<T>(readonly value: T&) -> readonly Readable& where T: Counter {{
    return value as readonly Readable&
}}
func Main() -> int {{ var local = Derived(40, 2); return Narrow(&local).Read() }}
"#
    );
    assert_eq!(execute(&source).unwrap().value, Value::Int32(42));
}

#[test]
fn bundled_generic_interface_views_substitute_without_copying() {
    let source = r#"
func Compare<T>(readonly value: T&, other: T) -> int where T: System.Comparable<T> {
    let view: readonly System.Comparable<T>& = value
    return view.CompareTo(other)
}
func Main() -> int { var n = 42; return Compare(&n, 42) }
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(0));
}

#[test]
fn missing_evidence_downcasts_values_and_readonly_upgrades_are_rejected() {
    for function in [
        "func Bad<T>(value: T&) -> Base& { return value }",
        "func Bad<T>(value: T&) -> Derived& where T: Base { return value }",
        "func Bad<T>(value: T&) -> Base& where T: Counter { return value }",
        "func Bad<T>(readonly value: T&) -> Base& where T: Base { return value }",
        "func Bad<T>(value: T) -> Base& where T: Base { return value }",
        "func Bad<T>(value: T&) -> Base where T: Base { return value }",
        "func Initialize(out value: Base&) -> () { value = Base(42) }\nfunc Bad<T>(value: T&) -> () where T: Base { Initialize(out value) }",
        "record Box<T>(Value: T)\nfunc Bad<T>(value: Box<T&>) -> Box<Base&> where T: Base { return value }",
    ] {
        assert!(
            frontend::compile(&format!(
                "{DECLARATIONS}\n{function}\nfunc Main() -> () {{}}"
            ))
            .is_err(),
            "{function}"
        );
    }
}

#[test]
fn projected_reference_cannot_escape_its_frame() {
    let source = format!(
        r#"{DECLARATIONS}
func Escape() -> Base& {{ var local = Derived(40, 2); return View(&local) }}
func Main() -> int {{ return Escape().Read() }}
"#
    );
    assert!(execute(&source).is_err());
}

#[test]
fn example_keeps_views_of_local_and_heap_instances() {
    let result = execute(include_str!("../examples/source/constrained-views.neo")).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["42"]);
}

#[test]
fn source_base_bound_can_prove_a_bundled_interface_view() {
    let source = r#"
record Score(Value: int): System.Comparable<int> {
    readonly func CompareTo(other: int) -> int { return this.Value - other }
}
func View<T>(readonly value: T&) -> readonly System.Comparable<int>& where T: Score { return value }
func Main() -> int { var value = Score(42); return View(&value).CompareTo(0) }
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(42));
}

#[test]
fn returned_heap_view_outlives_the_allocating_function() {
    let source = format!(
        r#"{DECLARATIONS}
func Make() -> readonly Readable& {{
    let value = new Derived(40, 2)
    return Observe(value)
}}
func Main() -> int {{ let view = Make(); return view.Read() }}
"#
    );
    assert_eq!(execute(&source).unwrap().value, Value::Int32(42));
}
