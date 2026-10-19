use neoclr::{Limits, Value, assemble, run_with_library, verify_with_library};

fn library() -> neoclr::Module {
    assemble(&format!(
        ".module System\n{}",
        std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/memory_views.neoil"
        ))
        .unwrap()
    ))
    .unwrap()
}

fn app(extra: &str, body: &str, returns: &str) -> neoclr::Module {
    let source = format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    );
    neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        &library(),
    )
    .unwrap()
    .remove(0)
}

#[test]
fn nested_views_alias_array_storage_without_guest_allocations() {
    let module = app(
        "",
        r#"
.local arrayref<Int32> data
.local System.Slice<Int32> slice
ldc.i4 5
newarr Int32
stloc data
ldloc data
ldc.i4 1
ldc.i4 3
call System.MemoryViews.AsSlice<Int32>(arrayref<Int32>,Int32,Int32)
stloc slice
ldloc slice
ldc.i4 1
call instance System.Slice<Int32>::Slice(Int32)
ldc.i4 0
ldc.i4 42
call instance System.Slice<Int32>::set_Item(Int32,Int32)
pop
ldloc data
ldc.i4 2
ldelem Int32
"#,
        "Int32",
    );
    let library = library();
    verify_with_library(&module, &library).unwrap();
    let result = run_with_library(&module, &library, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
}

#[test]
fn readonly_view_observes_writable_alias_and_utf8_is_byte_based() {
    let library = library();
    let module = app(
        "",
        r#"
.local arrayref<Int32> data
.local System.ReadOnlySlice<Int32> read
ldc.i4 1
newarr Int32
stloc data
ldloc data
call System.MemoryViews.AsReadOnlySlice<Int32>(arrayref<Int32>)
stloc read
ldloc data
ldc.i4 0
ldc.i4 71
stelem Int32
ldloc read
ldc.i4 0
call instance System.ReadOnlySlice<Int32>::get_Item(Int32)
"#,
        "Int32",
    );
    verify_with_library(&module, &library).unwrap();
    assert_eq!(
        run_with_library(&module, &library, Limits::default())
            .unwrap()
            .value,
        Value::Int32(71)
    );
    let module = app(
        "",
        "ldstr \"é\"\ncall System.MemoryViews.AsUtf8Slice(String)\nldc.i4 1\ncall instance System.ReadOnlySlice<Byte>::get_Item(Int32)",
        "Byte",
    );
    verify_with_library(&module, &library).unwrap();
    assert_eq!(
        run_with_library(&module, &library, Limits::default())
            .unwrap()
            .value,
        Value::Byte(0xa9)
    );
}

#[test]
fn memory_in_heap_field_survives_collection_and_yield() {
    let module = app(
        r#"
.type Holder
.field Data System.Memory<Int32>
.end
.function neoCLR.Runtime.RuntimeAsyncYield() -> Void
.methodimpl InternalCall
.end
.function Make() -> Holder&
.local arrayref<Int32> data
ldc.i4 1
newarr Int32
stloc data
ldloc data
ldc.i4 0
ldc.i4 42
stelem Int32
ldloc data
call System.MemoryViews.AsMemory<Int32>(arrayref<Int32>)
newobj Holder
heap.new
ret
.end
"#,
        r#"
.local Holder& holder
call Make()
stloc holder
ldc.i4 0
heap.new
pop
ldc.i4 1
heap.new
pop
call neoCLR.Runtime.RuntimeAsyncYield()
pop
ldloc holder
ldfld Holder::Data
call instance System.Memory<Int32>::AsSlice()
ldc.i4 0
call instance System.Slice<Int32>::get_Item(Int32)
"#,
        "Int32",
    );
    let library = library();
    verify_with_library(&module, &library).unwrap();
    let result = run_with_library(
        &module,
        &library,
        Limits {
            heap_objects: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.reclaimed_objects() >= 2);
}

#[test]
fn invalid_windows_and_element_indexes_fault() {
    let library = library();
    for (start, count) in [(-1, 0), (0, -1), (4, 0), (1, 3), (i32::MAX, i32::MAX)] {
        let module = app(
            "",
            &format!(
                "ldc.i4 3\nnewarr Int32\nldc.i4 {start}\nldc.i4 {count}\ncall System.MemoryViews.AsSlice<Int32>(arrayref<Int32>,Int32,Int32)\ncall instance System.Slice<Int32>::get_Length()"
            ),
            "Int32",
        );
        verify_with_library(&module, &library).unwrap();
        assert!(run_with_library(&module, &library, Limits::default()).is_err());
    }
    for index in [-1, 0, 1] {
        let module = app(
            "",
            &format!(
                "ldc.i4 0\nnewarr Int32\ncall System.MemoryViews.AsSlice<Int32>(arrayref<Int32>)\nldc.i4 {index}\ncall instance System.Slice<Int32>::get_Item(Int32)"
            ),
            "Int32",
        );
        assert!(run_with_library(&module, &library, Limits::default()).is_err());
    }
}

#[test]
fn empty_null_and_default_views_are_usable() {
    let library = library();
    for body in [
        ".local arrayref<Int32> empty\nldloca empty\ninitobj arrayref<Int32>\nldloc empty\ncall System.MemoryViews.AsSlice<Int32>(arrayref<Int32>)",
        ".local System.Slice<Int32> view\nldloca view\ninitobj System.Slice<Int32>\nldloc view",
        "ldc.i4 2\nnewarr Int32\nldc.i4 2\nldc.i4 0\ncall System.MemoryViews.AsSlice<Int32>(arrayref<Int32>,Int32,Int32)",
    ] {
        let module = app(
            "",
            &format!("{body}\ncall instance System.Slice<Int32>::get_Length()"),
            "Int32",
        );
        verify_with_library(&module, &library).unwrap();
        assert_eq!(
            run_with_library(&module, &library, Limits::default())
                .unwrap()
                .value,
            Value::Int32(0)
        );
    }
}

#[test]
fn borrowed_views_remain_usable_across_synchronous_host_calls() {
    let library = library();
    let module = app(
        ".function neoCLR.Runtime.WriteLine(String text) -> Void\n.methodimpl InternalCall\n.end",
        r#"
.local System.Slice<Int32> view
ldc.i4 1
newarr Int32
call System.MemoryViews.AsSlice<Int32>(arrayref<Int32>)
stloc view
ldloc view
ldc.i4 0
ldc.i4 42
call instance System.Slice<Int32>::set_Item(Int32,Int32)
pop
ldstr "borrow retained"
call neoCLR.Runtime.WriteLine(String)
pop
ldloc view
ldc.i4 0
call instance System.Slice<Int32>::get_Item(Int32)
"#,
        "Int32",
    );
    verify_with_library(&module, &library).unwrap();
    let result = run_with_library(&module, &library, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["borrow retained"]);
}

#[test]
fn borrowed_views_cannot_cross_yield_or_escape_through_boxing() {
    let library = library();
    let extra =
        ".function neoCLR.Runtime.RuntimeAsyncYield() -> Void\n.methodimpl InternalCall\n.end";
    let module = app(
        extra,
        ".local System.Slice<Int32> view\nldc.i4 1\nnewarr Int32\ncall System.MemoryViews.AsSlice<Int32>(arrayref<Int32>)\nstloc view\ncall neoCLR.Runtime.RuntimeAsyncYield()",
        "Void",
    );
    verify_with_library(&module, &library).unwrap();
    assert!(
        run_with_library(&module, &library, Limits::default())
            .unwrap_err()
            .message
            .contains("suspension")
    );
    let module = app(
        "",
        "ldc.i4 1\nnewarr Int32\ncall System.MemoryViews.AsSlice<Int32>(arrayref<Int32>)\nheap.new",
        "System.Slice<Int32>&",
    );
    assert!(run_with_library(&module, &library, Limits::default()).is_err());
}

#[test]
fn opaque_views_reject_records_fields_arrays_and_generic_escape() {
    let library = library();
    for (extra, body, returns) in [
        ("", "newobj System.Slice<Int32>", "System.Slice<Int32>"),
        (
            ".type Bad\n.field View System.Slice<Int32>\n.end",
            "ldc.i4 0",
            "Int32",
        ),
        (
            "",
            "ldc.i4 1\nnewarr System.Slice<Int32>",
            "arrayref<System.Slice<Int32>>",
        ),
        (
            ".type Box<T>\n.field Value T\n.end",
            ".local Box<System.Slice<Int32>> x\nldc.i4 0",
            "Int32",
        ),
    ] {
        let source = format!(
            ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
        );
        assert!(
            neoclr::assembler::read_modules(
                &[neoclr::assembler::ModuleInput::Source(&source)],
                &library
            )
            .and_then(|modules| verify_with_library(&modules[0], &library))
            .is_err(),
            "{body}"
        );
    }
}

#[test]
fn caller_borrow_can_return_but_callee_storage_cannot() {
    let library = library();
    let helper = r#"
.function Borrow(Int32[]& data) -> System.Slice<Int32>
ldarg data
call System.MemoryViews.AsSlice<Int32>(Int32[]&)
ret
.end
.function Bad() -> System.Slice<Int32>
.local Int32[] data
ldc.i4 1
array.new Int32
ldobj Int32[]
stloc data
ldloca data
call Borrow(Int32[]&)
ret
.end
"#;
    let module = app(
        helper,
        "call Bad()\ncall instance System.Slice<Int32>::get_Length()",
        "Int32",
    );
    verify_with_library(&module, &library).unwrap();
    assert!(
        run_with_library(&module, &library, Limits::default())
            .unwrap_err()
            .message
            .contains("current frame")
    );
    let module = app(
        helper,
        ".local Int32[] data\nldc.i4 1\narray.new Int32\nldobj Int32[]\nstloc data\nldloca data\ncall Borrow(Int32[]&)\nldc.i4 0\nldc.i4 73\ncall instance System.Slice<Int32>::set_Item(Int32,Int32)\npop\nldloca data\nldc.i4 0\nldelem Int32",
        "Int32",
    );
    verify_with_library(&module, &library).unwrap();
    assert_eq!(
        run_with_library(&module, &library, Limits::default())
            .unwrap()
            .value,
        Value::Int32(73)
    );
    let module = app(
        "",
        ".local Int32[] data\nldc.i4 1\narray.new Int32\nldobj Int32[]\nstloc data\nldloca data\ncall System.MemoryViews.AsMemory<Int32>(Int32[]&)\ncall instance System.Memory<Int32>::get_Length()",
        "Int32",
    );
    assert!(run_with_library(&module, &library, Limits::default()).is_err());
}

#[test]
fn readonly_byref_factory_and_persistent_subwindows_keep_capabilities() {
    let library = library();
    let module = app(
        "",
        "ldc.i4 2\narray.new Int32\ncall System.MemoryViews.AsReadOnlySlice<Int32>(Int32[]&)\ncall instance System.ReadOnlySlice<Int32>::get_Length()",
        "Int32",
    );
    verify_with_library(&module, &library).unwrap();
    assert_eq!(
        run_with_library(&module, &library, Limits::default())
            .unwrap()
            .value,
        Value::Int32(2)
    );
    let module = app(
        "",
        "ldc.i4 3\nnewarr Int32\ncall System.MemoryViews.AsMemory<Int32>(arrayref<Int32>)\nldc.i4 1\nldc.i4 2\ncall instance System.Memory<Int32>::Slice(Int32,Int32)\ncall instance System.Memory<Int32>::AsReadOnly()\ncall instance System.ReadOnlyMemory<Int32>::AsSlice()\ncall instance System.ReadOnlySlice<Int32>::get_Length()",
        "Int32",
    );
    verify_with_library(&module, &library).unwrap();
    assert_eq!(
        run_with_library(&module, &library, Limits::default())
            .unwrap()
            .value,
        Value::Int32(2)
    );
}
