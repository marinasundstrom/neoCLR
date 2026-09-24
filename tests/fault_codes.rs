use neoclr::{FaultCode, Limits, LoadedProgram, assemble};

fn failure(body: &str, result: &str, limits: Limits) -> neoclr::Fault {
    let module = assemble(&format!(
        ".module Codes\n.entry Main\n.function Main() -> {result}\n{body}\nret\n.end"
    ))
    .unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(limits)
        .unwrap_err()
}

#[test]
fn arithmetic_and_explicit_faults_have_independent_codes() {
    for (body, result, code) in [
        ("ldc.i4 1\nldc.i4 0\ndiv", "Int32", FaultCode::DivideByZero),
        ("ldc.i4 1\nldc.i4 0\nrem", "Int32", FaultCode::DivideByZero),
        (
            "ldc.i4 2147483647\nldc.i4 1\nadd.ovf",
            "Int32",
            FaultCode::ArithmeticOverflow,
        ),
        (
            "ldc.i4 -2147483648\nldc.i4 -1\ndiv",
            "Int32",
            FaultCode::ArithmeticOverflow,
        ),
        (
            "ldc.i4 256\nconv.ovf.u1",
            "Byte",
            FaultCode::ArithmeticOverflow,
        ),
        ("fault \"StackOverflow\"", "Int32", FaultCode::UserFault),
        ("fault \"division by zero\"", "Int32", FaultCode::UserFault),
    ] {
        let fault = failure(body, result, Limits::default());
        assert_eq!(fault.code, code, "{fault}");
        assert!(fault.stack_trace.is_some());
        assert!(fault.to_string().contains(&format!("[code={code}]")));
    }
}

#[test]
fn resource_and_memory_failures_have_specific_codes() {
    let array = "ldc.i4 2\nnewarr Int32\nldlen";
    for (limits, expected) in [
        (
            Limits {
                frames: 0,
                ..Limits::default()
            },
            FaultCode::StackOverflow,
        ),
        (
            Limits {
                instructions: 0,
                ..Limits::default()
            },
            FaultCode::InstructionLimitExceeded,
        ),
        (
            Limits {
                stack: 0,
                ..Limits::default()
            },
            FaultCode::EvaluationStackOverflow,
        ),
        (
            Limits {
                heap_objects: 0,
                ..Limits::default()
            },
            FaultCode::HeapLimitExceeded,
        ),
        (
            Limits {
                array_elements: 1,
                ..Limits::default()
            },
            FaultCode::ArrayLimitExceeded,
        ),
    ] {
        assert_eq!(failure(array, "UIntPtr", limits).code, expected);
    }
    assert_eq!(
        failure(
            "ldc.i4 1\nheap.alloc Int32",
            "Int32*",
            Limits {
                pointer_bytes: 0,
                ..Limits::default()
            }
        )
        .code,
        FaultCode::NativeMemoryLimitExceeded
    );
    assert_eq!(
        failure("ptr.null Int32\nldind.i4", "Int32", Limits::default()).code,
        FaultCode::NullPointer
    );
    assert_eq!(failure(".local arrayref<Int32> bytes\nldloca bytes\ninitobj arrayref<Int32>\nldloc bytes\nldlen", "UIntPtr", Limits::default()).code, FaultCode::NullReference);
    assert_eq!(
        failure(
            "ldc.i4 1\nnewarr Int32\nldc.i4 1\nldelem Int32",
            "Int32",
            Limits::default()
        )
        .code,
        FaultCode::IndexOutOfRange
    );
}

#[test]
fn verifier_and_debugger_launch_failures_keep_codes_without_execution() {
    let module = assemble(".module Invalid\n.function Bad() -> Int32\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let fault = program.verify().unwrap_err();
    assert_eq!(fault.code, FaultCode::InvalidProgram);
    assert!(fault.stack_trace.is_none());
    // The VM also publishes codes when its root-frame check fails before a pause.
    let root_debugger = neoclr::debugger::Debugger::new();
    let valid =
        assemble(".module Root\n.entry Main\n.function Main() -> Int32\nldc.i4 1\nret\n.end")
            .unwrap();
    let root_fault = LoadedProgram::new(&valid)
        .unwrap()
        .run(neoclr::ExecutionOptions {
            debugger: Some(root_debugger.clone()),
            limits: Limits {
                frames: 0,
                ..Limits::default()
            },
            ..neoclr::ExecutionOptions::default()
        })
        .unwrap_err();
    assert_eq!(root_fault.code, FaultCode::StackOverflow);
    assert_eq!(
        root_debugger.snapshot().fault_code,
        Some(FaultCode::StackOverflow)
    );

    let debugger = neoclr::debugger::Debugger::new();
    debugger.launch_failure(&fault);
    let snapshot = debugger.snapshot();
    assert_eq!(snapshot.fault_code, Some(FaultCode::InvalidProgram));
    assert_eq!(
        serde_json::to_value(snapshot).unwrap()["fault_code"],
        "InvalidProgram"
    );
    assert_eq!(
        serde_json::to_value(neoclr::debugger::Debugger::new().snapshot()).unwrap()["fault_code"],
        serde_json::Value::Null
    );
}

#[test]
fn symbolic_code_serialization_is_stable_and_has_no_numeric_ordinal_contract() {
    let codes = [
        (FaultCode::InternPoolLimitExceeded, "InternPoolLimitExceeded"),
        (FaultCode::RuntimeError, "RuntimeError"),
        (FaultCode::UserFault, "UserFault"),
        (FaultCode::InvalidProgram, "InvalidProgram"),
        (FaultCode::StackOverflow, "StackOverflow"),
        (
            FaultCode::EvaluationStackOverflow,
            "EvaluationStackOverflow",
        ),
        (
            FaultCode::InstructionLimitExceeded,
            "InstructionLimitExceeded",
        ),
        (FaultCode::ExecutionCancelled, "ExecutionCancelled"),
        (FaultCode::NullReference, "NullReference"),
        (FaultCode::NullPointer, "NullPointer"),
        (FaultCode::IndexOutOfRange, "IndexOutOfRange"),
        (FaultCode::ArithmeticOverflow, "ArithmeticOverflow"),
        (FaultCode::DivideByZero, "DivideByZero"),
        (FaultCode::HeapLimitExceeded, "HeapLimitExceeded"),
        (FaultCode::ArrayLimitExceeded, "ArrayLimitExceeded"),
        (
            FaultCode::NativeMemoryLimitExceeded,
            "NativeMemoryLimitExceeded",
        ),
    ];
    for (code, spelling) in codes {
        assert_eq!(code.as_str(), spelling);
        assert_eq!(serde_json::to_value(code).unwrap(), spelling);
    }
}
