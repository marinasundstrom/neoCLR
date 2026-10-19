use neoclr::{Limits, LoadedProgram, Value};

#[test]
fn implicit_object_slots_dispatch_to_reference_class_and_inherited_override() {
    let source = r#"
.module Overrides
.references (System)
.entry Main
.type class Root
.method instance .ctor() -> noresult
ret
.end
.method instance override ToString() -> String
ldstr "root"
ret
.end
.method instance override GetHashCode() -> Int32
ldc.i4 42
ret
.end
.method instance override Equals(System.Object other) -> Boolean
ldc.bool true
ret
.end
.end
.type class Leaf
.extends Root
.method instance .ctor() -> noresult
ldarg this
call instance Root::.ctor()
ret
.end
.end
.function Main() -> Int32
.local System.Object value
newobj instance Leaf::.ctor()
stloc value
ldloc value
ldloc value
callvirt instance System.Object::Equals(System.Object)
brfalse bad
ldloc value
callvirt instance System.Object::GetHashCode()
ret
bad:
ldc.i4 1
ret
.end
"#;
    let system = ".module System\n.type class abstract System.Object\n.method instance virtual Equals(System.Object other) -> Boolean\nldc.bool false\nret\n.end\n.method instance virtual GetHashCode() -> Int32\nldc.i4 0\nret\n.end\n.method instance virtual ToString() -> String\nldstr \"object\"\nret\n.end\n.end";
    let system = neoclr::assemble(system).unwrap();
    let modules =
        neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], &system)
            .unwrap();
    let program = LoadedProgram::with_library(&modules[0], &system).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let graph = program
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            1000,
        )
        .unwrap();
    for name in ["Root.Equals", "Root.GetHashCode"] {
        assert!(graph.functions.iter().any(|f| f.target.name == name));
    }
}

#[test]
fn intrinsic_string_object_override_uses_text_receiver_and_checks_null() {
    use neoclr::metadata::{Representation, Type};
    let mut seed = neoclr::assemble(
        r#"
.module System
.references ()
.type class abstract System.Object
.method instance virtual Equals(System.Object other) -> Boolean
ldc.bool false
ret
.end
.method instance virtual ToString() -> String
ldstr "object"
ret
.end
.method instance virtual GetHashCode() -> Int32
ldc.i4 0
ret
.end
.end
.type System.String
.method instance GetHashCode() -> Int32
ldc.i4 42
ret
.end
.end
"#,
    )
    .unwrap();
    let root = seed
        .types
        .iter()
        .find(|t| t.name == "System.Object")
        .unwrap()
        .definition
        .clone()
        .unwrap();
    let definition = seed
        .types
        .iter_mut()
        .find(|t| t.name == "System.String")
        .unwrap();
    definition.representation = Representation::Runtime;
    definition.is_reference_type = false;
    definition.is_sealed = true;
    definition.base = Some(Type::from_name("System.Object"));
    let method = seed
        .functions
        .iter_mut()
        .find(|f| f.owner == Some(Type::String))
        .unwrap();
    method.receiver_byref = false;
    method.is_virtual = true;
    method.is_override = true;
    for (load, expected) in [
        ("ldstr \"text\"", Some(42)),
        (
            ".local String empty\nldloca empty\ninitobj String\nldloc empty",
            None,
        ),
    ] {
        let source = format!(
            ".module App\n.entry Main\n.function Main() -> Int32\n{load}\ncallvirt instance System.String::GetHashCode()\nret\n.end"
        );
        let app = neoclr::assembler::read_modules_with_object_root(
            &[neoclr::assembler::ModuleInput::Source(&source)],
            &seed,
            &root,
        )
        .unwrap()
        .remove(0);
        let program = LoadedProgram::with_modules_and_object_root(&app, &seed, &[], &root).unwrap();
        program.verify().unwrap();
        match expected {
            Some(value) => assert_eq!(
                program.run(Limits::default()).unwrap().value,
                Value::Int32(value)
            ),
            None => assert_eq!(
                program.run(Limits::default()).unwrap_err().code,
                neoclr::FaultCode::NullReference
            ),
        }
    }
    let source = ".module BoxedText\n.entry Main\n.function Main() -> Int32\nldstr \"text\"\ncastclass System.Object\ncallvirt instance System.Object::GetHashCode()\nret\n.end";
    let app = neoclr::assembler::read_modules_with_object_root(
        &[neoclr::assembler::ModuleInput::Source(source)],
        &seed,
        &root,
    )
    .unwrap()
    .remove(0);
    let program = LoadedProgram::with_modules_and_object_root(&app, &seed, &[], &root).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let app = neoclr::assemble(".module Empty").unwrap();
    let method = seed
        .functions
        .iter_mut()
        .find(|f| f.owner == Some(Type::String))
        .unwrap();
    method.name = "System.String.NotAnObjectSlot".into();
    assert!(LoadedProgram::with_modules_and_object_root(&app, &seed, &[], &root).is_err());
}
