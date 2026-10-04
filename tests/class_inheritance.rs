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

#[test]
fn three_binary_assemblies_preserve_base_construction_dispatch_and_identity() {
    let base = r#"
.module Models
.revision r1
.type class Base
.field Number Int32
.method instance .ctor(Int32 number) -> noresult
ldarg this
ldarg number
stfld Base::Number
ret
.end
.method instance virtual Read() -> Int32
ldarg this
ldfld Base::Number
ret
.end
.end
"#;
    let derived = r#"
.module Implementation
.revision r1
.references (Models#r1)
.type class Derived
.extends Base
.field Extra Int32
.method instance .ctor(Int32 number,Int32 extra) -> noresult
ldarg this
ldarg number
call instance Base::.ctor(Int32)
ldarg this
ldarg extra
stfld Derived::Extra
ret
.end
.method instance override Read() -> Int32
ldarg this
ldfld Base::Number
ldarg this
ldfld Derived::Extra
add
ret
.end
.end
"#;
    let app = r#"
.module Consumer
.references (Models#r1,Implementation#r1)
.entry Main
.function Main() -> Int32
.local Derived original
.local Base view
.local Int32 first
ldc.i4 40
ldc.i4 2
newobj instance Derived::.ctor(Int32,Int32)
stloc original
ldloc original
stloc view
ldloc view
callvirt instance Base::Read()
stloc first
ldloc view
ldc.i4 41
stfld Base::Number
ldloc original
callvirt instance Base::Read()
ldc.i4 43
ceq
brfalse Bad
ldloc first
ldc.i4 42
ceq
brfalse Bad
ldloc view
castclass Derived
ldloc original
ref.eq
brfalse Bad
ldc.i4 42
ret
Bad:
ldc.i4 0
ret
.end
"#;
    let modules = neoclr::assembler::assemble_modules(&[app, base, derived]).unwrap();
    let decoded: Vec<_> = modules
        .iter()
        .map(|module| {
            let image = neoclr::metadata_container::write_module(module).unwrap();
            assert_eq!(&image[..4], b"NEOX");
            neoclr::metadata_container::decode_envelope(&image).unwrap()
        })
        .collect();
    let program = neoclr::LoadedProgram::with_modules(
        &decoded[0],
        neoclr::library::system().unwrap(),
        &decoded[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert!(
        neoclr::LoadedProgram::with_modules(
            &decoded[0],
            neoclr::library::system().unwrap(),
            &decoded[2..],
        )
        .is_err()
    );
}
