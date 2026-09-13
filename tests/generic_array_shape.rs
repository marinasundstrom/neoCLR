use neoclr::{Limits, Value, assemble, run_with_library, verify_with_library};

const ITERATION: &str = r#"
.interface System.Collections.Iterable<T>
.method instance GetIterator() -> System.Collections.Iterator<T>
.end
.end
.interface System.Collections.Iterator<T>
.method instance Read() -> T
.end
.end
.type class TestIterator<T>
.implements System.Collections.Iterator<T>
.field Data arrayref<T>
.method instance Read() -> T
ldarg this
ldfld TestIterator<T>::Data
ldc.i4 0
ldelem T
ret
.end
.end
.type System.Collections.ArrayEnumerable
.method static GetIterator<T>(arrayref<T> data) -> System.Collections.Iterator<T>
ldarg data
newobj TestIterator<T>
ret
.end
.end
"#;

fn library() -> neoclr::Module {
    assemble(&format!(
        ".module System\n{ITERATION}\n{}",
        include_str!("../runtime/raven/Array.neoil")
    ))
    .unwrap()
}

#[test]
fn generic_name_and_cli_array_share_storage_and_member_calls() {
    let library = library();
    let m = assemble(
        r#".module App
.entry Main
.function Main() -> Int32
.local System.Array<Int32> array
.local arrayref<Int32> alias
ldc.i4 2
newarr Int32
stloc array
ldloc array
stloc alias
ldloc array
ldc.i4 1
ldc.i4 42
call instance System.Array<Int32>::set_Item(Int32,Int32)
ldloc array
ldloc alias
ref.eq
brfalse Bad
ldloc array
call instance System.Array<Int32>::get_Length()
ldc.i4 2
ceq
brfalse Bad
ldloc alias
ldc.i4 1
call instance System.Array<Int32>::get_Item(Int32)
ret
Bad:
ldc.i4 -1
ret
.end"#,
    )
    .unwrap();
    verify_with_library(&m, &library).unwrap();
    let result = run_with_library(&m, &library, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
}

#[test]
fn managed_array_definition_cannot_add_record_storage() {
    let fault =
        assemble(".module System\n.type class System.Array<T>\n.field Data T*\n.end").unwrap_err();
    assert!(fault.message.contains("intrinsic System array shape"));
}

#[test]
fn array_alias_preserves_bounds_and_invariance() {
    for (body, message) in [
        (
            "ldc.i4 1\nnewarr Int32\nldc.i4 1\ncall instance System.Array<Int32>::get_Item(Int32)\nret",
            "array index out of range",
        ),
        (
            "ldc.i4 1\nnewarr Int32\nldc.i4 -1\nldc.i4 42\ncall instance System.Array<Int32>::set_Item(Int32,Int32)\nldc.i4 0\nret",
            "array index out of range",
        ),
        (
            "ldc.i4 1\nnewarr Int32\ncastclass System.Array<String>\npop\nldc.i4 0\nret",
            "identical element types",
        ),
        ("newobj System.Array<Int32>\npop\nldc.i4 0\nret", "record"),
    ] {
        let app = assemble(&format!(
            ".module App\n.entry Main\n.function Main() -> Int32\n{body}\n.end"
        ))
        .unwrap();
        let fault = run_with_library(&app, &library(), Limits::default()).unwrap_err();
        assert!(fault.message.contains(message), "{fault:?}");
    }
}

fn reflection_library() -> neoclr::Module {
    // Preserve historical library contracts separately while using ordinary reference
    // iterator declarations for the managed-array profile under test.
    let source = neoclr::library::system_source()
        .replace(include_str!("../runtime/System/Array.neoil"), "")
        .replace("System.Collections.Iterable", "Historical.Iterable")
        .replace("System.Collections.Iterator", "Historical.Iterator");
    assemble(&format!(
        "{source}\n{ITERATION}\n{}",
        include_str!("../runtime/raven/Array.neoil")
    ))
    .unwrap()
}

#[test]
fn generic_identity_and_reflection_describe_the_existing_array() {
    let library = reflection_library();
    let app = assemble(".module App").unwrap();
    let loaded = neoclr::LoadedProgram::with_library(&app, &library).unwrap();
    let named = neoclr::assembler::parse_type("System.Array<Int32>").unwrap();
    let cli = neoclr::assembler::parse_type("arrayref<Int32>").unwrap();
    let a = loaded.describe_type(&named).unwrap();
    let b = loaded.describe_type(&cli).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.generic_arguments.len(), 1);
    assert_eq!(a.generic_arguments[0].name, "System.Int32");
    for (method, result, count) in [
        ("GetProperties", "System.Reflection.PropertyInfo[]", 2),
        ("GetMethods", "System.Reflection.MethodInfo[]", 3),
        ("GetGenericArguments", "System.Type[]", 1),
    ] {
        let app = assemble(&format!(".module App\n.entry Main\n.function Main() -> {result}\nldtoken arrayref<Int32>\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\ncall instance System.Type::{method}()\nret\n.end")).unwrap();
        let value = run_with_library(&app, &library, Limits::default())
            .unwrap()
            .value;
        let Value::Array { elements, .. } = value else {
            panic!("expected metadata array");
        };
        assert_eq!(elements.len(), count, "{method}");
    }
}

#[test]
fn declared_iterable_dispatch_reads_the_original_array() {
    let source = r#".module App
.entry Main
.function Main() -> Int32
.local System.Array<Int32> values
ldc.i4 1
newarr Int32
stloc values
ldloc values
ldc.i4 0
ldc.i4 42
stelem Int32
ldloc values
castclass System.Collections.Iterable<Int32>
callvirt instance System.Collections.Iterable<Int32>::GetIterator()
callvirt instance System.Collections.Iterator<Int32>::Read()
ret
.end"#;
    let library = library();
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(source)],
        &library,
    )
    .unwrap()
    .remove(0);
    verify_with_library(&app, &library).unwrap();
    assert_eq!(
        run_with_library(&app, &library, Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let definition = library
        .types
        .iter()
        .find(|t| t.name == "System.Array")
        .unwrap();
    assert_eq!(definition.implements.len(), 1);
}
