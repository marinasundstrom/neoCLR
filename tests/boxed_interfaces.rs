use neoclr::{Limits, Value, assemble, run, verify};

const TYPES: &str = r#"
.type class System.Object
.end
.interface Mutable
.method instance Add(Int32 amount) -> Int32
.end
.end
.type Counter
.implements Mutable
.field Count Int32
.method instance byref Add(Int32 amount) -> Int32
ldarg this
ldarg this
ldfld Counter::Count
ldarg amount
add
stfld Counter::Count
pop
ldarg this
ldfld Counter::Count
ret
.end
.end
"#;
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(".module Boxing\n.entry Main\n{TYPES}{body}")).unwrap()
}
#[test]
fn boxed_value_is_independent_but_interface_aliases_share_mutations() {
    let m = module(
        r#"
.function Main() -> Int32
.local Counter value
.local Mutable first
.local Mutable second
ldc.i4 10
newobj Counter
stloc value
ldloc value
box Counter
castclass Mutable
stloc first
ldloc first
stloc second
ldloc first
ldc.i4 2
callvirt instance Mutable::Add(Int32)
pop
ldloc second
ldc.i4 3
callvirt instance Mutable::Add(Int32)
ldloc value
ldfld Counter::Count
add
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(25));
}
#[test]
fn returned_box_outlives_frame() {
    let m = module(
        r#"
.function Main() -> Int32
call Make()
ldc.i4 2
callvirt instance Mutable::Add(Int32)
ret
.end
.function Make() -> Mutable
ldc.i4 40
newobj Counter
box Counter
castclass Mutable
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn invalid_boxing_operand_and_interface_cast_are_rejected() {
    let wrong = module(".function Main() -> System.Object\nldc.i4 1\nbox Counter\nret\n.end");
    assert!(verify(&wrong).is_err());
    assert!(run(&wrong, Limits::default()).is_err());
    let wrong =
        module(".function Main() -> Mutable\nldc.i4 1\nbox Int32\ncastclass Mutable\nret\n.end");
    assert!(run(&wrong, Limits::default()).is_err());
}
#[test]
fn box_is_rooted_across_gc_and_observes_heap_limit() {
    let m = module(
        r#"
.function Main() -> Int32
.local Mutable live
ldc.i4 40
newobj Counter
box Counter
castclass Mutable
stloc live
ldc.i4 0
box Int32
pop
ldc.i4 1
box Int32
pop
ldloc live
ldc.i4 2
callvirt instance Mutable::Add(Int32)
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
    assert!(
        run(
            &m,
            Limits {
                heap_objects: 1,
                ..Limits::default()
            }
        )
        .is_err()
    );
}
#[test]
fn object_cast_cannot_expose_box_as_unboxed_value() {
    let m = module(
        ".function Main() -> Counter\nldc.i4 1\nnewobj Counter\nbox Counter\ncastclass Counter\nret\n.end",
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}
#[test]
fn intrinsic_string_interface_dispatch_and_cast_back() {
    let library = assemble(".module System").unwrap();
    let m = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(
            r#"
.module Strings
.entry Main
.interface Equal
.method instance Equals(String other) -> Boolean
.end
.end
.type System.String
.implements Equal
.method instance readonly byref Equals(String other) -> Boolean
ldarg this
ldobj String
ldarg other
ceq
ret
.end
.end
.function Main() -> String
.local Equal value
ldstr "hello"
castclass Equal
stloc value
ldloc value
ldstr "hello"
callvirt instance Equal::Equals(String)
brfalse Failed
ldloc value
castclass String
ret
Failed:
fault "string comparison failed"
.end
"#,
        )],
        &library,
    )
    .unwrap()
    .remove(0);
    let program = neoclr::LoadedProgram::with_library(&m, &library).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("hello".into())
    );
}

#[test]
fn type_test_accepts_an_interface_implemented_by_a_boxed_value() {
    let m = module(
        r#"
.function Main() -> Int32
ldc.i4 40
newobj Counter
box Counter
isinst Mutable
ldc.i4 2
callvirt instance Mutable::Add(Int32)
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn intrinsic_string_type_test_preserves_string_representation() {
    let library = assemble(".module System").unwrap();
    let m = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(
            r#"
.module Strings
.entry Main
.interface Equal
.method instance Equals(String other) -> Boolean
.end
.end
.type System.String
.implements Equal
.method instance readonly byref Equals(String other) -> Boolean
ldarg this
ldobj String
ldarg other
ceq
ret
.end
.end
.function Main() -> String
.local Equal value
ldstr "hello"
isinst Equal
stloc value
ldloc value
ldstr "hello"
callvirt instance Equal::Equals(String)
brfalse Failed
ldloc value
isinst String
ret
Failed:
fault "string comparison failed"
.end
"#,
        )],
        &library,
    )
    .unwrap()
    .remove(0);
    let program = neoclr::LoadedProgram::with_library(&m, &library).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("hello".into())
    );
}

#[test]
fn boxed_value_type_test_recognizes_object_root() {
    let m = module(
        r#"
.function Main() -> Boolean
ldc.i4 40
newobj Counter
box Counter
isinst System.Object
ref.isnull
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::Boolean(false)
    );
}
