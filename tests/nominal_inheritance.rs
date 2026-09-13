use neoclr::{Limits, Value, assemble, run, verify};

const TYPES: &str = r#"
.interface Reader
.method instance Read() -> Int32
.end
.end
.type class abstract Base
.implements Reader
.field Age Int32
.method instance .ctor(Int32 age) -> void
ldarg 0
ldarg age
stfld Base::Age
ret
.end
.method instance abstract Read() -> Int32
.end
.method instance virtual Set(Int32 age) -> void
ldarg 0
ldarg age
stfld Base::Age
ret
.end
.end
.type class Derived
.extends Base
.field Extra Int32
.method instance .ctor(Int32 age) -> void
ldarg 0
ldarg age
call instance Base::.ctor(Int32)
ldarg 0
ldc.i4 1
stfld Derived::Extra
ret
.end
.method instance override Read() -> Int32
ldarg 0
ldfld Base::Age
ldarg 0
ldfld Derived::Extra
add
ret
.end
.method instance override Set(Int32 age) -> void
ldarg 0
ldarg age
call instance Base::Set(Int32)
ret
.end
.end
"#;
fn module(main: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n{TYPES}\n.function Main() -> Int32\n{main}\n.end"
    ))
    .unwrap()
}

#[test]
fn base_and_interface_dispatch_use_concrete_object_and_base_constructor_fields() {
    let m = module(
        r#"
.local Base parent
.local Reader reader
ldc.i4 7
newobj instance Derived::.ctor(Int32)
stloc parent
ldloc parent
stloc reader
ldloc parent
ldc.i4 41
callvirt instance Base::Set(Int32)
ldloc reader
callvirt instance Reader::Read()
ret
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn class_virtual_dispatch_and_nonvirtual_call_have_distinct_behavior() {
    let source = TYPES.replace(
        ".method instance abstract Read() -> Int32\n.end",
        ".method instance virtual Read() -> Int32\nldc.i4 100\nret\n.end",
    );
    let m = assemble(&format!(".module Test\n.entry Main\n{source}\n.function Main() -> Int32\n.local Base b\nldc.i4 7\nnewobj instance Derived::.ctor(Int32)\nstloc b\nldloc b\ncallvirt instance Base::Read()\nldloc b\ncall instance Base::Read()\nadd\nret\n.end")).unwrap();
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(108));
}

#[test]
fn abstract_construction_and_wrong_override_contract_are_rejected() {
    assert!(assemble(&format!(".module Test\n.entry Main\n{TYPES}\n.function Main() -> Base\nldc.i4 1\nnewobj instance Base::.ctor(Int32)\nret\n.end")).is_err());
    let source = TYPES.replace(
        ".method instance override Set(Int32 age) -> void",
        ".method instance override Set(Int32 age) -> Void",
    );
    let error = assemble(&format!(
        ".module Test\n.entry Main\n{source}\n.function Main() -> Int32\nldc.i4 0\nret\n.end"
    ))
    .unwrap_err();
    assert!(error.message.contains("exact inherited virtual contract"));
}

#[test]
fn repeated_and_foreign_receiver_constructor_calls_fault_without_verifier() {
    for chain in [
        "ldarg 0\nldarg age\ncall instance Base::.ctor(Int32)\nldarg 0\nldarg age\ncall instance Base::.ctor(Int32)",
        "ldc.i4 1\nnewobj instance Other::.ctor(Int32)\nldarg age\ncall instance Base::.ctor(Int32)",
    ] {
        let source = TYPES.replace(
            "ldarg 0\nldarg age\ncall instance Base::.ctor(Int32)",
            chain,
        );
        let m = assemble(&format!(".module Test\n.entry Main\n{source}\n.type class Other\n.extends Base\n.method instance .ctor(Int32 age) -> void\nldarg 0\nldarg age\ncall instance Base::.ctor(Int32)\nret\n.end\n.method instance override Read() -> Int32\nldc.i4 0\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 7\nnewobj instance Derived::.ctor(Int32)\npop\nldc.i4 0\nret\n.end")).unwrap();
        assert!(
            run(&m, Limits::default())
                .unwrap_err()
                .message
                .contains("own receiver")
        );
    }
}

#[test]
fn missing_base_chain_and_cyclic_this_delegation_fault() {
    let missing = TYPES.replace("ldarg 0\nldarg age\ncall instance Base::.ctor(Int32)", "");
    let m = assemble(&format!(".module Test\n.entry Main\n{missing}\n.function Main() -> Int32\nldc.i4 7\nnewobj instance Derived::.ctor(Int32)\npop\nldc.i4 0\nret\n.end")).unwrap();
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("must complete")
    );
    let m = assemble(".module Test\n.entry Main\n.type class C\n.method instance .ctor() -> void\nldarg 0\ncall instance C::.ctor()\nret\n.end\n.end\n.function Main() -> C\nnewobj instance C::.ctor()\nret\n.end").unwrap();
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("cyclic")
    );
}
