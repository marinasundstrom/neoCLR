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
.method instance .ctor(T value) -> noresult
ldarg this
ldarg value
stfld Cell<T>::Value
ret
.end
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
"#;
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module GenericClasses\n.entry Main\n{TYPES}{body}"
    ))
    .unwrap()
}

#[test]
fn closed_constructor_alias_and_inherited_interface_dispatch() {
    let m = module(
        r#"
.function Main() -> Int32
.local Cell<Int32> cell
.local Mutable<Int32> alias
ldc.i4 1
newobj instance Cell<Int32>::.ctor(Int32)
stloc cell
ldloc cell
castclass Mutable<Int32>
stloc alias
ldloc alias
ldc.i4 42
callvirt instance Mutable<Int32>::Set(Int32)
ldloc alias
callvirt instance Read<Int32>::Get()
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn closed_instantiations_remain_distinct() {
    let m = module(
        r#"
.function Main() -> Read<Boolean>
ldc.i4 42
newobj instance Cell<Int32>::.ctor(Int32)
castclass Read<Boolean>
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn wrong_closed_field_value_is_rejected() {
    let m = module(
        r#"
.function Main() -> noresult
ldc.i4 42
newobj instance Cell<Int32>::.ctor(Int32)
ldstr "wrong"
stfld Cell<Int32>::Value
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn nested_generic_reference_survives_return_and_collection() {
    let m = module(
        r#"
.function Main() -> Int32
.local Read<Cell<Int32>> root
call Make()
stloc root
ldc.i4 0
newobj instance Cell<Int32>::.ctor(Int32)
pop
ldc.i4 1
newobj instance Cell<Int32>::.ctor(Int32)
pop
ldloc root
callvirt instance Read<Cell<Int32>>::Get()
call instance Cell<Int32>::Get()
ret
.end
.function Make() -> Read<Cell<Int32>>
ldc.i4 42
newobj instance Cell<Int32>::.ctor(Int32)
newobj instance Cell<Cell<Int32>>::.ctor(Cell<Int32>)
castclass Read<Cell<Int32>>
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
fn void_payload_is_distinct_from_no_result_method() {
    let m = module(
        r#"
.function Main() -> Void
.local Mutable<Void> cell
ldvoid
newobj instance Cell<Void>::.ctor(Void)
castclass Mutable<Void>
stloc cell
ldloc cell
ldvoid
callvirt instance Mutable<Void>::Set(Void)
ldloc cell
callvirt instance Read<Void>::Get()
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Void);
}

#[test]
fn closed_class_default_is_null_and_native_layout_remains_unavailable() {
    let m = module(
        r#"
.function Main() -> Cell<Int32>
.local Cell<Int32> cell
ldloca cell
initobj Cell<Int32>
ldloc cell
ret
.end
"#,
    );
    verify(&m).unwrap();
    let execution = run(&m, Limits::default()).unwrap();
    let Value::NullObjectReference(ty) = execution.value else {
        panic!("expected typed null")
    };
    assert!(m.is_reference_type(&ty));
    assert!(neoclr::memory::layout(&m, &ty).is_err());
    assert_eq!(execution.heap.statistics().allocated_objects, 0);
}

#[test]
fn generic_class_arguments_enforce_existing_constraints() {
    use neoclr::metadata::{ConstraintKind, GenericConstraint};
    let mut m = module(
        r#"
.function Main() -> Cell<Void>
ldvoid
newobj instance Cell<Void>::.ctor(Void)
ret
.end
"#,
    );
    m.types
        .iter_mut()
        .find(|t| t.name == "Cell")
        .unwrap()
        .generic_constraints
        .push(GenericConstraint {
            parameter: 0,
            kind: ConstraintKind::NotVoid,
        });
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn closed_receiver_cannot_call_another_instantiation() {
    let m = module(
        r#"
.function Main() -> Boolean
ldc.i4 42
newobj instance Cell<Int32>::.ctor(Int32)
call instance Cell<Boolean>::Get()
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn constructor_keeps_existing_unsupported_default_fault() {
    let m = module(
        r#"
.function Main() -> Cell<String>
ldstr "value"
newobj instance Cell<String>::.ctor(String)
ret
.end
"#,
    );
    // String defaults are not admitted by the current nominal class constructor path.
    // A supplied constructor argument must not silently bypass that field policy.
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("default initialization")
    );
}

#[test]
fn callvirt_invokes_nonvirtual_class_methods_with_noresult_mutation() {
    let m = module(
        r#"
.function Main() -> Int32
.local Cell<Int32> cell
ldc.i4 1
newobj Cell<Int32>
stloc cell
ldloc cell
ldc.i4 42
callvirt instance Cell<Int32>::Set(Int32)
ldloc cell
callvirt instance Cell<Int32>::Get()
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn nonvirtual_class_callvirt_checks_null_before_entering_body() {
    let m = module(
        r#"
.type class Empty
.method instance Answer() -> Int32
ldc.i4 42
ret
.end
.end
.function Main() -> Int32
.local Empty empty
ldloca empty
initobj Empty
ldloc empty
callvirt instance Empty::Answer()
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("null class receiver")
    );
}

#[test]
fn nonvirtual_class_callvirt_rejects_wrong_closed_receiver() {
    let m = module(
        r#"
.function Main() -> Int32
ldc.bool false
newobj Cell<Boolean>
callvirt instance Cell<Int32>::Get()
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn ordinary_class_callvirt_does_not_accept_a_byref_to_the_reference_slot() {
    let m = module(
        r#"
.function Main() -> Int32
.local Cell<Int32> cell
ldc.i4 42
newobj Cell<Int32>
stloc cell
ldloca cell
callvirt instance Cell<Int32>::Get()
ret
.end
"#,
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}
