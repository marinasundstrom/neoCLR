use neoclr::{Limits, LoadedProgram, Module, Value, assemble};
use std::{process::Command, sync::OnceLock};
fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3").current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}

#[test]
fn handle_fields_must_be_assigned_by_class_constructors() {
    for body in ["", "ldarg this\nldfld Holder::Handle\npop\n"] {
        let text = format!(".module Probe\n.entry Main\n.type class Holder\n.field private Handle System.RuntimeTypeHandle\n.method instance .ctor() -> noresult\n{body}ret\n.end\n.end\n.function Main() -> Holder\nnewobj instance Holder::.ctor()\nret\n.end");
        let app = assemble(&text).unwrap();
        let program = LoadedProgram::with_library(&app, library()).unwrap();
        let error = program.run(Limits::default()).err().expect("uninitialized handle must fault");
        assert!(error.to_string().contains("uninitialized"), "{error}");
    }
}

#[test]
fn type_has_no_metadata_query_exports() {
    let text = ".module Probe\n.function Query(System.Type value) -> System.Reflection.FieldInfo[]\nldarg value\ncall instance System.Type::GetFields()\nret\n.end";
    let app = assemble(text).unwrap();
    let result = LoadedProgram::with_library(&app, library()).and_then(|p| p.verify());
    assert!(result.is_err());
}
#[test]
fn reflection_snapshots_are_managed_classes_with_base_views() {
    let app = assemble(
        r#"
.module Reflect
.entry Main
.type class Item
.field Count Int32
.end
.function Main() -> String
.local System.Reflection.FieldInfo field
ldtoken Item
call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)
call instance System.Type::get_Info()
call instance System.Reflection.TypeInfo::GetFields()
ldc.i4 0
ldelem System.Reflection.FieldInfo
stloc field
ldloc field
castclass System.Reflection.MemberInfo
call instance System.Reflection.MemberInfo::get_Name()
ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("Count".into())
    );
}

#[test]
fn returned_descriptor_keeps_nested_type_snapshot_alive_and_obeys_heap_limit() {
    let app = assemble(
        r#"
.module Reflect
.entry Main
.type class Item
.field Count Int32
.end
.function Main() -> System.Reflection.MemberInfo
ldtoken Item
call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)
call instance System.Type::get_Info()
call instance System.Reflection.TypeInfo::GetFields()
ldc.i4 0
ldelem System.Reflection.FieldInfo
castclass System.Reflection.MemberInfo
ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    let Value::ObjectReference(descriptor) = result.value else {
        panic!("expected descriptor class");
    };
    assert_eq!(
        descriptor.target(),
        &neoclr::metadata::Type::from_name("System.Reflection.MemberInfo")
    );
    let Some(Value::Object { fields, .. }) = result.heap.get(descriptor.allocation_id()) else {
        panic!("descriptor not rooted");
    };
    let Value::ObjectReference(declaring_type) = &fields[1] else {
        panic!("Type not projected as class");
    };
    assert!(result.heap.get(declaring_type.allocation_id()).is_some());
    assert!(
        program
            .run(Limits {
                heap_objects: 1,
                ..Limits::default()
            })
            .unwrap_err()
            .message
            .contains("heap object limit")
    );
}

#[test]
fn value_type_classification_uses_type_category_not_addressing_mode() {
    for (name, expected) in [
        ("Int32", true),
        ("Boolean", true),
        ("Void", true),
        ("String", false),
        ("System.Type", false),
        ("System.Reflection.BindingFlags", true),
        ("Item", false),
        ("Point", true),
        ("System.Collections.Iterable<Int32>", false),
        ("arrayref<Int32>", false),
        ("Int32&", false),
        ("Int32*", false),
        ("System.Option<Int32>", true),
    ] {
        let app = assemble(&format!(
            r#"
.module Categories
.entry Main
.type class Item
.end
.type Point
.field X Int32
.end
.function Main() -> Boolean
ldtoken {name}
call System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)
call instance System.Type::get_IsValueType()
ret
.end
"#
        ))
        .unwrap();
        let program = LoadedProgram::with_library(&app, library()).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(Limits::default()).unwrap().value,
            Value::Boolean(expected),
            "{name}"
        );
    }
}
