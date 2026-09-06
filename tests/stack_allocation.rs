use neoclr::{Limits, Value, assemble, load, run};

fn program(body: &str, extra: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n.function Main() -> Int32\n{body}\nret\n.end\n{extra}"
    ))
    .unwrap()
}

#[test]
fn sample_passes_frame_storage_to_callee_and_returns_an_independent_record() {
    let module = assemble(include_str!("../examples/stack.neoil")).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(result.output, ["42"]);
    assert_eq!(result.memory.live_bytes(), 0);
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn local_storage_is_uninitialized_and_cannot_be_freed_manually() {
    for (body, message) in [
        (
            "ldc.i4 4\nlocalloc\nptr.cast Int32\nldobj Int32",
            "uninitialized",
        ),
        (
            "ldc.i4 4\nlocalloc\nheap.free\npop\nldc.i4 0",
            "frame-owned",
        ),
        (
            "ldc.i4 4\nlocalloc\nconv.u\nptr.fromint Byte\nheap.free\npop\nldc.i4 0",
            "frame-owned",
        ),
    ] {
        assert!(
            run(&program(body, ""), Limits::default())
                .unwrap_err()
                .to_string()
                .contains(message)
        );
    }
}

#[test]
fn escaped_local_pointers_fault_after_return_even_when_stored_in_heap() {
    let extra = ".function Escape() -> Int32*\nldc.i4 4\nlocalloc\nptr.cast Int32\ndup\ninitobj Int32\nret\n.end";
    for body in [
        "call Escape()\nldobj Int32",
        ".local Int32** slot\nldc.i4 1\nheap.alloc Int32*\nstloc slot\nldloc slot\ncall Escape()\nstobj Int32*\nldloc slot\nldobj Int32*\nldobj Int32",
    ] {
        assert!(
            run(&program(body, extra), Limits::default())
                .unwrap_err()
                .to_string()
                .contains("use after free")
        );
    }
}

#[test]
fn callee_release_keeps_caller_storage_live() {
    let body = ".local Int32* p\nldc.i4 4\nlocalloc\nptr.cast Int32\nstloc p\nldloc p\ncall Mutate(Int32*)\npop\nldloc p\nldobj Int32";
    let extra = ".function Mutate(Int32* p) -> Void\nldc.i4 8\nlocalloc\npop\nldarg p\nldc.i4 42\nstobj Int32\nldvoid\nret\n.end";
    let result = run(&program(body, extra), Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn returning_reclaims_byte_budget_but_preserves_heap_allocations() {
    let extra = ".function Allocate() -> Void\nldc.i4 8\nlocalloc\npop\nldvoid\nret\n.end";
    let body =
        "ldc.i4 1\nheap.alloc Int32\npop\ncall Allocate()\npop\ncall Allocate()\npop\nldc.i4 42";
    let result = run(
        &program(body, extra),
        Limits {
            pointer_bytes: 12,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.memory.live_bytes(), 4);
    assert_eq!(result.memory.live_allocations(), 1);
}

#[test]
fn local_allocations_obey_limits_and_stack_contract() {
    for (body, limits, message) in [
        (
            "ldc.i4 -1\nlocalloc",
            Limits::default(),
            "negative localloc size",
        ),
        (
            "ldc.bool true\nlocalloc",
            Limits::default(),
            "requires integer",
        ),
        (
            "ldc.i4 1\nldc.i4 4\nlocalloc",
            Limits::default(),
            "only its size",
        ),
        (
            "ldc.i4 9\nlocalloc",
            Limits {
                pointer_bytes: 8,
                ..Limits::default()
            },
            "byte limit",
        ),
        (
            "ldc.i4 0\nlocalloc",
            Limits {
                pointer_allocations: 0,
                ..Limits::default()
            },
            "allocation limit",
        ),
    ] {
        assert!(
            run(&program(body, ""), limits)
                .unwrap_err()
                .to_string()
                .contains(message)
        );
    }
}

#[test]
fn zero_size_and_native_counts_have_frame_lifetime() {
    for conversion in ["", "conv.i", "conv.u"] {
        let body =
            format!("ldc.i4 0\n{conversion}\nlocalloc\nldc.i4 0\nldc.i4 0\ninitblk\nldc.i4 42");
        let result = run(&program(&body, ""), Limits::default()).unwrap();
        assert_eq!(result.value, Value::Int32(42));
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn local_storage_supports_widest_primitive_alignment() {
    let body = ".local Double* p\nldc.i4 8\nlocalloc\nptr.cast Double\nstloc p\nldloc p\nldc.r8 42\nstind.r8\nldloc p\nldind.r8\nconv.i4";
    assert_eq!(
        run(&program(body, ""), Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
