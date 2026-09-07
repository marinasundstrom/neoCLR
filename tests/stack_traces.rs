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
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].location,
        CodeLocation::IlInstruction(0)
    );
    let module =
        assemble(".module App\n.entry Main\n.function Main() -> Void\nldvoid\npop\n.end").unwrap();
    let fault = neoclr::run(&module, Limits::default()).unwrap_err();
    assert!(fault.message.contains("fell through"));
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
