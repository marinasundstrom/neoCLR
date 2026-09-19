use neoclr::{Limits, Value, assemble, run_with_library};

fn run(body: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let library = assemble(&format!(
        ".module System\n.type System.Runtime.InteropServices.NativeMemory\n{}\n.end\n{}",
        include_str!("../runtime/raven/generated/NativeMemory.methods.neoil"),
        include_str!("../runtime/raven/generated/NativeMemory.helpers.neoil")
    ))
    .unwrap();
    let source = format!(
        ".module App\n.entry Main\n.function Main() -> Int32\n.local Void* allocation\n{body}\n.end"
    );
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        &library,
    )
    .unwrap()
    .remove(0);
    run_with_library(&app, &library, Limits::default())
}

#[test]
fn native_allocation_supports_typed_access_and_explicit_release() {
    let result = run("ldc.i4 1\nconv.u\nldc.i4 4\nconv.u\ncall System.Runtime.InteropServices.NativeMemory::Alloc(UIntPtr,UIntPtr)\nstloc allocation\nldloc allocation\nptr.cast Int32\nldc.i4 42\nstobj Int32\nldloc allocation\nptr.cast Int32\nldobj Int32\nldloc allocation\ncall System.Runtime.InteropServices.NativeMemory::Free(Void*)\nret").unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.memory.live_allocations(), 0);
    assert_eq!(result.heap.statistics().allocated_objects, 0);
}

#[test]
fn zero_allocations_and_null_release_are_supported() {
    assert_eq!(run("ldc.i4 0\nconv.u\ncall System.Runtime.InteropServices.NativeMemory::Alloc(UIntPtr)\ncall System.Runtime.InteropServices.NativeMemory::Free(Void*)\nptr.null Void\ncall System.Runtime.InteropServices.NativeMemory::Free(Void*)\nldc.i4 0\nret").unwrap().memory.live_allocations(), 0);
}

#[test]
fn native_allocation_preserves_bounds_lifetime_and_overflow_checks() {
    let prefix = "ldc.i4 4\nconv.u\ncall System.Runtime.InteropServices.NativeMemory::Alloc(UIntPtr)\nstloc allocation\n";
    for (body, expected) in [
        (
            "ldloc allocation\ncall System.Runtime.InteropServices.NativeMemory::Free(Void*)\nldloc allocation\ncall System.Runtime.InteropServices.NativeMemory::Free(Void*)\nldc.i4 0\nret",
            "double free",
        ),
        (
            "ldloc allocation\ncall System.Runtime.InteropServices.NativeMemory::Free(Void*)\nldloc allocation\nptr.cast Int32\nldobj Int32\nret",
            "use after free",
        ),
        (
            "ldloc allocation\nptr.cast Int32\nldc.i4 4\nptr.add\nldobj Int32\nret",
            "bounds",
        ),
    ] {
        let fault = run(&format!("{prefix}{body}")).unwrap_err();
        assert!(fault.message.contains(expected), "{fault:?}");
    }
    let fault = run("ldc.i8 -1\nconv.u\nldc.i4 2\nconv.u\ncall System.Runtime.InteropServices.NativeMemory::Alloc(UIntPtr,UIntPtr)\npop\nldc.i4 0\nret").unwrap_err();
    assert!(fault.message.contains("overflow"), "{fault:?}");
}
