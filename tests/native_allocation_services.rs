use neoclr::{
    FaultCode, Limits, LoadedProgram, RuntimeService, Value, assemble,
    assembler::parse_function_ref,
};

const SERVICES: &str = r#".module System
.function neoCLR.Runtime.NativeAllocate(UIntPtr count) -> Void*
.methodimpl InternalCall
.end
.function neoCLR.Runtime.NativeFree(Void* pointer) -> noresult
.methodimpl InternalCall
.end
.function neoCLR.Runtime.NativeMultiplyChecked(UIntPtr count,UIntPtr size) -> UIntPtr
.methodimpl InternalCall
.end
"#;

fn program(body: &str) -> LoadedProgram {
    let module = assemble(&format!(
        "{SERVICES}\n.function Test() -> Int32\n{body}\nret\n.end\n"
    ))
    .unwrap();
    let image = neoclr::metadata_container::write_module(&module).unwrap();
    let module = neoclr::metadata_container::decode_envelope(&image).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

fn run(body: &str, limits: Limits) -> Result<neoclr::Execution, neoclr::Fault> {
    program(body)
        .resolve_function(&parse_function_ref("Test()").unwrap())
        .unwrap()
        .invoke(vec![], limits)
}

#[test]
fn services_preserve_memory_identity_write_read_and_release() {
    let result = run(
        r#"
.local Void* memory
ldc.i4 2
conv.u
ldc.i4 3
conv.u
call neoCLR.Runtime.NativeMultiplyChecked(UIntPtr,UIntPtr)
call neoCLR.Runtime.NativeAllocate(UIntPtr)
stloc 0
ldloc 0
ptr.cast Byte
ldc.i4 42
stind.i1
ldloc 0
ptr.cast Byte
ldind.u1
ldloc 0
call neoCLR.Runtime.NativeFree(Void*)
"#,
        Limits::default(),
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.memory.live_allocations(), 0);
    assert_eq!(result.memory.live_bytes(), 0);
}

#[test]
fn allocation_limits_and_size_overflow_are_reported() {
    let body = "ldc.i4 8\nconv.u\ncall neoCLR.Runtime.NativeAllocate(UIntPtr)\npop\nldc.i4 42";
    for limits in [
        Limits {
            pointer_bytes: 7,
            ..Limits::default()
        },
        Limits {
            pointer_allocations: 0,
            ..Limits::default()
        },
    ] {
        assert_eq!(
            run(body, limits).unwrap_err().code,
            FaultCode::NativeMemoryLimitExceeded
        );
    }
    let overflow = "ldc.i8 -1\nconv.u\nldc.i4 2\nconv.u\ncall neoCLR.Runtime.NativeMultiplyChecked(UIntPtr,UIntPtr)\npop\nldc.i4 42";
    assert_eq!(
        run(overflow, Limits::default()).unwrap_err().code,
        FaultCode::ArithmeticOverflow
    );
}

#[test]
fn null_zero_length_and_double_free_keep_existing_rules() {
    run("ptr.null Void\ncall neoCLR.Runtime.NativeFree(Void*)\nldc.i4 0\nconv.u\ncall neoCLR.Runtime.NativeAllocate(UIntPtr)\ncall neoCLR.Runtime.NativeFree(Void*)\nldc.i4 42", Limits::default()).unwrap();
    let duplicate = "ldc.i4 1\nconv.u\ncall neoCLR.Runtime.NativeAllocate(UIntPtr)\ndup\ncall neoCLR.Runtime.NativeFree(Void*)\ncall neoCLR.Runtime.NativeFree(Void*)\nldc.i4 42";
    assert!(
        run(duplicate, Limits::default())
            .unwrap_err()
            .message
            .contains("double free")
    );
}

#[test]
fn exact_signatures_and_logical_services_are_preserved() {
    for invalid in [
        SERVICES.replace("NativeAllocate(UIntPtr", "NativeAllocate(Int32"),
        SERVICES.replace(
            "NativeFree(Void* pointer) -> noresult",
            "NativeFree(Void* pointer) -> Void",
        ),
        SERVICES.replace(
            "NativeMultiplyChecked(UIntPtr count,UIntPtr size) -> UIntPtr",
            "NativeMultiplyChecked(UIntPtr count,UIntPtr size) -> IntPtr",
        ),
    ] {
        assert!(assemble(&invalid).is_err());
    }
    let graph = program("ldc.i4 1\nconv.u\ncall neoCLR.Runtime.NativeAllocate(UIntPtr)\ncall neoCLR.Runtime.NativeFree(Void*)\nldc.i4 42")
        .analyze_reachability(&[parse_function_ref("Test()").unwrap()], 32).unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::NativeAllocation)
    );
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::PointerMemory)
    );
}

#[test]
fn services_and_instructions_share_the_same_allocation_heap() {
    for body in [
        "ldc.i4 1\nheap.alloc Byte\nptr.cast Void\ncall neoCLR.Runtime.NativeFree(Void*)\nldc.i4 42",
        "ldc.i4 1\nconv.u\ncall neoCLR.Runtime.NativeAllocate(UIntPtr)\nheap.free\npop\nldc.i4 42",
    ] {
        let result = run(body, Limits::default()).unwrap();
        assert_eq!(result.value, Value::Int32(42));
        assert_eq!(result.memory.live_bytes(), 0);
        assert_eq!(result.memory.live_allocations(), 0);
    }
    let unread =
        "ldc.i4 1\nconv.u\ncall neoCLR.Runtime.NativeAllocate(UIntPtr)\nptr.cast Byte\nldind.u1";
    assert!(
        run(unread, Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized")
    );
}
