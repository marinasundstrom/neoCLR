use neoclr::{Limits, Value, assemble, run, verify};
const TYPES: &str = r#"
.interface Read<T>
.method instance Get() -> T
.end
.end
.interface Mutable<T>
.implements Read<T>
.method instance Set(T value) -> noresult
.end
.end
.type class Cell<T>
.implements Mutable<T>
.field Value T
.method instance Get() -> T
ldarg this
ldfld Cell<T>::Value
ret
.end
.method instance Set(T value) -> noresult
ldarg this
ldarg value
stfld Cell<T>::Value
ret
.end
.end
.type class Other
.field Value Int32
.end
.type class Holder
.field Item Read<Int32>
.method instance .ctor(Read<Int32> item) -> noresult
ldarg this
ldarg item
stfld Holder::Item
ret
.end
.end
"#;
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Assignability\n.entry Main\n{TYPES}{body}"
    ))
    .unwrap()
}
fn check(body: &str, expected: Value) {
    let m = module(body);
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, expected);
}

#[test]
fn locals_parameters_returns_and_parent_interfaces_convert_without_casts() {
    check(
        r#"
.function Main() -> Int32
.local Mutable<Int32> alias
ldc.i4 1
newobj Cell<Int32>
stloc alias
ldloc alias
ldc.i4 42
callvirt instance Mutable<Int32>::Set(Int32)
ldloc alias
call Forward(Read<Int32>)
callvirt instance Read<Int32>::Get()
ret
.end
.function Forward(Read<Int32>) -> Read<Int32>
ldc.i4 7
newobj Cell<Int32>
starg 0
call Make()
ret
.end
.function Make() -> Read<Int32>
ldc.i4 42
newobj Cell<Int32>
ret
.end
"#,
        Value::Int32(42),
    );
}

#[test]
fn constructor_arguments_and_class_fields_accept_implementing_references() {
    check(
        r#"
.function Main() -> Int32
.local Holder holder
ldc.i4 1
newobj Cell<Int32>
newobj instance Holder::.ctor(Read<Int32>)
stloc holder
ldloc holder
ldc.i4 42
newobj Cell<Int32>
stfld Holder::Item
ldloc holder
ldfld Holder::Item
callvirt instance Read<Int32>::Get()
ret
.end
"#,
        Value::Int32(42),
    );
}

#[test]
fn array_element_and_indirect_slot_stores_preserve_identity() {
    check(
        r#"
.function Main() -> Boolean
.local Cell<Int32> original
.local arrayref<Read<Int32>> buffer
ldc.i4 42
newobj Cell<Int32>
stloc original
ldc.i4 1
newarr Read<Int32>
stloc buffer
ldloc buffer
ldc.i4 0
ldloc original
stelem Read<Int32>
ldloc buffer
ldc.i4 0
ldelema Read<Int32>
ldloc original
stobj Read<Int32>
ldloc buffer
ldc.i4 0
ldelem Read<Int32>
ldloc original
ref.eq
ret
.end
"#,
        Value::Boolean(true),
    );
}

#[test]
fn typed_null_class_reference_upcasts_to_implemented_interface() {
    let m = module(
        r#"
.function Main() -> Read<Int32>
.local Cell<Int32> empty
ldloca empty
initobj Cell<Int32>
ldloc empty
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::NullObjectReference(neoclr::assembler::parse_type("Read<Int32>").unwrap())
    );
}

#[test]
fn unrelated_or_wrong_generic_reference_cannot_be_implicitly_stored() {
    for ty in ["Other", "Cell<Boolean>"] {
        let m = module(&format!(
            ".function Main() -> Read<Int32>\n.local {ty} empty\nldloca empty\ninitobj {ty}\nldloc empty\nret\n.end"
        ));
        assert!(verify(&m).is_err());
        assert!(run(&m, Limits::default()).is_err());
        // The non-null unrelated class is rejected by the same conversion rule.
        if ty == "Other" {
            let m = module(".function Main() -> Read<Int32>\nldc.i4 1\nnewobj Other\nret\n.end");
            assert!(verify(&m).is_err());
            assert!(run(&m, Limits::default()).is_err());
        }
    }
}

#[test]
fn byref_slots_are_invariant_even_for_implementing_classes() {
    let m = module(
        r#"
.function Main() -> noresult
.local Cell<Int32> value
ldc.i4 1
newobj Cell<Int32>
stloc value
ldloca value
call Replace(Read<Int32>&)
ret
.end
.function Replace(Read<Int32>& slot) -> noresult
ldarg slot
ldc.i4 42
newobj Cell<Int32>
stobj Read<Int32>
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn downcast_still_requires_explicit_checked_cast() {
    let m = module(
        r#"
.function Main() -> Cell<Int32>
.local Read<Int32> view
ldc.i4 42
newobj Cell<Int32>
stloc view
ldloc view
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn implicitly_returned_interface_keeps_object_alive_across_gc() {
    let m = module(
        r#"
.function Main() -> Int32
.local Read<Int32> root
call Make()
stloc root
ldc.i4 0
newobj Other
pop
ldc.i4 0
newobj Other
pop
ldloc root
callvirt instance Read<Int32>::Get()
ret
.end
.function Make() -> Read<Int32>
ldc.i4 42
newobj Cell<Int32>
ret
.end
"#,
    );
    verify(&m).unwrap();
    let result = run(
        &m,
        Limits {
            heap_objects: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn type_tests_use_concrete_type_through_interface_views() {
    check(
        r#"
.function Main() -> Int32
.local Read<Int32> view
ldc.i4 42
newobj Cell<Int32>
stloc view
ldloc view
isinst Mutable<Int32>
callvirt instance Read<Int32>::Get()
ret
.end
"#,
        Value::Int32(42),
    );
}

#[test]
fn failed_type_tests_and_null_inputs_produce_typed_nulls() {
    for input in ["ldc.i4 1\nnewobj Other", "ldloc empty"] {
        check(
            &format!(
                r#"
.function Main() -> Boolean
.local Other empty
ldloca empty
initobj Other
{input}
isinst Read<Int32>
isinst Mutable<Int32>
ref.isnull
ret
.end
"#
            ),
            Value::Boolean(true),
        );
    }
}

#[test]
fn type_tests_reject_value_and_byref_operands() {
    for input in ["ldc.i4 1", "ldloca cell"] {
        let source = format!(
            r#"
.function Main() -> Boolean
.local Cell<Int32> cell
{input}
isinst Read<Int32>
ref.isnull
ret
.end
"#
        );
        assert!(verify(&module(&source)).is_err());
    }
}

#[test]
fn successful_type_test_preserves_object_identity() {
    check(
        r#"
.function Main() -> Boolean
ldc.i4 42
newobj Cell<Int32>
dup
isinst Read<Int32>
ref.eq
ret
.end
"#,
        Value::Boolean(true),
    );
}
