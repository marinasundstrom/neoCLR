use neoclr::{Limits, Value, assemble, load, memory::layout, metadata::Type, run};

fn program(returns: &str, body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap()
}
fn eval(returns: &str, body: &str) -> Value {
    run(&program(returns, body), Limits::default())
        .unwrap()
        .value
}

#[test]
fn point_sample_initializes_reads_copies_and_frees_storage() {
    let module = assemble(include_str!("../examples/pointers.neoil")).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let execution = run(&loaded, Limits::default()).unwrap();
    assert_eq!(execution.output, ["42"]);
    assert_eq!(execution.memory.live_allocations(), 0);
    assert_eq!(execution.memory.live_bytes(), 0);
    assert!(execution.heap.is_empty());
}

#[test]
fn layout_has_deterministic_field_alignment_and_tail_padding() {
    let module = assemble(".module Layout\n.type Inner\n.field flag Boolean\n.field number Int32\n.end\n.type Outer\n.field prefix Boolean\n.field nested Inner\n.field suffix Boolean\n.end").unwrap();
    let inner = layout(&module, &Type::Named("Inner".into())).unwrap();
    assert_eq!((inner.size, inner.alignment), (8, 4));
    assert_eq!(
        inner.fields.iter().map(|f| f.offset).collect::<Vec<_>>(),
        [0, 4]
    );
    let outer = layout(&module, &Type::Named("Outer".into())).unwrap();
    assert_eq!((outer.size, outer.alignment), (16, 4));
    assert_eq!(
        outer.fields.iter().map(|f| f.offset).collect::<Vec<_>>(),
        [0, 4, 12]
    );
}

#[test]
fn aggregate_reads_require_fields_but_not_padding_to_be_initialized() {
    let module = assemble(".module Test\n.entry Main\n.type Mixed\n.field flag Boolean\n.field value Int32\n.end\n.function Main() -> Mixed\n.local Mixed* p\nldc.i4 1\nheap.alloc Mixed\nstloc p\nldloc p\nldflda 0\nldc.bool true\nstobj Boolean\nldloc p\nldflda 1\nldc.i4 42\nstind.i4\nldloc p\nldobj Mixed\nldloc p\nheap.free\npop\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Object {
            name: "Mixed".into(),
            fields: vec![Value::Boolean(true), Value::Int32(42)]
        }
    );
}

#[test]
fn pointer_aliases_observe_writes_and_offsets_can_move_back_from_one_past() {
    assert_eq!(
        eval(
            "Int32",
            ".local int32* p\nldc.i4 2\nheap.alloc int32\nstloc p\nldloc p\nldc.i4 4\nptr.add\nldc.i4 -123\nstind.i4\nldloc p\nldc.i4 8\nptr.add\nldc.i4 -4\nptr.add\nldind.i4\nldloc p\nheap.free\npop"
        ),
        Value::Int32(-123)
    );
}

#[test]
fn invalid_pointer_operations_fault_with_instruction_locations() {
    for (body, expected) in [
        ("ptr.null int32\nldind.i4", "null pointer"),
        ("ptr.null int32\nldc.i4 1\nstind.i4", "null pointer"),
        ("ptr.null int32\nldc.i4 0\nptr.add", "null pointer"),
        ("ldc.i4 1\nheap.alloc int32\nldind.i4", "uninitialized"),
        (
            "ldc.i4 1\nheap.alloc int32\nldc.i4 4\nptr.add\nldind.i4",
            "out of bounds",
        ),
        (
            "ldc.i4 1\nheap.alloc int32\nldc.i4 -1\nptr.add",
            "out of bounds",
        ),
        (
            "ldc.i4 1\nheap.alloc int32\nldc.i4 5\nptr.add",
            "out of bounds",
        ),
        (
            "ldc.i4 2\nheap.alloc int32\nldc.i4 1\nptr.add\nldind.i4",
            "misaligned",
        ),
        (
            "ldc.i4 2\nheap.alloc int32\nldc.i4 4\nptr.add\nheap.free",
            "base pointer",
        ),
        (
            "ldc.i4 1\nheap.alloc int32\ndup\nheap.free\npop\nldind.i4",
            "use after free",
        ),
        (
            "ldc.i4 1\nheap.alloc int32\ndup\nheap.free\npop\nheap.free",
            "double free",
        ),
        ("ldc.i4 -1\nheap.alloc int32", "negative allocation"),
        ("ldc.i4 0\nheap.alloc int32\nldind.i4", "out of bounds"),
        (
            "ldc.i4 1\nheap.alloc int32\nldc.bool true\nstind.i4",
            "store type mismatch",
        ),
        (
            "ldc.i4 1\nheap.alloc Boolean\nldind.i4",
            "pointer type mismatch",
        ),
        ("ldvoid\nheap.free", "expected Ptr"),
        ("ldc.i4 1\nheap.alloc int32\nldflda 0", "pointer to record"),
    ] {
        let fault = run(&program("Void", body), Limits::default()).unwrap_err();
        assert!(fault.message.contains(expected), "{body}: {fault}");
        assert_eq!(fault.function.as_deref(), Some("Main"));
        assert!(fault.instruction.is_some());
    }
}

#[test]
fn allocation_identities_are_not_reused_after_free() {
    let fault = run(&program("Int32", ".local int32* stale\nldc.i4 1\nheap.alloc int32\ndup\nstloc stale\nheap.free\npop\nldc.i4 1\nheap.alloc int32\nldc.i4 42\nstind.i4\nldloc stale\nldind.i4"), Limits::default()).unwrap_err();
    assert!(fault.message.contains("use after free"));
}

#[test]
fn null_and_zero_sized_allocations_have_defined_behavior() {
    assert_eq!(eval("Void", "ptr.null int32\nheap.free"), Value::Void);
    assert_eq!(
        eval("Boolean", "ptr.null int32\nptr.null int32\nceq"),
        Value::Boolean(true)
    );
    assert_eq!(
        eval(
            "Boolean",
            "ldc.i4 0\nheap.alloc Void\nldc.i4 0\nheap.alloc Void\nceq"
        ),
        Value::Boolean(false)
    );
    assert_eq!(
        eval(
            "Void",
            "ldc.i4 1\nheap.alloc Void\ndup\nldobj Void\npop\nheap.free"
        ),
        Value::Void
    );
}

#[test]
fn freeing_releases_byte_budget_but_not_allocation_identity_budget() {
    let module = program(
        "Void",
        "ldc.i4 1\nheap.alloc int32\nheap.free\npop\nldc.i4 1\nheap.alloc int32\nheap.free",
    );
    let result = run(
        &module,
        Limits {
            pointer_bytes: 4,
            pointer_allocations: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.memory.live_bytes(), 0);
    assert!(
        run(
            &module,
            Limits {
                pointer_allocations: 1,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("allocation limit")
    );
    assert!(
        run(
            &program("Int32*", "ldc.i4 2\nheap.alloc int32"),
            Limits {
                pointer_bytes: 4,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("byte limit")
    );
}

#[test]
fn pointer_can_escape_call_frame_until_explicit_free() {
    let module = assemble(".module Test\n.entry Main\n.function Make() -> int32*\nldc.i4 1\nheap.alloc int32\ndup\nldc.i4 42\nstind.i4\nret\n.end\n.function Main() -> int32\n.local int32* p\ncall Make()\nstloc p\nldloc p\nldind.i4\nldloc p\nheap.free\npop\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn unsupported_and_recursive_layouts_are_rejected_before_execution() {
    for ty in ["String", "Error", "Ref<Int32>", "Option<Int32>"] {
        let source =
            format!(".module Test\n.entry Main\n.function Main() -> Int32\nsizeof {ty}\nret\n.end");
        assert!(
            assemble(&source)
                .unwrap_err()
                .message
                .contains("layout is not implemented")
        );
    }
    let source = ".module Test\n.type Recursive\n.field self Recursive\n.end\n.function Unused() -> Int32\nsizeof Recursive\nret\n.end";
    assert!(assemble(source).unwrap_err().message.contains("recursive"));
}

#[test]
fn records_can_be_allocated_as_contiguous_elements() {
    let module = assemble(".module Test\n.entry Main\n.type Box\n.field value Int32\n.end\n.function Main() -> Int32\n.local Box* p\nldc.i4 2\nheap.alloc Box\nstloc p\nldloc p\nsizeof Box\nptr.add\nldc.i4 42\nnewobj Box\nstobj Box\nldloc p\nsizeof Box\nptr.add\nldflda 0\nldind.i4\nldloc p\nheap.free\npop\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn addresses_are_native_and_pointer_storage_has_native_width() {
    let execution = run(
        &program(
            "int32*",
            "ldc.i4 1\nheap.alloc int32\ndup\nldc.i4 42\nstind.i4",
        ),
        Limits::default(),
    )
    .unwrap();
    let Value::Pointer(pointer) = &execution.value else {
        panic!("expected pointer")
    };
    assert_ne!(pointer.address, 0);
    assert_eq!(pointer.address % std::mem::align_of::<i32>(), 0);
    // SAFETY: execution owns this live, aligned native allocation, initialized
    // above as an i32. There is no concurrent access and execution stays alive.
    assert_eq!(unsafe { (pointer.address as *const i32).read() }, 42);
    assert_eq!(
        eval("int32", "sizeof int32*"),
        Value::Int32(std::mem::size_of::<usize>() as i32)
    );
}

#[test]
fn pointers_round_trip_through_native_storage_and_casts() {
    assert_eq!(
        eval(
            "int32",
            ".local int32* p\n.local int32** slot\nldc.i4 1\nheap.alloc int32\nstloc p\nldloc p\nldc.i4 42\nstind.i4\nldc.i4 1\nheap.alloc int32*\nstloc slot\nldloc slot\nldloc p\nstobj int32*\nldloc slot\nptr.cast Void\nptr.cast int32*\nldobj int32*\nldind.i4\nldloc p\nheap.free\npop\nldloc slot\nheap.free\npop"
        ),
        Value::Int32(42)
    );
    assert_eq!(
        eval(
            "Boolean",
            "ldc.i4 1\nheap.alloc int32*\ndup\nptr.null int32\nstobj int32*\nldobj int32*\nptr.null int32\nceq"
        ),
        Value::Boolean(true)
    );
}

#[test]
fn recursive_pointer_fields_have_finite_layout_and_preserve_aliases() {
    let module = assemble(".module Test\n.entry Main\n.type Node\n.field next Node*\n.field value int32\n.end\n.function Main() -> int32\n.local Node* node\nldc.i4 1\nheap.alloc Node\nstloc node\nldloc node\nldloc node\nldc.i4 42\nnewobj Node\nstobj Node\nldloc node\nldobj Node\nldfld 0\nldflda 1\nldind.i4\nldloc node\nheap.free\npop\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn stored_pointers_retain_use_after_free_diagnostics() {
    let fault = run(&program("int32", ".local int32* p\nldc.i4 1\nheap.alloc int32\nstloc p\nldc.i4 1\nheap.alloc int32*\ndup\nldloc p\nstobj int32*\nldloc p\nheap.free\npop\nldobj int32*\nldind.i4"), Limits::default()).unwrap_err();
    assert!(fault.message.contains("use after free"));
}

#[test]
fn overlapping_nonpointer_store_discards_pointer_tracking() {
    let fault = run(&program("int32", "ldc.i4 1\nheap.alloc int32*\ndup\nldc.i4 1\nheap.alloc int32\nstobj int32*\ndup\nptr.cast int32\ndup\nldind.i4\nstind.i4\nldobj int32*\nldind.i4"), Limits::default()).unwrap_err();
    assert!(fault.message.contains("untracked native pointer"));
}

#[test]
fn pointer_casts_validate_target_metadata_even_in_unused_bodies() {
    let source = ".module Test\n.function Unused() -> Void\nptr.null int32\nptr.cast Missing\npop\nldvoid\nret\n.end";
    assert!(assemble(source).is_err());
}
