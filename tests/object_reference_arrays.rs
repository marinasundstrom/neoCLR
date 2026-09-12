use neoclr::{Limits, Value, assemble, run, verify};

const TYPES: &str = r#"
.interface Read
.method instance Get() -> Int32
.end
.end
.type class Cell
.implements Read
.field Value Int32
.method instance Get() -> Int32
ldarg this
ldfld Cell::Value
ret
.end
.end
"#;
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module ReferenceArrays\n.entry Main\n{TYPES}{body}"
    ))
    .unwrap()
}

#[test]
fn object_reference_elements_default_to_typed_null() {
    for element in ["Cell", "Read"] {
        let m = module(&format!(
            ".function Main() -> {element}\nldc.i4 1\narray.new {element}\nldc.i4 0\nldelem {element}\nret\n.end"
        ));
        verify(&m).unwrap();
        let execution = run(&m, Limits::default()).unwrap();
        assert_eq!(
            execution.value,
            Value::NullObjectReference(neoclr::metadata::Type::Named(element.into()))
        );
        assert_eq!(execution.heap.statistics().allocated_objects, 1);
    }
}

#[test]
fn interface_array_element_retains_identity_and_dispatch() {
    let m = module(
        r#"
.function Main() -> Int32
.local Cell original
.local Read[]& buffer
ldc.i4 1
array.new Read
stloc buffer
ldc.i4 7
newobj Cell
stloc original
ldloc buffer
ldc.i4 0
ldloc original
castclass Read
stelem Read
ldloc original
ldc.i4 42
stfld Cell::Value
ldloc buffer
ldc.i4 0
ldelem Read
ldloc original
ref.eq
brfalse Bad
ldloc buffer
ldc.i4 0
ldelem Read
callvirt instance Read::Get()
ret
Bad:
ldc.i4 0
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn interface_element_slot_can_be_rebound_without_mutating_old_object() {
    let m = module(
        r#"
.function Main() -> Int32
.local Read[]& buffer
.local Read& slot
.local Read old
ldc.i4 1
array.new Read
stloc buffer
ldloc buffer
ldc.i4 0
ldelema Read
stloc slot
ldloc slot
ldc.i4 7
newobj Cell
castclass Read
stobj Read
ldloc slot
ldobj Read
stloc old
ldloc slot
ldc.i4 35
newobj Cell
castclass Read
stobj Read
ldloc old
callvirt instance Read::Get()
ldloc buffer
ldc.i4 0
ldelem Read
callvirt instance Read::Get()
add
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn element_slot_roots_array_and_object_across_return_and_gc() {
    let m = module(
        r#"
.function Main() -> Int32
.local Read& slot
call Make()
stloc slot
ldc.i4 0
newobj Cell
pop
ldc.i4 1
newobj Cell
pop
ldloc slot
ldobj Read
callvirt instance Read::Get()
ret
.end
.function Make() -> Read&
.local Read[]& buffer
ldc.i4 1
array.new Read
stloc buffer
ldloc buffer
ldc.i4 0
ldc.i4 42
newobj Cell
castclass Read
stelem Read
ldloc buffer
ldc.i4 0
ldelema Read
ret
.end
"#,
    );
    let restored = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    verify(&restored).unwrap();
    let execution = run(
        &restored,
        Limits {
            heap_objects: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(execution.value, Value::Int32(42));
    assert!(execution.heap.statistics().collections > 0);
}

#[test]
fn incompatible_element_types_are_rejected_without_verification() {
    for value in ["ldc.i4 1", "ldc.i4 1\nnewobj Cell"] {
        let m = module(&format!(
            ".function Main() -> noresult\nldc.i4 1\narray.new Read\nldc.i4 0\n{value}\nstelem Read\nret\n.end"
        ));
        assert!(verify(&m).is_err());
        assert!(run(&m, Limits::default()).is_err());
    }
}

#[test]
fn null_element_dispatch_faults() {
    let m = module(
        ".function Main() -> Int32\nldc.i4 1\narray.new Read\nldc.i4 0\nldelem Read\ncallvirt instance Read::Get()\nret\n.end",
    );
    verify(&m).unwrap();
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("null interface receiver")
    );
}

#[test]
fn frame_interface_slot_cannot_escape_even_when_object_is_on_heap() {
    let m = module(
        r#"
.function Main() -> Read&
call Escape()
ret
.end
.function Escape() -> Read&
.local Read item
ldc.i4 42
newobj Cell
castclass Read
stloc item
ldloca item
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn nominal_generic_buffer_can_retain_an_interface_array() {
    let m = module(
        r#"
.type class Buffer<T>
.field Items T[]&
.end
.function Main() -> Int32
.local Buffer<Read> buffer
ldc.i4 1
array.new Read
newobj Buffer<Read>
stloc buffer
ldloc buffer
ldfld Buffer<Read>::Items
ldc.i4 0
ldc.i4 42
newobj Cell
castclass Read
stelem Read
ldloc buffer
ldfld Buffer<Read>::Items
ldc.i4 0
ldelem Read
callvirt instance Read::Get()
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn reference_array_elements_remain_invariant() {
    let m = module(
        r#"
.function Main() -> Read
.local Cell[]& items
ldc.i4 1
array.new Cell
stloc items
ldloc items
ldc.i4 0
ldelem Read
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn legacy_borrowed_view_is_not_an_interface_reference_slot() {
    let m = module(
        r#"
.type Legacy
.implements Read
.field Value Int32
.method instance Get() -> Int32
ldarg this
ldfld Legacy::Value
ret
.end
.end
.function Main() -> Read
.local Legacy value
ldc.i4 42
newobj Legacy
stloc value
ldloca value
interface.borrow Read
ldobj Read
ret
.end
"#,
    );
    assert!(run(&m, Limits::default()).is_err());
}
