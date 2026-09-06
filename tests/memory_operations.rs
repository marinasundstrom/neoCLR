use neoclr::{Limits, Value, assemble, load, run};

fn execute(body: &str) -> Result<Value, neoclr::Fault> {
    let module = assemble(&format!(".module Test\n.entry Main\n.function Main() -> Int32\n.local Int32* p\n.local Int32* q\nldc.i4 2\nheap.alloc Int32\nstloc p\nldc.i4 2\nheap.alloc Int32\nstloc q\n{body}\nret\n.end")).unwrap();
    run(&module, Limits::default()).map(|e| e.value)
}

#[test]
fn sample_roundtrips_and_releases_allocations() {
    let module = assemble(include_str!("../examples/memory.neoil")).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(result.output, ["42", "42"]);
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn typed_initialization_and_copy_preserve_value_independence() {
    assert_eq!(execute("ldloc p\ninitobj Int32\nldloc q\nldloc p\ncpobj Int32\nldloc p\nldc.i4 42\nstobj Int32\nldloc q\nldobj Int32").unwrap(), Value::Int32(0));
}

#[test]
fn block_fill_truncates_and_copy_supports_overlap_and_unaligned_bytes() {
    assert_eq!(execute("ldloc p\nldc.i4 257\nldc.i4 8\ninitblk\nldloc p\nldc.i4 1\nptr.add\nptr.cast Byte\nldc.i4 2\nstind.i1\nldloc p\nldc.i4 2\nptr.add\nldloc p\nldc.i4 1\nptr.add\nldc.i4 4\ncpblk\nldloc p\nldc.i4 2\nptr.add\nptr.cast Byte\nldind.u1\nldloc p\nldc.i4 3\nptr.add\nptr.cast Byte\nldind.u1\nadd").unwrap(), Value::Int32(3));
}

#[test]
fn copying_uninitialized_bytes_does_not_initialize_them() {
    for copy in [
        "ldloc q\nldloc p\nldc.i4 4\ncpblk\nldloc q\nldobj Int32",
        "ldloc q\nldloc p\ncpobj Int32\nldc.i4 0",
    ] {
        assert!(
            execute(copy)
                .unwrap_err()
                .to_string()
                .contains("uninitialized")
        );
    }
}

#[test]
fn memory_operations_validate_ranges_types_and_counts() {
    for (body, message) in [
        ("ldloc p\nldc.i4 0\nldc.i4 9\ninitblk", "out of bounds"),
        ("ldloc p\nldloc q\nldc.i4 9\ncpblk", "out of bounds"),
        (
            "ldloc p\nldc.i4 0\nldc.i4 -1\ninitblk",
            "negative block size",
        ),
        ("ldloc p\ninitobj Byte", "type mismatch"),
        ("ldloc p\nldc.i4 1\nptr.add\ninitobj Int32", "misaligned"),
        (
            "ptr.null Int32\nldc.i4 0\nldc.i4 0\ninitblk",
            "null pointer",
        ),
        (
            "ldloc p\nheap.free\npop\nldloc p\ninitobj Int32",
            "use after free",
        ),
    ] {
        assert!(
            execute(&format!("{body}\nldc.i4 0"))
                .unwrap_err()
                .to_string()
                .contains(message),
            "{body}"
        );
    }
}

#[test]
fn block_counts_accept_native_integers_and_zero_at_one_past_end() {
    for conversion in ["", "conv.i", "conv.u"] {
        assert_eq!(
            execute(&format!(
                "ldloc p\nldc.i4 8\nptr.add\nldc.i4 255\nldc.i4 0\n{conversion}\ninitblk\nldc.i4 42"
            ))
            .unwrap(),
            Value::Int32(42)
        );
    }
}

#[test]
fn block_copy_preserves_full_pointer_tracking() {
    let module = assemble(".module Test\n.entry Main\n.function Main() -> Int32\n.local Int32* p\n.local Int32** src\n.local Int32** dst\nldc.i4 1\nheap.alloc Int32\nstloc p\nldc.i4 1\nheap.alloc Int32*\nstloc src\nldc.i4 1\nheap.alloc Int32*\nstloc dst\nldloc src\nldloc p\nstobj Int32*\nldloc dst\nldloc src\nsizeof Int32*\ncpblk\nldloc p\nheap.free\npop\nldloc dst\nldobj Int32*\nldobj Int32\nret\n.end").unwrap();
    assert!(
        run(&module, Limits::default())
            .unwrap_err()
            .to_string()
            .contains("use after free")
    );
}

#[test]
fn unsupported_native_layouts_cannot_be_zero_initialized() {
    assert!(assemble(".module Test\n.function Zero(String* p) -> Void\nldarg p\ninitobj String\nldvoid\nret\n.end").is_err());
}

#[test]
fn zero_length_writes_preserve_pointer_tracking_but_partial_writes_invalidate_it() {
    for (operation, message) in [
        (
            "ldloc slot\nldc.i4 1\nptr.add\nldc.i4 0\nldc.i4 0\ninitblk",
            "use after free",
        ),
        (
            "ldloc slot\nldc.i4 1\nptr.add\nldloc slot\nldc.i4 0\ncpblk",
            "use after free",
        ),
        (
            "ldloc slot\nldc.i4 1\nptr.add\nptr.cast Void\nldvoid\nstobj Void",
            "use after free",
        ),
        // Copy a single byte onto itself: address bits stay the same, but a
        // partial pointer write cannot carry the original allocation identity.
        (
            "ldloc slot\nldloc slot\nldc.i4 1\ncpblk",
            "untracked native pointer access",
        ),
    ] {
        let module = assemble(&format!(".module Test\n.entry Main\n.function Main() -> Int32\n.local Int32* p\n.local Int32** slot\nldc.i4 1\nheap.alloc Int32\nstloc p\nldc.i4 1\nheap.alloc Int32*\nstloc slot\nldloc slot\nldloc p\nstobj Int32*\n{operation}\nldloc p\nheap.free\npop\nldloc slot\nldobj Int32*\nldobj Int32\nret\n.end")).unwrap();
        assert!(
            run(&module, Limits::default())
                .unwrap_err()
                .to_string()
                .contains(message),
            "{operation}"
        );
    }
}
