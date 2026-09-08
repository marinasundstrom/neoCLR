use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&loaded)
        .unwrap()
        .run(Limits::default())
        .unwrap()
}

#[test]
fn locals_fields_arrays_and_collection_slots_copy_references() {
    let source = r#"
record Counter(Age: int)
record Holder(Item: Counter&)
func Main() -> int {
    let first = new Counter(1)
    let second = new Counter(10)
    var selected = first
    selected = second
    selected.Age = 11
    let holder = new Holder(first)
    holder.Item = second
    holder.Item.Age = 12
    let array = new Counter&[1] { first }
    array[0] = second
    array[0].Age = 13
    var list = System.Collections.ArrayList<Counter&>.Allocate(1)
    list.Add(first)
    list[0] = second
    list[0].Age = 14
    if first.Age != 1 { return -1 }
    return selected.Age + holder.Item.Age + array[0].Age
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn captured_reference_binding_retargets_and_factory_runs_once() {
    let source = r#"
record Counter(Age: int)
func Next(calls: int&) -> Counter& {
    calls = calls + 1
    return new Counter(40)
}
func Main() -> int {
    let first = new Counter(1)
    var selected = first
    let read: System.Func<int> = () => selected.Age
    var calls = 0
    selected = Next(&calls)
    selected.Age = selected.Age + 1
    if first.Age != 1 { return -1 }
    return read() + calls
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn output_reference_still_writes_the_target_and_value_copy_is_explicit() {
    let source = r#"
record Counter(Age: int)
func Initialize(out target: Counter&, source: Counter&) -> () { target = source }
func Main() -> int {
    let first = new Counter(1)
    let second = new Counter(40)
    let alias = first
    Initialize(out first, second)
    second.Age = 99
    let snapshot: Counter = alias
    first = snapshot
    return alias.Age + 2
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn immutable_bindings_parameters_and_immutable_containers_cannot_be_retargeted() {
    for source in [
        "record C(N: int)\nfunc Main() -> () { let a = new C(1); let b = new C(2); a = b }",
        "record C(N: int)\nfunc Replace(a: C&, b: C&) -> () { a = b }\nfunc Main() -> () {}",
        "record C(N: int)\nrecord H(Item: C&)\nfunc Main() -> () { let a = new C(1); let h = H(a); h.Item = new C(2) }",
        "record C(N: int)\nfunc Main() -> () { let a = new C(1); let b: readonly C& = a; var c = a; c = b }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn reference_slot_stores_do_not_hide_frame_escape() {
    let source = r#"
record C(N: int)
record H(Item: C&)
func Main() -> () {
    let h = new H(new C(1))
    var local = C(2)
    h.Item = &local
}
"#;
    let module = frontend::compile(source).unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn inferred_alias_invokes_members_on_the_same_object() {
    let source = r#"
record Foo(Count: int) {
    func Bar() -> () { this.Count = this.Count + 1 }
}
func Main() -> int {
    let foo = new Foo(40)
    let foo2 = foo
    foo2.Bar()
    foo.Bar()
    return foo2.Count
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn slot_owner_index_and_reference_rhs_are_evaluated_once_in_order() {
    let source = r#"
record C(N: int)
record H(Item: C&)
func Owner(h: H&, sequence: int&) -> H& { sequence = sequence * 10 + 1; return h }
func Index(sequence: int&) -> int { sequence = sequence * 10 + 2; return 0 }
func Source(c: C&, sequence: int&) -> C& { sequence = sequence * 10 + 3; return c }
func Main() -> int {
    let a = new C(1)
    let b = new C(42)
    let h = new H(a)
    var sequence = 0
    Owner(h, &sequence).Item = Source(b, &sequence)
    if sequence != 13 { return -1 }
    let array = new C&[1] { a }
    sequence = 0
    array[Index(&sequence)] = Source(b, &sequence)
    if sequence != 23 { return -2 }
    if a.N != 1 { return -3 }
    return array[0].N
}
"#;
    assert_eq!(run(source).value, Value::Int32(42));
}

#[test]
fn base_interface_readonly_and_generic_views_keep_identity_and_virtual_dispatch() {
    let result = run(include_str!("../examples/source/reference-views.neo"));
    assert_eq!(result.output, ["42", "42", "42"]);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn reassigned_interface_view_keeps_complete_heap_owner_alive() {
    let source = r#"
interface Readable { readonly func Read() -> int }
record C(N: int): Readable { readonly func Read() -> int { return this.N } }
func Make() -> readonly Readable& {
    var selected: readonly Readable& = new C(1)
    selected = new C(42)
    return selected
}
func Discard() -> () { let temporary = new C(0) }
func Main() -> int {
    let view = Make()
    Discard()
    Discard()
    Discard()
    return view.Read()
}
"#;
    let module = frontend::compile(source).unwrap();
    let result = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits {
            heap_objects: 2,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.reclaimed_objects() >= 3);
}

#[test]
fn readonly_container_view_cannot_replace_its_reference_field() {
    let source = r#"
record C(N: int)
record H(Item: C&)
func Replace(readonly holder: H&, replacement: C&) -> () {
    holder.Item = replacement
}
func Main() -> () {
    let original = new C(1)
    let holder = new H(original)
    Replace(holder, new C(2))
}
"#;
    match frontend::compile(source) {
        Err(_) => (),
        Ok(module) => {
            let fault = LoadedProgram::new(&module)
                .unwrap()
                .run(Limits::default())
                .unwrap_err();
            assert!(fault.message.contains("readonly"), "{fault:?}");
        }
    }
}
