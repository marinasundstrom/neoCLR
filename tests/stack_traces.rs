use neoclr::{
    CodeLocation, ExecutionOptions, Limits, LoadedProgram, StackTrace, Value, assemble,
    assembler::parse_function_ref,
};

fn nested() -> neoclr::Module {
    assemble(".module App\n.revision r1\n.entry Main\n.function Main() -> Void\ncall Middle()\nret\n.end\n.function Middle() -> Void\nldvoid\npop\ncall Fail()\nret\n.end\n.function Fail() -> Void\nfault \"broken\"\n.end").unwrap()
}

#[test]
fn nested_faults_preserve_call_sites_and_owned_definition_identities() {
    let fault = {
        let module = nested();
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap_err()
    };
    assert_eq!(fault.message, "broken");
    assert_eq!(fault.code, neoclr::FaultCode::UserFault);
    let trace = fault.stack_trace.as_ref().unwrap();
    assert!(!trace.truncated);
    assert_eq!(
        trace
            .frames
            .iter()
            .map(|f| (f.function.name.as_str(), &f.location))
            .collect::<Vec<_>>(),
        [
            ("Fail", &CodeLocation::IlInstruction(0)),
            ("Middle", &CodeLocation::IlInstruction(2)),
            ("Main", &CodeLocation::IlInstruction(0)),
        ]
    );
    for frame in &trace.frames {
        let id = frame.function.definition.as_ref().unwrap();
        assert_eq!(id.module, "App");
        assert_eq!(id.revision.as_deref(), Some("r1"));
    }
    assert_eq!(
        trace.frames[0].function.definition.as_ref().unwrap().index,
        2
    );
    let formatted = fault.to_string();
    assert!(formatted.contains("[App#r1:2] IL instruction 0"));
    assert!(formatted.contains("Middle"));
    assert_eq!(fault.clone(), fault);
}

#[test]
fn generic_faults_preserve_the_bound_overload_and_closed_owner() {
    let source = include_str!("../examples/member-identities.neoil")
        .replace("ldstr \"generic declaration\"", "fault \"generic failure\"");
    let module = assemble(&source).unwrap();
    let fault = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    let frames = &fault.stack_trace.unwrap().frames;
    assert_eq!(frames[0].function.name, "Choice.Describe");
    assert_eq!(frames[0].function.definition.as_ref().unwrap().index, 0);
    assert_eq!(
        frames[0].function.owner,
        Some(neoclr::assembler::parse_type("Choice<Int32>").unwrap())
    );
    assert_eq!(
        frames[0].function.parameters,
        [neoclr::metadata::Type::Int32]
    );
    assert_eq!(frames[1].function.name, "Choice.Forward");
    assert_eq!(frames[2].function.name, "Main");
}

#[test]
fn recursive_frame_limits_bound_capture_and_mark_truncation() {
    let module =
        assemble(".module App\n.entry Main\n.function Main() -> Void\ncall Main()\nret\n.end")
            .unwrap();
    let fault = neoclr::run(
        &module,
        Limits {
            frames: 80,
            ..Limits::default()
        },
    )
    .unwrap_err();
    assert!(fault.message.contains("frame limit"));
    assert_eq!(fault.code, neoclr::FaultCode::StackOverflow);
    let trace = fault.stack_trace.as_ref().unwrap();
    assert_eq!(trace.frames.len(), StackTrace::MAX_FRAMES);
    assert!(trace.truncated);
    assert!(
        trace
            .frames
            .iter()
            .all(|f| f.location == CodeLocation::IlInstruction(0))
    );
    assert!(fault.to_string().contains("stack trace truncated"));
}

#[test]
fn instruction_and_stack_limits_and_fallthrough_have_active_stack_context() {
    let program = LoadedProgram::new(&nested()).unwrap();
    let fault = program
        .run(Limits {
            instructions: 2,
            ..Limits::default()
        })
        .unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::InstructionLimitExceeded);
    let trace = fault.stack_trace.unwrap();
    assert_eq!(trace.frames[0].function.name, "Middle");
    assert_eq!(trace.frames[0].location, CodeLocation::IlInstruction(1));
    assert_eq!(trace.frames[1].location, CodeLocation::IlInstruction(0));
    let fault = program
        .run(Limits {
            stack: 0,
            ..Limits::default()
        })
        .unwrap_err();
    assert!(fault.message.contains("evaluation stack limit"));
    assert_eq!(fault.code, neoclr::FaultCode::EvaluationStackOverflow);
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].location,
        CodeLocation::IlInstruction(0)
    );
    let module =
        assemble(".module App\n.entry Main\n.function Main() -> Void\nldvoid\npop\n.end").unwrap();
    let fault = neoclr::run(&module, Limits::default()).unwrap_err();
    assert!(fault.message.contains("fell through"));
    assert_eq!(fault.code, neoclr::FaultCode::InvalidProgram);
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].location,
        CodeLocation::IlInstruction(2)
    );
}

#[test]
fn precancellation_and_zero_limits_capture_the_requested_execution_root() {
    let program = LoadedProgram::new(&nested()).unwrap();
    let token = neoclr::CancellationToken::new();
    token.cancel();
    for options in [
        ExecutionOptions {
            cancellation: Some(token),
            ..ExecutionOptions::default()
        },
        Limits {
            frames: 0,
            ..Limits::default()
        }
        .into(),
        Limits {
            instructions: 0,
            ..Limits::default()
        }
        .into(),
    ] {
        let trace = program.run(options).unwrap_err().stack_trace.unwrap();
        assert_eq!(trace.frames.len(), 1);
        assert_eq!(trace.frames[0].function.name, "Main");
        assert_eq!(trace.frames[0].location, CodeLocation::IlInstruction(0));
    }
}

#[test]
fn loader_verifier_and_host_input_faults_do_not_fabricate_guest_traces() {
    assert!(assemble("invalid").unwrap_err().stack_trace.is_none());
    let module = assemble(".module App\n.function Echo(Int32 x) -> Int32\nldarg x\nret\n.end\n.function Bad() -> Void\nldc.i4 1\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().unwrap_err().stack_trace.is_none());
    let echo = program
        .resolve_function(&parse_function_ref("Echo(Int32)").unwrap())
        .unwrap();
    assert!(
        echo.invoke(vec![Value::Void], Limits::default())
            .unwrap_err()
            .stack_trace
            .is_none()
    );
    assert!(
        program
            .run(Limits::default())
            .unwrap_err()
            .stack_trace
            .is_none()
    );
    assert_eq!(
        echo.invoke(vec![Value::Int32(7)], Limits::default())
            .unwrap()
            .value,
        Value::Int32(7)
    );
    let bad = program
        .resolve_function(&parse_function_ref("Bad()").unwrap())
        .unwrap();
    assert_eq!(
        bad.invoke(vec![], Limits::default())
            .unwrap_err()
            .stack_trace
            .unwrap()
            .frames[0]
            .location,
        CodeLocation::IlInstruction(1)
    );
}

#[test]
fn native_boundary_faults_show_guest_callers_without_inventing_native_frames() {
    let module = assemble(".module App\n.entry Main\n.function Main() -> Void\ncall Foreign()\nret\n.end\n.function Foreign() -> Void\n.pinvoke \"nonexistent_stacktrace_fixture\" \"entry\" cdecl\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let safe = program.run(Limits::default()).unwrap_err();
    // Loading a nonexistent library executes no foreign code.
    let native = unsafe { program.run_with_native(Limits::default()) }.unwrap_err();
    for fault in [safe, native] {
        let frames = fault.stack_trace.unwrap().frames;
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].function.name, "Main");
        assert_eq!(frames[0].location, CodeLocation::IlInstruction(0));
    }
}

#[test]
fn guest_capture_is_owned_current_stack_and_does_not_fault() {
    let module = assemble(
        r#"
.module Capture
.entry Main
.function neoCLR.Runtime.StackTraceCapture(Int32 skip) -> String
.methodimpl InternalCall
.end
.function Main() -> String
call Capture()
ret
.end
.function Capture() -> String
ldc.i4 0
call neoCLR.Runtime.StackTraceCapture(Int32)
ret
.end
"#,
    )
    .unwrap();
    let execution = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    let Value::String(text) = execution.value else {
        panic!("expected owned text")
    };
    drop(module);
    assert_eq!(
        text.as_str(),
        "   at Capture [instruction 1]\n   at Main [instruction 0]\n"
    );
}

#[test]
fn guest_capture_checks_skip_and_marks_truncation() {
    let source = r#"
.module Capture
.entry Main
.function neoCLR.Runtime.StackTraceCapture(Int32 skip) -> String
.methodimpl InternalCall
.end
.function Main() -> String
ldc.i4 70
call Recurse(Int32)
ret
.end
.function Recurse(Int32 n) -> String
ldarg 0
ldc.i4 0
ceq
brtrue capture
ldarg 0
ldc.i4 1
sub
call Recurse(Int32)
ret
capture:
ldc.i4 0
call neoCLR.Runtime.StackTraceCapture(Int32)
ret
.end
"#;
    let module = assemble(source).unwrap();
    let result = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    let Value::String(text) = result.value else {
        panic!("expected text")
    };
    assert_eq!(text.as_str().matches("   at ").count(), 64);
    assert!(text.as_str().ends_with("   ... stack trace truncated\n"));
    let skipped = assemble(&source.replace("capture:\nldc.i4 0", "capture:\nldc.i4 1000")).unwrap();
    let result = LoadedProgram::new(&skipped)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.value, Value::String("".into()));
    let negative = assemble(&source.replace("capture:\nldc.i4 0", "capture:\nldc.i4 -1")).unwrap();
    let fault = LoadedProgram::new(&negative)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::RuntimeError);
    assert!(fault.stack_trace.is_some());
}

#[test]
fn native_diagnostic_names_decode_only_reserved_valid_transport_names() {
    use neoclr::diagnostic_function_name as name;
    let scope = format!("NeoMetadata_{}", "A".repeat(64));
    assert_eq!(
        name(&format!(
            "{scope}.F_3C4E656F46756E6374696F6E3E34313633364436355F353237353645"
        )),
        "Acme.Run"
    );
    assert_eq!(
        name(&format!("{scope}.T_53797374656D_5468696E67.M_52756E")),
        "System.Thing.Run"
    );
    for raw in [
        "App.Run",
        "NeoMetadata_A.F_123",
        "NeoMetadata_A.F_00",
        "NeoMetadata_A.F_FF",
    ] {
        assert_eq!(name(raw), raw);
    }
    let malformed = format!("{scope}.F_FF");
    assert_eq!(name(&malformed), malformed);
}
