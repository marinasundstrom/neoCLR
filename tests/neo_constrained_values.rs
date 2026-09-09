use neoclr::{Limits, LoadedProgram, Value, frontend, load};

fn execute(source: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = frontend::compile(source)?;
    let module = load(&serde_json::to_string(&module).unwrap())?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    program.run(Limits::default())
}

const DEFINITIONS: &str = r#"
interface Readable { readonly func Read() -> int }
interface Counter: Readable { func Add(n: int) -> () }
record Cell(Value: int): Counter {
    readonly func Read() -> int { return this.Value }
    func Add(n: int) -> () { this.Value = this.Value + n }
}
func Read<T>(value: T) -> int where T: Readable, notreference { return value.Read() }
func Adjust<T>(value: T) -> int where T: Counter, notreference {
    var local = value
    local.Add(1)
    return local.Read()
}
"#;

#[test]
fn value_parameters_and_mutable_locals_keep_copy_semantics() {
    let source = format!(
        r#"{DEFINITIONS}
func Main() -> int {{
    let value = Cell(20)
    let changed = Adjust(value)
    if Read(value) != 20 {{ return -1 }}
    return changed + Read(Cell(21))
}}
"#
    );
    assert_eq!(execute(&source).unwrap().value, Value::Int32(42));
}

#[test]
fn generic_library_interface_and_static_methods_use_value_receivers() {
    let source = r#"
record Helpers() {
    static func Compare<T>(left: T, right: T) -> int where T: System.Comparable<T>, notreference {
        return left.CompareTo(right)
    }
}
func Main() -> int { return Helpers.Compare(42, 42) }
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(0));
}

#[test]
fn mutating_immutable_value_receivers_and_unproven_address_modes_are_rejected() {
    for function in [
        "func Bad<T>(value: T) -> () where T: Counter, notreference { value.Add(1) }",
        "func Bad<T>(value: T) -> () where T: Counter, notreference { let local = value; local.Add(1) }",
        "func Bad<T>(value: T) -> int where T: Readable { return value.Read() }",
    ] {
        assert!(
            frontend::compile(&format!(
                "{DEFINITIONS}\n{function}\nfunc Main() -> () {{}}"
            ))
            .is_err(),
            "{function}"
        );
    }
    assert!(
        execute(&format!(
            "{DEFINITIONS}\nfunc Main() -> int {{ let value = new Cell(42); return Read(value) }}"
        ))
        .is_err()
    );
}

#[test]
fn shallow_value_constraint_keeps_shared_reference_fields() {
    let source = format!(
        r#"{DEFINITIONS}
record Facade(Target: Cell&): Counter {{
    readonly func Read() -> int {{ return this.Target.Value }}
    func Add(n: int) -> () {{ this.Target.Add(n) }}
}}
func Main() -> int {{
    let target = new Cell(41)
    let facade = Facade(target)
    if Adjust(facade) != 42 {{ return -1 }}
    return target.Value
}}
"#
    );
    assert_eq!(execute(&source).unwrap().value, Value::Int32(42));
}

#[test]
fn base_bound_on_value_receiver_preserves_override_dispatch() {
    let source = r#"
abstract record Base(Value: int) { readonly abstract func Read() -> int }
record Derived(Extra: int): Base { readonly override func Read() -> int { return this.Value + this.Extra } }
func Read<T>(value: T) -> int where T: Base, notreference { return value.Read() }
func Main() -> int { return Read(Derived(40, 2)) }
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(42));
}

#[test]
fn field_receiver_effects_run_once_and_array_elements_are_addressable() {
    let source = format!(
        r#"{DEFINITIONS}
record Box<T>(Value: T)
func Get<T>(box: Box<T>&, hits: Cell&) -> Box<T>& {{ hits.Value = hits.Value + 1; return box }}
func ReadBox<T>(box: Box<T>&, hits: Cell&) -> int where T: Readable, notreference {{
    return Get<T>(box, hits).Value.Read()
}}
func ReadArray<T>(values: T[]&) -> int where T: Readable, notreference {{ return values[0].Read() }}
func Main() -> int {{
    let hits = new Cell(0)
    let box = new Box<Cell>(Cell(20))
    let values: Cell[]& = new Cell[1] {{ Cell(22) }}
    let result = ReadBox(box, hits) + ReadArray(values)
    if hits.Value != 1 {{ return -1 }}
    return result
}}
"#
    );
    assert_eq!(execute(&source).unwrap().value, Value::Int32(42));
}

#[test]
fn value_receiver_view_cannot_escape_the_callee_copy() {
    let source = r#"
interface Viewable { readonly func View() -> readonly Cell& }
record Cell(Value: int): Viewable { readonly func View() -> readonly Cell& { return this } }
func View<T>(value: T) -> readonly Cell& where T: Viewable, notreference { return value.View() }
func Main() -> int { return View(Cell(42)).Value }
"#;
    let error = execute(source).unwrap_err();
    assert!(error.message.contains("frame"), "{error:?}");
}

#[test]
fn example_demonstrates_independent_value_mutation() {
    let result = execute(include_str!("../examples/source/constrained-values.neo")).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["42"]);
}
