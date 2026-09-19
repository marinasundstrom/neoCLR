use neoclr::{Limits, LoadedProgram, Module, Value};
use std::{process::Command, sync::OnceLock};
fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3").current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        neoclr::assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}

// The selected Raven profile has interface contracts; the bundled legacy profile
// still has value descriptors, so validate callers against the selected library.
fn assemble(source: &str) -> Result<Module, neoclr::Fault> {
    neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], library())
        .map(|mut modules| modules.remove(0))
}

#[test]
fn handle_fields_must_be_assigned_by_class_constructors() {
    for body in ["", "ldarg this\nldfld Holder::Handle\npop\n"] {
        let text = format!(
            ".module Probe\n.entry Main\n.type class Holder\n.field private Handle System.RuntimeTypeHandle\n.method instance .ctor() -> noresult\n{body}ret\n.end\n.end\n.function Main() -> Holder\nnewobj instance Holder::.ctor()\nret\n.end"
        );
        let app = assemble(&text).unwrap();
        let program = LoadedProgram::with_library(&app, library()).unwrap();
        let error = program
            .run(Limits::default())
            .err()
            .expect("uninitialized handle must fault");
        assert!(error.to_string().contains("uninitialized"), "{error}");
    }
}

#[test]
fn type_has_no_metadata_query_exports() {
    let text = ".module Probe\n.function Query(System.Type value) -> System.Introspection.FieldInfo[]\nldarg value\ncallvirt instance System.Introspection.TypeInfo::GetFields()\nret\n.end";
    let result = assemble(text)
        .and_then(|app| LoadedProgram::with_library(&app, library()))
        .and_then(|p| p.verify());
    assert!(result.is_err());
}
#[test]
fn reflection_snapshots_implement_public_info_interfaces() {
    let app = assemble(
        r#"
.module Reflect
.entry Main
.type class Item
.field Count Int32
.end
.function Main() -> String
.local System.Introspection.FieldInfo field
call System.Runtime.RuntimeContext::get_Current()
ldtoken Item
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
callvirt instance System.Introspection.TypeInfo::GetFields()
ldc.i4 0
ldelem System.Introspection.FieldInfo
stloc field
ldloc field
castclass System.Introspection.MemberInfo
callvirt instance System.Introspection.MemberInfo::get_Name()
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
.function Main() -> System.Introspection.MemberInfo
call System.Runtime.RuntimeContext::get_Current()
ldtoken Item
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
callvirt instance System.Introspection.TypeInfo::GetFields()
ldc.i4 0
ldelem System.Introspection.FieldInfo
castclass System.Introspection.MemberInfo
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
        descriptor.concrete_type(),
        neoclr::metadata::Type::from_name("System.Introspection.RuntimeFieldInfo")
    );
    assert_eq!(
        descriptor.target(),
        &neoclr::metadata::Type::from_name("System.Introspection.MemberInfo")
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
        ("System.Introspection.TypeInfo", false),
        ("System.Introspection.BindingFlags", true),
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
call System.Runtime.RuntimeContext::get_Current()
ldtoken {name}
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
callvirt instance System.Introspection.TypeInfo::get_IsValueType()
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

#[test]
fn descriptor_parameter_arrays_are_independent_copies() {
    let app = assemble(
        r#"
.module SnapshotCopies
.entry Main
.type class Item
.method static Compute(Int32 first,Int32 second) -> Int32
ldarg first
ret
.end
.end
.function Main() -> Int32
.local System.Introspection.MethodInfo method
.local arrayref<System.Introspection.ParameterInfo> parameters
call System.Runtime.RuntimeContext::get_Current()
ldtoken Item
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
callvirt instance System.Introspection.TypeInfo::GetMethods()
ldc.i4 0
ldelem System.Introspection.MethodInfo
stloc method
ldloc method
callvirt instance System.Introspection.MethodInfo::GetParameters()
stloc parameters
ldloc parameters
ldc.i4 0
ldloc parameters
ldc.i4 1
ldelem System.Introspection.ParameterInfo
stelem System.Introspection.ParameterInfo
ldloc method
callvirt instance System.Introspection.MethodInfo::GetParameters()
ldc.i4 0
ldelem System.Introspection.ParameterInfo
callvirt instance System.Introspection.ParameterInfo::get_Position()
ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(0)
    );
}

#[test]
fn descriptor_private_accessor_requires_explicit_inclusion() {
    for (argument, predicate) in [
        ("ldc.bool false", "get_IsNone"),
        ("ldc.bool true", "get_IsSome"),
    ] {
        let app = assemble(&format!(
            r#"
.module Accessors
.entry Main
.type class Item
.property instance Hidden() -> Int32
.get instance Item::get_Hidden()
.end
.method private instance get_Hidden() -> Int32
ldc.i4 42
ret
.end
.end
.function Main() -> Boolean
call System.Runtime.RuntimeContext::get_Current()
ldtoken Item
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
ldc.i4 36
call System.Introspection.BindingFlags::FromValue(Int32)
callvirt instance System.Introspection.TypeInfo::GetProperties(System.Introspection.BindingFlags)
ldc.i4 0
ldelem System.Introspection.PropertyInfo
{argument}
callvirt instance System.Introspection.PropertyInfo::GetGetMethod(Boolean)
call instance System.Option<System.Introspection.MethodInfo>::{predicate}()
ret
.end
"#
        ))
        .unwrap();
        let program = LoadedProgram::with_library(&app, library()).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(Limits::default()).unwrap().value,
            Value::Boolean(true)
        );
    }
}

#[test]
fn object_get_type_preserves_concrete_type_through_reference_views() {
    for (body, expected) in [
        ("ldstr \"hello\"\ncastclass System.Object", "System.String"),
        (
            "ldc.i4 42\nbox Int32\ncastclass System.Object",
            "System.Int32",
        ),
        (
            "ldc.i4 0\nnewarr Int32\ncastclass System.Object",
            "arrayref<System.Int32>",
        ),
    ] {
        let app = assemble(&format!(
            ".module GetType\n.entry Main\n.function Main() -> String\n{body}\ncall instance System.Object::GetType()\ncallvirt instance System.Introspection.TypeInfo::get_FullName()\nret\n.end"
        )).unwrap();
        let program = LoadedProgram::with_library(&app, library()).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(Limits::default()).unwrap().value,
            Value::String(expected.into())
        );
    }
}

#[test]
fn object_get_type_rejects_null() {
    let app = assemble(".module GetType\n.entry Main\n.function Main() -> System.Introspection.TypeInfo\n.local System.Object value\nldloca value\ninitobj System.Object\nldloc value\ncall instance System.Object::GetType()\nret\n.end").unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    assert!(program.run(Limits::default()).is_err());
}

#[test]
fn object_get_type_reports_derived_allocation_through_base_and_interface() {
    let app = assemble(
        r#"
.module GetType
.entry Main
.interface Tag
.end
.type class Base
.end
.type class Derived
.extends Base
.implements Tag
.end
.function Main() -> String
.local Base base
.local Tag tag
newobj Derived
stloc base
ldloc base
castclass Tag
stloc tag
ldloc tag
castclass System.Object
call instance System.Object::GetType()
callvirt instance System.Introspection.TypeInfo::get_FullName()
ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("Derived".into())
    );
}

#[test]
fn direct_native_type_result_preserves_public_interface_view() {
    let app = assemble(
        r#"
.module EnumInfo
.entry Main
.function Main() -> String
call System.Runtime.RuntimeContext::get_Current()
ldtoken System.Introspection.BindingFlags
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
callvirt instance System.Introspection.TypeInfo::GetEnumUnderlyingType()
callvirt instance System.Introspection.TypeInfo::get_FullName()
ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("System.Int32".into())
    );
}
