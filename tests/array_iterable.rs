use neoclr::{Limits, Value, assemble, run_with_library, verify_with_library};

const TYPES: &str = r#"
.interface System.Collections.Iterable<T>
.method instance GetIterator() -> System.Collections.Iterator<T>
.end
.end
.interface System.Collections.Iterator<T>
.method instance Read() -> T
.end
.end
.type class Cursor<T>
.implements System.Collections.Iterator<T>
.field Data arrayref<T>
.method instance Read() -> T
ldarg this
ldfld Cursor<T>::Data
ldc.i4 0
ldelem T
ret
.end
.end
.type internal System.Collections.ArrayIterator<T>
.method internal static Create(arrayref<T> data, Int32 count) -> System.Collections.Iterator<T>
ldarg data
newobj Cursor<T>
ret
.end
.end
"#;
fn module(body: &str) -> (neoclr::Module, neoclr::Module) {
    let library = assemble(".module System").unwrap();
    let source = format!(
        ".module ArrayIterable\n.entry Main\n{TYPES}\n{}\n{body}",
        include_str!("../runtime/raven/ArrayEnumerable.neoil")
    );
    let m = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        &library,
    )
    .unwrap()
    .remove(0);
    (m, library)
}

#[test]
fn array_interface_preserves_identity_and_dispatches_the_original_buffer() {
    let (m, library) = module(
        r#"
.function Main() -> Int32
.local arrayref<Int32> data
.local System.Collections.Iterable<Int32> view
ldc.i4 1
newarr Int32
stloc data
ldloc data
stloc view
ldloc data
ldloc view
castclass arrayref<Int32>
ref.eq
brfalse Wrong
ldloc view
castclass arrayref<Int32>
stloc data
ldloc data
ldc.i4 0
ldc.i4 42
stelem Int32
ldloc view
callvirt instance System.Collections.Iterable<Int32>::GetIterator()
callvirt instance System.Collections.Iterator<Int32>::Read()
ret
Wrong:
ldc.i4 -1
ret
.end
"#,
    );
    verify_with_library(&m, &library).unwrap();
    assert_eq!(
        run_with_library(&m, &library, Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn array_interface_is_invariant_and_null_dispatch_faults() {
    for (body, message) in [
        (
            "ldc.i4 0\nnewarr Int32\ncastclass System.Collections.Iterable<String>\npop\nldc.i4 0\nret",
            "matching interface",
        ),
        (
            ".local arrayref<Int32> empty\nldloca empty\ninitobj arrayref<Int32>\nldloc empty\ncastclass System.Collections.Iterable<Int32>\ncallvirt instance System.Collections.Iterable<Int32>::GetIterator()\npop\nldc.i4 0\nret",
            "null interface receiver",
        ),
    ] {
        let (m, library) = module(&format!(".function Main() -> Int32\n{body}\n.end"));
        verify_with_library(&m, &library).unwrap();
        assert!(
            run_with_library(&m, &library, Limits::default())
                .unwrap_err()
                .to_string()
                .contains(message)
        );
    }
}
