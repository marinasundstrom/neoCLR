use neoclr::{Limits, LoadedProgram, Value, assemble, load, metadata::Type, run};

const SAMPLE: &str = include_str!("../examples/pointer_union.neoil");

fn program(body: &str, returns: &str, extra: &str) -> neoclr::Module {
    let source = format!(
        "{}\n.function Probe() -> {returns}\n{body}\nret\n.end\n{extra}",
        SAMPLE.replace(".entry Main", ".entry Probe")
    );
    let module = assemble(&source).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&module).unwrap().verify().unwrap();
    module
}

#[test]
fn borrowed_carriers_alias_heap_and_stack_storage_without_copying_payloads() {
    let module = assemble(SAMPLE).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&module).unwrap().verify().unwrap();
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(result.output, ["42", "7", "11"]);
    assert_eq!(result.value, Value::Void);
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn wrong_case_faults_before_accessing_the_payload_even_when_types_match() {
    for (case, accessor, message) in [
        ("FromOk", "GetError", "does not contain Error"),
        ("FromError", "GetOk", "does not contain Ok"),
    ] {
        let body = format!(
            "ptr.null Int32\ncall PointerResult<Int32,Int32>::{case}(Int32*)\ncall instance PointerResult<Int32,Int32>::{accessor}()"
        );
        let fault = run(&program(&body, "Int32", ""), Limits::default()).unwrap_err();
        assert!(fault.message.contains(message), "{fault}");
        assert!(fault.stack_trace.unwrap().frames.len() >= 2);
    }
}

#[test]
fn erased_pointer_views_keep_heap_and_frame_lifetime_checks() {
    let heap = ".local Int32* owner\n.local PointerResult<Int32,Int32> view\nldc.i4 1\nheap.alloc Int32\nstloc owner\nldloc owner\nldc.i4 42\nstobj Int32\nldloc owner\ncall PointerResult<Int32,Int32>::FromOk(Int32*)\nstloc view\nldloc owner\nptr.cast Void\nheap.free\npop\nldloc view\ncall instance PointerResult<Int32,Int32>::GetOk()";
    let escape = ".function Escape() -> PointerResult<Int32,Int32>\nsizeof Int32\nlocalloc\nptr.cast Int32\ndup\nldc.i4 42\nstobj Int32\ncall PointerResult<Int32,Int32>::FromOk(Int32*)\nret\n.end";
    for (body, extra) in [
        (heap, ""),
        (
            "call Escape()\ncall instance PointerResult<Int32,Int32>::GetOk()",
            escape,
        ),
    ] {
        let fault = run(&program(body, "Int32", extra), Limits::default()).unwrap_err();
        assert!(fault.message.contains("use after free"), "{fault}");
    }
}

#[test]
fn native_carrier_storage_preserves_erased_pointer_aliases() {
    let body = ".local Int32* owner\n.local PointerResult<Int32,Int32>* slot\nldc.i4 1\nheap.alloc Int32\nstloc owner\nldloc owner\nldc.i4 42\nstobj Int32\nldc.i4 1\nheap.alloc PointerResult<Int32,Int32>\nstloc slot\nldloc slot\nldloc owner\ncall PointerResult<Int32,Int32>::FromOk(Int32*)\nstobj PointerResult<Int32,Int32>\nldloc slot\nldobj PointerResult<Int32,Int32>\ncall instance PointerResult<Int32,Int32>::GetOk()\nldloc slot\nheap.free\npop\nldloc owner\nheap.free\npop";
    let execution = run(&program(body, "Int32", ""), Limits::default()).unwrap();
    assert_eq!(execution.value, Value::Int32(42));
    assert_eq!(execution.memory.live_allocations(), 0);
}

#[test]
fn void_and_byte_payloads_use_existing_native_storage_rules() {
    for (ty, value, expected) in [
        ("Void", "ldvoid", Value::Void),
        ("Byte", "ldc.i4 257", Value::Byte(1)),
    ] {
        let body = format!(
            ".local {ty}* owner\nldc.i4 1\nheap.alloc {ty}\nstloc owner\nldloc owner\n{value}\nstobj {ty}\nldloc owner\ncall PointerResult<{ty},Int32>::FromOk({ty}*)\ncall instance PointerResult<{ty},Int32>::GetOk()\nldloc owner\nheap.free\npop"
        );
        let execution = run(&program(&body, ty, ""), Limits::default()).unwrap();
        assert_eq!(execution.value, expected);
        assert_eq!(execution.memory.live_allocations(), 0);
    }
}

#[test]
fn void_pointer_layout_does_not_require_a_native_layout_for_its_target() {
    let module = assemble(SAMPLE).unwrap();
    let void_ptr = Type::Ptr(Box::new(Type::Void));
    let string_ptr = Type::Ptr(Box::new(Type::String));
    assert_eq!(
        neoclr::memory::layout(&module, &void_ptr).unwrap().size,
        neoclr::memory::layout(&module, &string_ptr).unwrap().size
    );
    assert!(neoclr::memory::layout(&module, &Type::String).is_err());
}

#[test]
fn try_get_copies_both_cases_with_exact_storage_types() {
    for case in ["Ok", "Error"] {
        for (ty, value, expected) in [
            ("Int32", "ldc.i4 42", Value::Int32(42)),
            ("Byte", "ldc.i4 257", Value::Byte(1)),
            ("Void", "ldvoid", Value::Void),
        ] {
            let body = format!(
                ".local {ty}* payload\n.local {ty}* output\nsizeof {ty}\nlocalloc\nptr.cast {ty}\nstloc payload\nsizeof {ty}\nlocalloc\nptr.cast {ty}\nstloc output\nldloc payload\n{value}\nstobj {ty}\nldloc payload\ncall PointerResult<{ty},{ty}>::From{case}({ty}*)\nldloc output\ncall instance PointerResult<{ty},{ty}>::TryGet{case}({ty}*)\nbrtrue Copied\nfault \"expected match\"\nCopied:\nldloc output\nldobj {ty}"
            );
            assert_eq!(
                run(&program(&body, ty, ""), Limits::default())
                    .unwrap()
                    .value,
                expected
            );
        }
    }
}

#[test]
fn mismatch_copy_leaves_output_untouched_and_never_reads_the_payload() {
    for (case, other) in [("Ok", "Error"), ("Error", "Ok")] {
        let prefix = format!(
            ".local Int32* output\nsizeof Int32\nlocalloc\nptr.cast Int32\nstloc output\nldloc output\nldc.i4 99\nstobj Int32\nptr.null Int32\ncall PointerResult<Int32,Int32>::From{case}(Int32*)\nldloc output\ncall instance PointerResult<Int32,Int32>::TryGet{other}(Int32*)\nbrfalse Miss\nfault \"unexpected match\"\nMiss:\nldloc output\nldobj Int32"
        );
        assert_eq!(
            run(&program(&prefix, "Int32", ""), Limits::default())
                .unwrap()
                .value,
            Value::Int32(99)
        );
        let uninitialized = prefix.replace("ldloc output\nldc.i4 99\nstobj Int32\n", "");
        let fault = run(&program(&uninitialized, "Int32", ""), Limits::default()).unwrap_err();
        assert!(fault.message.contains("uninitialized"), "{fault}");
        let null_output = format!(
            "ptr.null Int32\ncall PointerResult<Int32,Int32>::From{case}(Int32*)\nptr.null Int32\ncall instance PointerResult<Int32,Int32>::TryGet{other}(Int32*)"
        );
        assert_eq!(
            run(&program(&null_output, "Boolean", ""), Limits::default())
                .unwrap()
                .value,
            Value::Boolean(false)
        );
    }
}

#[test]
fn mismatch_borrow_clears_a_previous_pointer_and_checks_output_storage() {
    for (case, other) in [("Ok", "Error"), ("Error", "Ok")] {
        let body = format!(
            ".local Int32** output\n.local Int32* previous\nsizeof Int32*\nlocalloc\nptr.cast Int32*\nstloc output\nsizeof Int32\nlocalloc\nptr.cast Int32\nstloc previous\nldloc output\nldloc previous\nstobj Int32*\nptr.null Int32\ncall PointerResult<Int32,Int32>::From{case}(Int32*)\nldloc output\ncall instance PointerResult<Int32,Int32>::TryGet{other}Pointer(Int32**)\nbrfalse Miss\nfault \"unexpected match\"\nMiss:\nldloc output\nldobj Int32*\nptr.null Int32\nceq"
        );
        assert_eq!(
            run(&program(&body, "Boolean", ""), Limits::default())
                .unwrap()
                .value,
            Value::Boolean(true)
        );
        let null_output = format!(
            "ptr.null Int32\ncall PointerResult<Int32,Int32>::From{case}(Int32*)\nptr.null Int32*\ncall instance PointerResult<Int32,Int32>::TryGet{other}Pointer(Int32**)"
        );
        let fault = run(&program(&null_output, "Boolean", ""), Limits::default()).unwrap_err();
        assert!(fault.message.contains("null"), "{fault}");
    }
}

#[test]
fn borrowed_output_aliases_the_selected_payload_for_both_cases() {
    for case in ["Ok", "Error"] {
        let body = format!(
            ".local Int32* payload\n.local Int32** output\nsizeof Int32\nlocalloc\nptr.cast Int32\nstloc payload\nldloc payload\nldc.i4 42\nstobj Int32\nsizeof Int32*\nlocalloc\nptr.cast Int32*\nstloc output\nldloc payload\ncall PointerResult<Int32,Int32>::From{case}(Int32*)\nldloc output\ncall instance PointerResult<Int32,Int32>::TryGet{case}Pointer(Int32**)\nbrtrue Matched\nfault \"expected match\"\nMatched:\nldloc output\nldobj Int32*\nldc.i4 7\nstobj Int32\nldloc payload\nldobj Int32"
        );
        assert_eq!(
            run(&program(&body, "Int32", ""), Limits::default())
                .unwrap()
                .value,
            Value::Int32(7)
        );
    }
}

#[test]
fn borrowing_a_stale_payload_does_not_validate_or_extend_its_lifetime() {
    let extra = ".function Expired() -> PointerResult<Int32,Int32>\nsizeof Int32\nlocalloc\nptr.cast Int32\ncall PointerResult<Int32,Int32>::FromOk(Int32*)\nret\n.end";
    let prefix = ".local Int32** output\nsizeof Int32*\nlocalloc\nptr.cast Int32*\nstloc output\ncall Expired()\nldloc output\ncall instance PointerResult<Int32,Int32>::TryGetOkPointer(Int32**)";
    assert_eq!(
        run(&program(prefix, "Boolean", extra), Limits::default())
            .unwrap()
            .value,
        Value::Boolean(true)
    );
    let body = format!("{prefix}\npop\nldloc output\nldobj Int32*\nldobj Int32");
    let fault = run(&program(&body, "Int32", extra), Limits::default()).unwrap_err();
    assert!(fault.message.contains("use after free"), "{fault}");
}

#[test]
fn matching_copy_requires_valid_payload_and_output_storage() {
    for body in [
        ".local Int32* output\nsizeof Int32\nlocalloc\nptr.cast Int32\nstloc output\nptr.null Int32\ncall PointerResult<Int32,Int32>::FromOk(Int32*)\nldloc output\ncall instance PointerResult<Int32,Int32>::TryGetOk(Int32*)",
        "sizeof Int32\nlocalloc\nptr.cast Int32\ndup\nldc.i4 42\nstobj Int32\ncall PointerResult<Int32,Int32>::FromOk(Int32*)\nptr.null Int32\ncall instance PointerResult<Int32,Int32>::TryGetOk(Int32*)",
    ] {
        let fault = run(&program(body, "Boolean", ""), Limits::default()).unwrap_err();
        assert!(fault.message.contains("null"), "{fault}");
    }
}
