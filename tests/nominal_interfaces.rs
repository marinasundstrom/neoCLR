use neoclr::{Limits, Value, assemble, run, verify};

const TYPES: &str = r#"
.interface Read
.method instance Get() -> Int32
.end
.end
.interface Mutable
.implements Read
.method instance Set(Int32 value) -> noresult
.end
.end
.type class Counter
.implements Mutable
.field Age Int32
.method instance Get() -> Int32
ldarg this
ldfld Counter::Age
ret
.end
.method instance Set(Int32 value) -> noresult
ldarg this
ldarg value
stfld Counter::Age
ret
.end
.end
.type class Other
.field Age Int32
.end
"#;
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module NominalInterfaces\n.entry Main\n{TYPES}{body}"
    ))
    .unwrap()
}

#[test]
fn interface_alias_mutates_the_same_class_and_inherited_dispatch_works() {
    let m = module(
        r#"
.function Main() -> Int32
.local Counter counter
.local Mutable alias
ldc.i4 1
newobj Counter
stloc counter
ldloc counter
castclass Mutable
stloc alias
ldloc alias
ldc.i4 42
callvirt instance Mutable::Set(Int32)
ldloc alias
castclass Read
callvirt instance Read::Get()
ldloc counter
ldfld Counter::Age
add
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(84));
}

#[test]
fn returned_interface_retains_object_and_cast_back_preserves_identity() {
    let m = module(
        r#"
.function Main() -> Boolean
.local Read view
call Make()
stloc view
ldloc view
ldloc view
castclass Counter
ref.eq
ret
.end
.function Make() -> Read
ldc.i4 42
newobj Counter
castclass Read
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::Boolean(true)
    );
}

#[test]
fn incompatible_cast_faults_even_without_verifier() {
    let m = module(".function Main() -> Read\nldc.i4 1\nnewobj Other\ncastclass Read\nret\n.end");
    verify(&m).unwrap(); // A checked down/cross-cast is valid IL, but fails for this object.
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn null_interface_dispatch_faults_and_null_cast_is_preserved() {
    let m = module(
        ".function Main() -> Int32\n.local Read r\nldloca r\ninitobj Read\nldloc r\ncastclass Mutable\ncastclass Read\ncallvirt instance Read::Get()\nret\n.end",
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
fn missing_class_implementation_is_rejected_before_execution() {
    let mut m = module(".function Main() -> Int32\nldc.i4 0\nret\n.end");
    m.functions.retain(|f| !f.name.ends_with(".Set"));
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn interface_field_keeps_escaped_object_alive_across_collections() {
    let m = module(
        r#"
.type class Holder
.field View Read
.end
.function Make() -> Holder
ldc.i4 42
newobj Counter
castclass Read
newobj Holder
ret
.end
.function Main() -> Int32
.local Holder holder
.local Int32 i
call Make()
stloc holder
ldc.i4 0
stloc i
Loop:
ldc.i4 0
newobj Other
pop
ldloc i
ldc.i4 1
add
stloc i
ldloc i
ldc.i4 30
blt Loop
ldloc holder
ldfld Holder::View
callvirt instance Read::Get()
ret
.end
"#,
    );
    verify(&m).unwrap();
    let result = run(
        &m,
        Limits {
            heap_objects: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn interface_signature_distinguishes_no_result_from_inhabited_void() {
    let mut m = module(".function Main() -> Int32\nldc.i4 0\nret\n.end");
    let method = m
        .functions
        .iter_mut()
        .find(|f| f.name == "Counter.Set")
        .unwrap();
    method.no_result = false;
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn ordinary_interface_requires_an_object_not_an_owned_value() {
    let m = module(".function Main() -> Read\nldc.i4 1\nret\n.end");
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn concrete_and_derived_interface_receivers_call_inherited_contract_directly() {
    for receiver in ["", "castclass Mutable\n"] {
        let m = module(&format!(
            ".function Main() -> Int32\nldc.i4 42\nnewobj Counter\n{receiver}callvirt instance Read::Get()\nret\n.end"
        ));
        verify(&m).unwrap();
        assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[test]
fn wrong_nominal_receiver_cannot_dispatch_with_or_without_verification() {
    let m = module(
        ".function Main() -> Int32\nldc.i4 42\nnewobj Other\ncallvirt instance Read::Get()\nret\n.end",
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}
