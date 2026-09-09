use neoclr::{Limits, LoadedProgram, Value, assemble, frontend, load};
fn program(source: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = frontend::compile(source)?;
    LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap())?)
}
const DEFINITIONS: &str = r#"
interface Readable { readonly func Read() -> int }
interface Counter: Readable { func Add(n: int) -> () }
record Cell(Value: int): Counter {
    readonly func Read() -> int { return this.Value }
    func Add(n: int) -> () { this.Value = this.Value + n }
}
func Read<T>(value: T) -> int where T: Readable { return value.Read() }
func Add<T>(value: T) -> () where T: Counter { value.Add(1) }
func Adjust<T>(value: T) -> int where T: Counter {
    var local = value
    local.Add(1)
    return local.Read()
}
"#;

#[test]
fn one_generic_body_preserves_value_copies_and_reference_identity() {
    let source = format!(
        r#"{DEFINITIONS}
func Main() -> int {{
    let value = Cell(20)
    if Adjust(value) != 21 {{ return -1 }}
    if Read(value) != 20 {{ return -2 }}
    let heap = new Cell(19)
    Add(heap)
    let view: Counter& = heap
    Add(view)
    if Adjust(heap) != 22 {{ return -3 }}
    if Read(heap) != 22 {{ return -4 }}
    return Read(value) + Read(view)
}}
"#
    );
    let program = program(&source).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn immutable_values_and_readonly_references_cannot_acquire_writable_access() {
    for setup in [
        "let value = Cell(42)",
        "let value: readonly Cell& = new Cell(42)",
    ] {
        let source = format!("{DEFINITIONS}\nfunc Main() -> () {{ {setup}; Add(value) }}");
        let program = program(&source).unwrap();
        program.verify().unwrap(); // Open T's capability is checked at concrete execution.
        let error = program.run(Limits::default()).unwrap_err();
        assert!(error.message.contains("readonly"), "{error:?}");
    }
}

#[test]
fn readonly_reference_receiver_can_call_readonly_members() {
    let source = format!(
        "{DEFINITIONS}\nfunc Main() -> int {{ let value: readonly Cell& = new Cell(42); return Read(value) }}"
    );
    assert_eq!(
        program(&source)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn managed_receiver_adaptation_does_not_extend_frame_lifetimes() {
    let source = r#"
interface Viewable { readonly func View() -> readonly Cell& }
record Cell(Value: int): Viewable { readonly func View() -> readonly Cell& { return this } }
func View<T>(value: T) -> readonly Cell& where T: Viewable { return value.View() }
func Make() -> readonly Cell& { return View(new Cell(42)) }
func Main() -> int { return Make().Value }
"#;
    let p = program(source).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let bad = source.replace("View(new Cell(42))", "View(Cell(42))");
    assert!(
        program(&bad)
            .unwrap()
            .run(Limits::default())
            .unwrap_err()
            .message
            .contains("frame")
    );
}

fn raw(body: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = assemble(&format!(
        ".module Receivers\n.entry Main\n.function Main() -> Int32\n{body}\n.end"
    ))?;
    LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap())?)
}

#[test]
fn raw_receiver_load_uses_exact_slot_or_reference_and_checks_readonly() {
    let prefix = ".local Int32 value\n.local Int32& alias\nldc.i4 41\nstloc value\nldloca value\nstloc alias\n";
    let p = raw(&format!(
        "{prefix}ldreceiver local alias readonly\nldc.i4 42\nstobj Int32\nldloc value\nret"
    ))
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let bad = raw(&format!(
        "{prefix}ldreceiver local value readonly\nldc.i4 42\nstobj Int32\nldloc value\nret"
    ))
    .unwrap();
    assert!(bad.verify().is_err());
    assert!(bad.run(Limits::default()).is_err());
    let readonly = raw(".local Int32 value\n.local readonly Int32& alias\nldc.i4 41\nstloc value\nldloca value\nstloc alias\nldreceiver local alias\nldc.i4 42\nstobj Int32\nldloc value\nret").unwrap();
    assert!(readonly.verify().is_err());
    assert!(readonly.run(Limits::default()).is_err());
}

#[test]
fn invalid_receiver_indices_operands_and_uninitialized_slots_are_rejected() {
    for op in [
        "ldreceiver",
        "ldreceiver field 0",
        "ldreceiver local 0 unknown",
        "ldreceiver local 4",
        "ldreceiver arg 0",
    ] {
        assert!(raw(&format!("{op}\nldobj Int32\nret")).is_err(), "{op}");
    }
    let p = raw(".local Int32 value\nldreceiver local value\nldobj Int32\nret").unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn adaptive_receivers_keep_explicit_default_and_base_virtual_dispatch() {
    let source = r#"
interface Readable {
    readonly func Read() -> int
    readonly func Twice() -> int { return this.Read() + this.Read() }
}
record Cell(Value: int): Readable { readonly func Readable.Read() -> int { return this.Value } }
abstract record Base(Value: int) { readonly abstract func Read() -> int }
record Derived(Extra: int): Base { readonly override func Read() -> int { return this.Value + this.Extra } }
func Twice<T>(value: T) -> int where T: Readable { return value.Twice() }
func Read<T>(value: T) -> int where T: Base { return value.Read() }
func Main() -> int {
    if Twice(Cell(21)) != 42 { return -1 }
    if Twice(new Cell(21)) != 42 { return -2 }
    if Read(Derived(40, 2)) != 42 { return -3 }
    return Read(new Derived(40, 2))
}
"#;
    let p = program(source).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn constructor_receiver_and_unassigned_output_cannot_be_adapted() {
    assert!(assemble(".module Bad\n.type Cell\n.method instance .ctor() -> Void\nldreceiver arg this\npop\nldvoid\nret\n.end\n.end").is_err());
    let module = assemble(".module Outputs\n.entry Main\n.function Bad(out Int32& value) -> Void\nldreceiver arg value\npop\nldvoid\nret\n.end\n.function Main() -> Int32\n.local Int32 value\nldloca value\ncall Bad(Int32&)\npop\nldloc value\nret\n.end").unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn adaptive_frontend_requires_direct_storage_and_declared_members() {
    for f in [
        "func Bad<T>(value: T) -> int { return value.Read() }",
        "func Identity<T>(value: T) -> T { return value }\nfunc Bad<T>(value: T) -> int where T: Readable { return Identity(value).Read() }",
    ] {
        assert!(program(&format!("{DEFINITIONS}\n{f}\nfunc Main() -> () {{}}")).is_err());
    }
}

#[test]
fn example_runs_both_addressing_modes() {
    let p = program(include_str!("../examples/source/generic-receivers.neo")).unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["42"]);
}
