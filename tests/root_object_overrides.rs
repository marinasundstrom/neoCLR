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
