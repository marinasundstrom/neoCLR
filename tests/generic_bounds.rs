use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_type, frontend, load};

fn execute(source: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = frontend::compile(source)?;
    let module = load(&serde_json::to_string(&module).unwrap())?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    program.run(Limits::default())
}

const DECLARATIONS: &str = r#"
interface Readable { readonly func Read() -> int }
interface Counter: Readable { func Add(amount: int) -> () }
record Cell(Value: int): Counter {
    readonly func Read() -> int { return this.Value }
    func Add(amount: int) -> () { this.Value = this.Value + amount }
}
record Holder<T>(Value: T) where T: Counter
func Read<T>(readonly value: T&) -> int where T: Readable { return value.Read() }
func Add<T>(value: T&, amount: int) -> () where T: Counter { value.Add(amount) }
"#;

#[test]
fn borrowed_bounds_preserve_local_and_heap_identity() {
    let source = format!(
        r#"{DECLARATIONS}
func Main() -> int {{
    var local = Cell(19)
    Add(&local, 1)
    let heap = new Cell(20)
    let holder = Holder<Cell&>(heap)
    Add(holder.Value, 2)
    return Read(&local) + Read(holder.Value)
}}
"#
    );
    assert_eq!(execute(&source).unwrap().value, Value::Int32(42));
}

#[test]
fn base_bound_calls_virtual_override_without_value_slicing() {
    let source = r#"
abstract record Base(Value: int) { readonly abstract func Read() -> int }
record Derived(Extra: int): Base {
    readonly override func Read() -> int { return this.Value + this.Extra }
}
func Read<T>(readonly value: T&) -> int where T: Base { return value.Read() }
func Main() -> int { var value = Derived(40, 2); return Read(&value) }
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(42));
}

#[test]
fn wrong_bounds_and_readonly_mutation_are_rejected() {
    for tail in [
        "func Main() -> () { let h = Holder<int>(42) }",
        "func Main() -> () { var n = 42; Add(&n, 1) }",
        "func Bad<T>(readonly value: T&) -> () where T: Counter { value.Add(1) }\nfunc Main() -> () {}",
        "func Bad<T>(value: T&) -> int { return value.Read() }\nfunc Main() -> () {}",
        "func Bad<T>(value: T) -> int where T: Readable { return value.Read() }\nfunc Main() -> () {}",
    ] {
        assert!(
            execute(&format!("{DECLARATIONS}\n{tail}")).is_err(),
            "{tail}"
        );
    }
}

#[test]
fn raw_metadata_checks_bounds_and_substitution() {
    let module = assemble(
        r#"
.module Bounds
.interface Readable<T>
.end
.type Cell
.implements Readable<Int32>
.end
.type Other
.end
.type Holder<T,U>
.constraint T Readable<U>
.field Value T
.end
"#,
    )
    .unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    for good in [
        "Holder<Cell,Int32>",
        "Holder<Cell&,Int32>",
        "Holder<readonly Cell&,Int32>",
    ] {
        assert!(
            module
                .instantiated_fields(&parse_type(good).unwrap())
                .is_ok(),
            "{good}"
        );
    }
    for bad in [
        "Holder<Other,Int32>",
        "Holder<Cell,String>",
        "Holder<Int32,Int32>",
        "Holder<Cell*,Int32>",
    ] {
        assert!(
            module
                .instantiated_fields(&parse_type(bad).unwrap())
                .is_err(),
            "{bad}"
        );
    }
}

#[test]
fn bundled_generic_interface_bounds_substitute_method_parameters() {
    let source = r#"
func Compare<T>(readonly left: T&, right: T) -> int where T: System.Comparable<T> {
    return left.CompareTo(right)
}
record Helpers() {
    static func Compare<T>(readonly left: T&, right: T) -> int where T: System.Comparable<T> {
        return Compare<T>(left, right)
    }
}
func Main() -> int { var n = 42; return Helpers.Compare(&n, 42) }
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(0));
}

#[test]
fn constrained_dispatch_uses_explicit_and_default_implementations() {
    let source = r#"
interface Readable {
    readonly func Read() -> int
    readonly func Twice() -> int { return this.Read() + this.Read() }
}
record Cell(Value: int): Readable { readonly func Readable.Read() -> int { return this.Value } }
func Twice<T>(readonly value: T&) -> int where T: Readable { return value.Twice() }
func Main() -> int { var value = Cell(21); return Twice(&value) }
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(42));
}

#[test]
fn host_calls_cannot_bypass_bounds_and_verifier_requires_evidence() {
    use neoclr::assembler::parse_function_ref;
    let module = assemble(
        r#"
.module Bounds
.interface Marker
.end
.type Good
.implements Marker
.end
.type Bad
.end
.function Accept<T>(T value) -> T
.constraint T Marker
ldarg value
ret
.end
.function Forward<T>(T value) -> T
ldarg value
call Accept<T>(T)
ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(
        program
            .resolve_function(&parse_function_ref("Accept<Good>(Good)").unwrap())
            .is_ok()
    );
    assert!(
        program
            .resolve_function(&parse_function_ref("Accept<Bad>(Bad)").unwrap())
            .is_err()
    );
    let forward = program
        .resolve_function(&parse_function_ref("Forward<Bad>(Bad)").unwrap())
        .unwrap();
    assert!(
        forward
            .invoke(
                vec![Value::Object {
                    ty: parse_type("Bad").unwrap(),
                    fields: vec![]
                }],
                Limits::default()
            )
            .is_err()
    );
    let unproven = assemble(".module Unproven\n.interface Marker\n.end\n.function Borrow<T>(T& value) -> Marker&\nldarg value\ninterface.borrow Marker\nret\n.end").unwrap();
    assert!(LoadedProgram::new(&unproven).unwrap().verify().is_err());
}

#[test]
fn malformed_nominal_bounds_are_rejected() {
    for bound in [
        "Missing",
        "Int32",
        "Base&",
        "T",
        "Base Base",
        "Base Other",
        "Contract<Int32,Int32>",
        "Contract<!!0>",
    ] {
        let source = format!(
            ".module Bad\n.type Base\n.end\n.type Other\n.end\n.interface Contract<T>\n.end\n.type Holder<T>\n.constraint T {bound}\n.end"
        );
        assert!(assemble(&source).is_err(), "{bound}");
    }
}

#[test]
fn scoped_bounds_require_module_references_and_preserve_serialized_identity() {
    use neoclr::{assembler::assemble_modules, library};
    let contracts = ".module Contracts\n.references ()\n.interface Marker\n.end\n.type Cell\n.implements Marker\n.end";
    let app = ".module App\n.references (Contracts)\n.type Holder<T>\n.constraint T [Contracts]Marker\n.field Value T\n.end\n.function Accept<T>(T value) -> T\n.constraint T [Contracts]Marker\nldarg value\nret\n.end";
    let modules = assemble_modules(&[app, contracts]).unwrap();
    let original = serde_json::to_value(&modules).unwrap();
    assert!(original.to_string().contains("Scoped"));
    let program =
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap();
    assert!(
        program
            .resolve_function(
                &neoclr::assembler::parse_function_ref("Accept<[Contracts]Cell>([Contracts]Cell)")
                    .unwrap()
            )
            .is_ok()
    );
    assert_eq!(original, serde_json::to_value(&modules).unwrap());
    let mut missing = modules.clone();
    missing[0].references = Some(vec![]);
    assert!(
        LoadedProgram::with_modules(&missing[0], library::system().unwrap(), &missing[1..])
            .is_err()
    );
}

#[test]
fn example_runs() {
    let result = execute(include_str!("../examples/source/generic-bounds.neo")).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["42"]);
}

#[test]
fn bound_types_substitute_owner_and_method_namespaces_independently() {
    let module = assemble(
        r#"
.module Independent
.interface Marker<T>
.end
.type Good
.implements Marker<Int32>
.end
.type Host<T>
.method static Accept<U>(U value) -> U
.constraint U Marker<T>
ldarg value
ret
.end
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let target = |owner| {
        neoclr::assembler::parse_function_ref(&format!("Host<{owner}>::Accept<Good>(Good)"))
            .unwrap()
    };
    assert!(program.resolve_function(&target("Int32")).is_ok());
    assert!(program.resolve_function(&target("String")).is_err());
}

#[test]
fn ambiguous_bounds_and_value_receiver_copying_are_not_silently_selected() {
    let ambiguous = r#"
interface First { readonly func Read() -> int }
interface Second { readonly func Read() -> int }
func Read<T>(readonly value: T&) -> int where T: First, Second { return value.Read() }
func Main() -> () {}
"#;
    assert!(
        frontend::compile(ambiguous)
            .unwrap_err()
            .message
            .contains("ambiguous constrained member")
    );
    let copying = r#"
func Read<T>(value: T&) -> int where T: System.Option.Some<int> { return value.get_Value() }
func Main() -> () {}
"#;
    assert!(
        frontend::compile(copying)
            .unwrap_err()
            .message
            .contains("byref method receiver")
    );
}
