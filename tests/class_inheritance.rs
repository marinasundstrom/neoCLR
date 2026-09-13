use neoclr::{Limits, Value, assemble, run, verify};

const TYPES: &str = r#"
.type class abstract Base
.field Number Int32
.method instance Read() -> Int32
ldarg this
ldfld Base::Number
ret
.end
.end
.type class Derived
.extends Base
.field Extra Int32
.end
"#;
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(".module Classes\n.entry Main\n{TYPES}{body}")).unwrap()
}

#[test]
fn base_reference_shares_derived_allocation_and_dispatches_inherited_method() {
    let m = module(
        r#"
.function Main() -> Int32
.local Derived original
.local Base view
ldc.i4 7
ldc.i4 9
newobj Derived
stloc original
ldloc original
stloc view
ldloc view
ldc.i4 42
stfld Base::Number
ldloc original
callvirt instance Base::Read()
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn downcast_preserves_concrete_identity() {
    let m = module(
        r#"
.function Main() -> Boolean
.local Derived original
ldc.i4 7
ldc.i4 9
newobj Derived
stloc original
ldloc original
castclass Base
castclass Derived
ldloc original
ref.eq
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
fn abstract_class_cannot_be_allocated() {
    let source = format!(
        ".module Abstract\n.entry Main\n{TYPES}.function Main() -> Base\nldc.i4 0\nnewobj Base\nret\n.end"
    );
    match assemble(&source) {
        Err(_) => (),
        Ok(m) => {
            assert!(verify(&m).is_err());
            assert!(run(&m, Limits::default()).is_err());
        }
    }
}

#[test]
fn unrelated_cast_faults_and_implicit_downcast_is_rejected() {
    let extra = ".type class Other\n.end\n";
    let m = module(&format!(
        "{extra}.function Main() -> Other\nldc.i4 1\nldc.i4 2\nnewobj Derived\ncastclass Other\nret\n.end"
    ));
    verify(&m).unwrap();
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("cast")
    );
    let m = module(
        ".function Main() -> Derived\n.local Base view\nldloca view\ninitobj Base\nldloc view\nret\n.end",
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn null_upcast_is_valid_and_byref_slots_remain_invariant() {
    let m = module(
        ".function Main() -> Boolean\n.local Derived child\n.local Base parent\nldloca child\ninitobj Derived\nldloc child\nstloc parent\nldloc child\nldloc parent\nref.eq\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::Boolean(true)
    );
    let m = module(
        ".function Main() -> noresult\n.local Derived child\nldloca child\ninitobj Derived\nldloca child\ncall Replace(Base&)\nret\n.end\n.function Replace(Base& value) -> noresult\nret\n.end",
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn inherited_storage_categories_cannot_be_mixed() {
    for types in [
        ".type Base\n.end\n.type class Child\n.extends Base\n.end",
        ".type class Base\n.end\n.type Child\n.extends Base\n.end",
    ] {
        assert!(
            assemble(&format!(
                ".module Mixed\n.entry Main\n{types}\n.function Main() -> noresult\nret\n.end"
            ))
            .is_err()
        );
    }
}

#[test]
fn closed_generic_base_reference_preserves_fields_and_return_identity() {
    let m = assemble(
        r#"
.module Generic
.entry Main
.type class Parent<T>
.field Value T
.end
.type class Child<T>
.extends Parent<T>
.field Other Int32
.end
.function Make() -> Parent<String>
ldstr "kept"
ldc.i4 9
newobj Child<String>
ret
.end
.function Main() -> String
call Make()
castclass Child<String>
ldfld Child<String>::Value
ret
.end
"#,
    )
    .unwrap();
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::String("kept".into())
    );
}
