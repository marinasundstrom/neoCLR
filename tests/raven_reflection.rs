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
            .expect_err("uninitialized handle must fault");
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
callvirt instance System.Collections.Sequence<System.Introspection.FieldInfo>::get_Item(Int32)
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
callvirt instance System.Collections.Sequence<System.Introspection.FieldInfo>::get_Item(Int32)
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
fn descriptor_parameter_sequences_do_not_expose_runtime_owned_storage() {
    // The public Sequence cannot mutate. An explicit backing-array cast may only
    // change this returned copy, never the runtime-owned parameter metadata.
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
callvirt instance System.Collections.Sequence<System.Introspection.MethodInfo>::get_Item(Int32)
stloc method
ldloc method
callvirt instance System.Introspection.MethodInfo::GetParameters()
castclass arrayref<System.Introspection.ParameterInfo>
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
callvirt instance System.Collections.Sequence<System.Introspection.ParameterInfo>::get_Item(Int32)
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
callvirt instance System.Collections.Sequence<System.Introspection.PropertyInfo>::get_Item(Int32)
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

fn run_introspection(source: &str) -> Value {
    let app = assemble(source).unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap().value
}

#[test]
fn executing_assembly_follows_source_caller_not_entry_or_runtime_facade() {
    assert_eq!(
        run_introspection(
            r#"
.module Imported
.assembly {"name":"Entry","full_name":"Entry","modules":["Entry.dll"],"references":["Library","System.Runtime"]}
.assembly {"name":"Library","full_name":"Library","modules":["Library.dll"],"references":["System.Runtime"]}
.entry Main
.function Main() -> String
.origin {"assembly":"Entry","module":"Entry.dll","name":"Main","token":100663297}
call LibraryContext()
ret
.end
.function LibraryContext() -> String
.origin {"assembly":"Library","module":"Library.dll","name":"LibraryContext","token":100663297}
call System.Runtime.RuntimeContext::get_Current()
call instance System.Runtime.RuntimeContext::get_ExecutingAssembly()
callvirt instance System.Introspection.AssemblyInfo::get_Name()
ret
.end
"#
        ),
        Value::String("Library".into())
    );
}

#[test]
fn executing_assembly_exposes_system_runtime_through_sequence() {
    assert_eq!(
        run_introspection(
            r#"
.module App
.entry Main
.function Main() -> String
call System.Runtime.RuntimeContext::get_Current()
call instance System.Runtime.RuntimeContext::get_ExecutingAssembly()
callvirt instance System.Introspection.AssemblyInfo::get_ReferencedAssemblies()
ldc.i4 0
callvirt instance System.Collections.Sequence<System.Introspection.AssemblyInfo>::get_Item(Int32)
callvirt instance System.Introspection.AssemblyInfo::get_Name()
ret
.end
"#
        ),
        Value::String("System.Runtime".into())
    );
}

#[test]
fn source_tokens_reach_type_member_and_parameter_interfaces() {
    let prefix = r#"
.module Imported
.assembly {"name":"App","full_name":"App","modules":["App.dll"],"references":["System.Runtime"]}
.entry Main
.type class Item
.origin {"assembly":"App","module":"App.dll","name":"Example.Item","token":33554434,"field_tokens":[67108870]}
.field Count Int32
.method static Echo(Int32 value) -> Int32
.origin {"assembly":"App","module":"App.dll","name":"Echo","token":100663303,"parameter_tokens":[134217737]}
ldarg value
ret
.end
.end
.function Main() -> Int32
call System.Runtime.RuntimeContext::get_Current()
ldtoken Item
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
"#;
    for (query, expected) in [
        (
            "castclass System.Introspection.MemberInfo\ncallvirt instance System.Introspection.MemberInfo::get_MetadataToken()",
            0x02000002,
        ),
        (
            "callvirt instance System.Introspection.TypeInfo::GetFields()\nldc.i4 0\ncallvirt instance System.Collections.Sequence<System.Introspection.FieldInfo>::get_Item(Int32)\ncastclass System.Introspection.MemberInfo\ncallvirt instance System.Introspection.MemberInfo::get_MetadataToken()",
            0x04000006,
        ),
        (
            "callvirt instance System.Introspection.TypeInfo::GetMethods()\nldc.i4 0\ncallvirt instance System.Collections.Sequence<System.Introspection.MethodInfo>::get_Item(Int32)\ncastclass System.Introspection.MemberInfo\ncallvirt instance System.Introspection.MemberInfo::get_MetadataToken()",
            0x06000007,
        ),
        (
            "callvirt instance System.Introspection.TypeInfo::GetMethods()\nldc.i4 0\ncallvirt instance System.Collections.Sequence<System.Introspection.MethodInfo>::get_Item(Int32)\ncallvirt instance System.Introspection.MethodInfo::GetParameters()\nldc.i4 0\ncallvirt instance System.Collections.Sequence<System.Introspection.ParameterInfo>::get_Item(Int32)\ncallvirt instance System.Introspection.ParameterInfo::get_MetadataToken()",
            0x08000009,
        ),
    ] {
        assert_eq!(
            run_introspection(&format!("{prefix}{query}\nret\n.end")),
            Value::Int32(expected)
        );
    }
    let query = "callvirt instance System.Introspection.TypeInfo::GetMethods()\nldc.i4 0\ncallvirt instance System.Collections.Sequence<System.Introspection.MethodInfo>::get_Item(Int32)\ncallvirt instance System.Introspection.MethodInfo::GetParameters()\nldc.i4 0\ncallvirt instance System.Collections.Sequence<System.Introspection.ParameterInfo>::get_Item(Int32)\ncallvirt instance System.Introspection.ParameterInfo::get_Module()\ncallvirt instance System.Introspection.ModuleInfo::get_Name()";
    assert_eq!(
        run_introspection(&format!(
            "{}{query}\nret\n.end",
            prefix.replace("Main() -> Int32", "Main() -> String")
        )),
        Value::String("App.dll".into())
    );
}

#[test]
fn module_inventory_contains_open_generic_definitions() {
    assert_eq!(
        run_introspection(
            r#"
.module Catalog
.entry Main
.type Box<T>
.field Value !0
.end
.function Main() -> Int32
call System.Runtime.RuntimeContext::get_Current()
call instance System.Runtime.RuntimeContext::get_ExecutingAssembly()
callvirt instance System.Introspection.AssemblyInfo::GetModules()
ldc.i4 0
callvirt instance System.Collections.Sequence<System.Introspection.ModuleInfo>::get_Item(Int32)
callvirt instance System.Introspection.ModuleInfo::GetTypes()
ldc.i4 0
callvirt instance System.Collections.Sequence<System.Introspection.TypeInfo>::get_Item(Int32)
callvirt instance System.Introspection.TypeInfo::get_GenericArgumentCount()
ret
.end
"#
        ),
        Value::Int32(1)
    );
}

#[test]
fn discovery_limits_and_unloaded_references_fail_without_loading() {
    let query = r#"
.module App
.assembly {"name":"App","full_name":"App","modules":["App"],"references":["Missing"]}
.entry Main
.function Main() -> System.Collections.Sequence<System.Introspection.AssemblyInfo>
call System.Runtime.RuntimeContext::get_Current()
call instance System.Runtime.RuntimeContext::get_ExecutingAssembly()
callvirt instance System.Introspection.AssemblyInfo::get_ReferencedAssemblies()
ret
.end
"#;
    let app = assemble(query).unwrap();
    let p = LoadedProgram::with_library(&app, library()).unwrap();
    p.verify().unwrap();
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .to_string()
            .contains("assembly metadata is not loaded: Missing")
    );
    let query = query.replace("Missing", "System.Runtime").replace(
        "callvirt instance System.Introspection.AssemblyInfo::get_ReferencedAssemblies()",
        "callvirt instance System.Introspection.AssemblyInfo::get_ReferencedAssemblies()\nldc.i4 0\ncallvirt instance System.Collections.Sequence<System.Introspection.AssemblyInfo>::get_Item(Int32)\ncallvirt instance System.Introspection.AssemblyInfo::GetTypes()"
    ).replace("Main() -> System.Collections.Sequence<System.Introspection.AssemblyInfo>", "Main() -> System.Collections.Sequence<System.Introspection.TypeInfo>");
    let app = assemble(&query).unwrap();
    let p = LoadedProgram::with_library(&app, library()).unwrap();
    p.verify().unwrap();
    let limits = Limits {
        array_elements: 4,
        ..Limits::default()
    };
    let fault = p.run(limits).unwrap_err().to_string();
    assert!(
        fault.contains("budget") || fault.contains("limit"),
        "{fault}"
    );
}

#[test]
fn constructed_types_reuse_definition_tokens_and_arrays_have_no_definition_token() {
    for (name, expected) in [
        ("Box<Int32>", 0x02000001),
        ("Box<String>", 0x02000001),
        ("Int32[]", 0),
    ] {
        assert_eq!(
            run_introspection(&format!(
                r#"
.module TokenShapes
.entry Main
.type Box<T>
.field Value !0
.end
.function Main() -> Int32
call System.Runtime.RuntimeContext::get_Current()
ldtoken {name}
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
castclass System.Introspection.MemberInfo
callvirt instance System.Introspection.MemberInfo::get_MetadataToken()
ret
.end
"#
            )),
            Value::Int32(expected)
        );
    }
}

#[test]
fn property_tokens_are_definition_rows_scoped_to_the_declaring_module() {
    let query = r#"
.module PropertyTokens
.entry Main
.function Main() -> Int32
call System.Runtime.RuntimeContext::get_Current()
ldtoken System.Date
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
callvirt instance System.Introspection.TypeInfo::GetProperties()
ldc.i4 0
callvirt instance System.Collections.Sequence<System.Introspection.PropertyInfo>::get_Item(Int32)
castclass System.Introspection.MemberInfo
callvirt instance System.Introspection.MemberInfo::get_MetadataToken()
ret
.end
"#;
    let Value::Int32(token) = run_introspection(query) else {
        panic!("expected token")
    };
    assert_eq!(token >> 24, 0x17);
    assert_ne!(token & 0x00ff_ffff, 0);
    let query = query.replace("Main() -> Int32", "Main() -> String").replace(
        "callvirt instance System.Introspection.MemberInfo::get_MetadataToken()",
        "callvirt instance System.Introspection.MemberInfo::get_Module()\ncallvirt instance System.Introspection.ModuleInfo::get_Assembly()\ncallvirt instance System.Introspection.AssemblyInfo::get_Name()"
    );
    assert_eq!(
        run_introspection(&query),
        Value::String("System.Runtime".into())
    );
}

#[test]
fn assembly_module_object_identity_preserves_catalog_scope() {
    // Both assemblies deliberately share short names, module names and tokens.
    // The first also has a second module to check the other half of the key.
    assert_eq!(
        run_introspection(
            r#"
.module ScopedDescriptors
.assembly {"name":"Shared","full_name":"Shared, Version=1","modules":["Shared.dll","Other.netmodule"],"references":["Shared, Version=2","System.Runtime"]}
.assembly {"name":"Shared","full_name":"Shared, Version=2","modules":["Shared.dll"],"references":["System.Runtime"]}
.entry Main
.function First() -> System.Introspection.AssemblyInfo
.origin {"assembly":"Shared, Version=1","module":"Shared.dll","name":"First","token":100663297}
call System.Runtime.RuntimeContext::get_Current()
call instance System.Runtime.RuntimeContext::get_ExecutingAssembly()
ret
.end
.function Second() -> System.Introspection.AssemblyInfo
.origin {"assembly":"Shared, Version=2","module":"Shared.dll","name":"Second","token":100663297}
call System.Runtime.RuntimeContext::get_Current()
call instance System.Runtime.RuntimeContext::get_ExecutingAssembly()
ret
.end
.function ModuleAt(System.Introspection.AssemblyInfo assembly, Int32 index) -> System.Object
ldarg assembly
callvirt instance System.Introspection.AssemblyInfo::GetModules()
ldarg index
callvirt instance System.Collections.Sequence<System.Introspection.ModuleInfo>::get_Item(Int32)
castclass System.Object
ret
.end
.function Main() -> Boolean
.origin {"assembly":"Shared, Version=1","module":"Shared.dll","name":"Main","token":100663298}
call First()
castclass System.Object
call Second()
castclass System.Object
callvirt instance System.Object::Equals(System.Object)
brtrue failed
call First()
castclass System.Object
call First()
castclass System.Object
callvirt instance System.Object::Equals(System.Object)
brfalse failed
call First()
ldc.i4 0
call ModuleAt(System.Introspection.AssemblyInfo, Int32)
call Second()
ldc.i4 0
call ModuleAt(System.Introspection.AssemblyInfo, Int32)
callvirt instance System.Object::Equals(System.Object)
brtrue failed
call First()
ldc.i4 0
call ModuleAt(System.Introspection.AssemblyInfo, Int32)
call First()
ldc.i4 1
call ModuleAt(System.Introspection.AssemblyInfo, Int32)
callvirt instance System.Object::Equals(System.Object)
brtrue failed
call First()
ldc.i4 0
call ModuleAt(System.Introspection.AssemblyInfo, Int32)
call First()
ldc.i4 0
call ModuleAt(System.Introspection.AssemblyInfo, Int32)
callvirt instance System.Object::Equals(System.Object)
ret
failed:
ldc.bool false
ret
.end
"#
        ),
        Value::Boolean(true)
    );
}

#[test]
fn member_object_identity_includes_kind_closed_owner_and_definition() {
    let mut checks = String::new();
    for (kind, query) in [
        ("Field", "GetFields"),
        ("Method", "GetMethods"),
        ("Property", "GetProperties"),
    ] {
        let member = |owner: &str, index| {
            format!(
                "call System.Runtime.RuntimeContext::get_Current()\nldtoken {owner}\ncall instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)\ncallvirt instance System.Introspection.TypeInfo::{query}()\nldc.i4 {index}\ncallvirt instance System.Collections.Sequence<System.Introspection.{kind}Info>::get_Item(Int32)\ncastclass System.Object\n"
            )
        };
        for (other_owner, other_index, branch) in [
            ("Box<Int32>", 0, "brfalse"),
            ("Box<Int32>", 1, "brtrue"),
            ("Box<String>", 0, "brtrue"),
            ("Other<Int32>", 0, "brtrue"),
        ] {
            checks.push_str(&member("Box<Int32>", 0));
            checks.push_str(&member(other_owner, other_index));
            checks.push_str(&format!(
                "callvirt instance System.Object::Equals(System.Object)\n{branch} failed\n"
            ));
        }
    }
    let mut types = String::new();
    for name in ["Box", "Other"] {
        types.push_str(&format!(
            r#"
.type class {name}<T>
.field First !0
.field Second !0
.method instance FirstMethod() -> Int32
ldc.i4 1
ret
.end
.method instance SecondMethod() -> Int32
ldc.i4 2
ret
.end
.property instance FirstProperty() -> Int32
.get instance {name}<!0>::FirstMethod()
.end
.property instance SecondProperty() -> Int32
.get instance {name}<!0>::SecondMethod()
.end
.end
"#
        ));
    }
    assert_eq!(
        run_introspection(&format!(
            ".module MemberIdentity\n.entry Main\n{types}\n.function Main() -> Boolean\n{checks}ldc.bool true\nret\nfailed:\nldc.bool false\nret\n.end\n"
        )),
        Value::Boolean(true)
    );
}

#[test]
fn member_queries_currently_enumerate_declarations_only() {
    assert_eq!(
        run_introspection(
            r#"
.module DeclaredMembers
.entry Main
.type class Parent
.field ParentField Int32
.end
.type class Child
.extends Parent
.field ChildField Int32
.end
.function Main() -> Int32
call System.Runtime.RuntimeContext::get_Current()
ldtoken Child
call instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
callvirt instance System.Introspection.TypeInfo::GetFields()
castclass System.Collections.Collection<System.Introspection.FieldInfo>
callvirt instance System.Collections.Collection<System.Introspection.FieldInfo>::get_Count()
ret
.end
"#
        ),
        Value::Int32(1)
    );
}

#[test]
fn parameter_identity_uses_owner_kind_closed_type_definition_and_position() {
    let parameter = |owner: &str, property: bool, member_index, position| {
        let (kind, query, params) = if property {
            ("Property", "GetProperties", "GetIndexParameters")
        } else {
            ("Method", "GetMethods", "GetParameters")
        };
        format!(
            "call System.Runtime.RuntimeContext::get_Current()\nldtoken {owner}\ncall instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)\ncallvirt instance System.Introspection.TypeInfo::{query}()\nldc.i4 {member_index}\ncallvirt instance System.Collections.Sequence<System.Introspection.{kind}Info>::get_Item(Int32)\ncallvirt instance System.Introspection.{kind}Info::{params}()\nldc.i4 {position}\ncallvirt instance System.Collections.Sequence<System.Introspection.ParameterInfo>::get_Item(Int32)\ncastclass System.Object\n"
        )
    };
    let mut checks = String::new();
    for property in [false, true] {
        for (owner, other_kind, member, position, branch) in [
            ("Box<Int32>", property, 0, 0, "brfalse"),
            ("Box<Int32>", property, 0, 1, "brtrue"),
            ("Box<Int32>", property, 1, 0, "brtrue"),
            ("Box<String>", property, 0, 0, "brtrue"),
            ("Other<Int32>", property, 0, 0, "brtrue"),
            ("Box<Int32>", !property, 0, 0, "brtrue"),
        ] {
            checks.push_str(&parameter("Box<Int32>", property, 0, 0));
            checks.push_str(&parameter(owner, other_kind, member, position));
            checks.push_str(&format!(
                "callvirt instance System.Object::Equals(System.Object)\n{branch} failed\n"
            ));
        }
    }
    let mut types = String::new();
    for (i, name) in ["Box", "Other"].iter().enumerate() {
        // Every parameter has token zero. Properties share an accessor, so its
        // token, name and types cannot accidentally become property identity.
        types.push_str(&format!(r#"
.type class {name}<T>
.origin {{"assembly":"App","module":"App.dll","name":"{name}","token":{type_token},"property_tokens":[{prop1},{prop2}]}}
.method static Read(Int32 value, Int32 other) -> Int32
.origin {{"assembly":"App","module":"App.dll","name":"Read","token":{method1},"parameter_tokens":[0,0]}}
ldarg value
ret
.end
.method static Another(Int32 value, Int32 other) -> Int32
.origin {{"assembly":"App","module":"App.dll","name":"Another","token":{method2},"parameter_tokens":[0,0]}}
ldarg value
ret
.end
.property static First(Int32, Int32) -> Int32
.get {name}<!0>::Read(Int32, Int32)
.end
.property static Second(Int32, Int32) -> Int32
.get {name}<!0>::Read(Int32, Int32)
.end
.end
"#, type_token = 0x02000001+i, prop1=0x17000001+2*i, prop2=0x17000002+2*i, method1=0x06000001+2*i, method2=0x06000002+2*i));
    }
    assert_eq!(
        run_introspection(&format!(
            ".module Parameters\n.assembly {{\"name\":\"App\",\"full_name\":\"App\",\"modules\":[\"App.dll\"],\"references\":[\"System.Runtime\"]}}\n.entry Main\n{types}\n.function Main() -> Boolean\n{checks}ldc.bool true\nret\nfailed:\nldc.bool false\nret\n.end\n"
        )),
        Value::Boolean(true)
    );
}

#[test]
fn type_classification_distinguishes_open_closed_leaf_and_nominal_union() {
    let definitions = r#"
.type class Open<T>
.end
.type class abstract Abstract
.end
.type class Leaf
.sealed
.end
.interface Closed
.closedhierarchy
.end
.interface Contract
.end
.type Choice<T>
.custom instance System.Runtime.CompilerServices.UnionAttribute::.ctor()
.end
.type Number
.field private Stored Int32
.enum Int32
.literal Zero 0
.end
"#;
    let mut body = String::new();
    let mut check = 0;
    // Columns are IsAbstract, IsOpen, IsClosedHierarchy, IsUnion, IsEnum, IsValueType.
    for (ty, expected) in [
        ("Open<Int32>", [false, true, false, false, false, false]),
        ("Abstract", [true, true, false, false, false, false]),
        ("Leaf", [false, false, false, false, false, false]),
        ("Closed", [true, false, true, false, false, false]),
        ("Contract", [true, true, false, false, false, false]),
        ("Choice<Int32>", [false, false, false, true, false, true]),
        ("Number", [false, false, false, false, true, true]),
        ("Int32", [false, false, false, false, false, true]),
        ("Int32&", [false, false, false, false, false, false]),
        (
            "arrayref<Int32>",
            [false, false, false, false, false, false],
        ),
    ] {
        for (property, expected) in [
            "IsAbstract",
            "IsOpen",
            "IsClosedHierarchy",
            "IsUnion",
            "IsEnum",
            "IsValueType",
        ]
        .into_iter()
        .zip(expected)
        {
            let branch = if expected { "brtrue" } else { "brfalse" };
            body.push_str(&format!(
                "call System.Runtime.RuntimeContext::get_Current()\nldtoken {ty}\ncall instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)\ncallvirt instance System.Introspection.TypeInfo::get_{property}()\n{branch} checked{check}\nfault \"{ty}.{property}\"\nchecked{check}:\n"
            ));
            check += 1;
        }
    }
    let source = format!(
        ".module Flags\n.entry Main\n{definitions}\n.function Main() -> Boolean\n{body}ldc.bool true\nret\n.end"
    );
    let app = assemble(&source).unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Boolean(true)
    );
}

#[test]
fn inheritance_classification_metadata_round_trips_and_checks_directives() {
    let module = neoclr::assemble(
        ".module Flags\n.type class Leaf\n.sealed\n.end\n.interface Family\n.closedhierarchy\n.end",
    )
    .unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert!(loaded.types[0].is_sealed);
    assert!(loaded.types[1].is_closed_hierarchy);
    for directive in [
        ".sealed extra",
        ".closedhierarchy extra",
        ".sealed\n.sealed",
        ".closedhierarchy\n.closedhierarchy",
    ] {
        assert!(
            neoclr::assemble(&format!(".module Flags\n.type class X\n{directive}\n.end")).is_err()
        );
    }
}
