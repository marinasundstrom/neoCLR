use neoclr::{
    CancellationToken, ExecutionOptions, Limits, LoadedProgram, Value, assemble,
    assembler::parse_function_ref,
};

fn cancelled() -> ExecutionOptions {
    let token = CancellationToken::new();
    token.cancel();
    ExecutionOptions {
        cancellation: Some(token),
        ..ExecutionOptions::default()
    }
}
fn assert_cancelled(fault: neoclr::Fault) {
    assert_eq!(fault.message, "execution cancelled");
    assert!(fault.function.is_some());
    assert!(fault.instruction.is_some());
}

#[test]
fn token_clones_share_a_monotonic_request_and_new_tokens_are_independent() {
    let token = CancellationToken::default();
    let clone = token.clone();
    assert!(!clone.is_cancelled());
    token.cancel();
    clone.cancel();
    assert!(token.is_cancelled());
    assert!(clone.is_cancelled());
    assert!(!CancellationToken::new().is_cancelled());
}

#[test]
fn entry_helpers_observe_precancellation_and_program_can_be_reused() {
    let module =
        assemble(".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 42\nret\n.end")
            .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let options = cancelled();
    let fault = program.run(options.clone()).unwrap_err();
    assert_eq!(fault.function.as_deref(), Some("Main"));
    assert_eq!(fault.instruction, Some(0));
    assert_cancelled(fault);
    assert_cancelled(neoclr::run(&module, options.clone()).unwrap_err());
    assert_cancelled(
        neoclr::run_with_library(&module, neoclr::library::system().unwrap(), options.clone())
            .unwrap_err(),
    );
    // No native declarations exist; native-enabled entry paths use the same controls.
    unsafe {
        assert_cancelled(program.run_with_native(options.clone()).unwrap_err());
        assert_cancelled(
            neoclr::run_with_native(&module, neoclr::library::system().unwrap(), options)
                .unwrap_err(),
        );
    }
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert_eq!(
        program.run(ExecutionOptions::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn resolved_static_and_instance_methods_share_controls_in_both_native_modes() {
    let module = assemble(".module App\n.function Echo(Int32 value) -> Int32\nldarg value\nret\n.end\n.type Point\n.method instance Echo() -> Point\nldarg this\nret\n.end\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let function = program
        .resolve_function(&parse_function_ref("Echo(Int32)").unwrap())
        .unwrap();
    let method = program
        .resolve_function(&parse_function_ref("instance Point::Echo()").unwrap())
        .unwrap();
    let receiver = Value::Object {
        ty: neoclr::metadata::Type::Named("Point".into()),
        fields: vec![],
    };
    assert_cancelled(
        function
            .invoke(vec![Value::Int32(1)], cancelled())
            .unwrap_err(),
    );
    assert_cancelled(
        method
            .invoke_instance(receiver.clone(), vec![], cancelled())
            .unwrap_err(),
    );
    unsafe {
        assert_cancelled(
            function
                .invoke_with_native(vec![Value::Int32(1)], cancelled())
                .unwrap_err(),
        );
        assert_cancelled(
            method
                .invoke_instance_with_native(receiver.clone(), vec![], cancelled())
                .unwrap_err(),
        );
    }
    assert_eq!(
        method
            .invoke_instance(receiver.clone(), vec![], Limits::default())
            .unwrap()
            .value,
        receiver
    );
}

#[test]
fn input_validation_precedes_cancellation_which_precedes_execution_limits() {
    let module =
        assemble(".module App\n.function Echo(Byte value) -> Byte\nldarg value\nret\n.end")
            .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let function = program
        .resolve_function(&parse_function_ref("Echo(Byte)").unwrap())
        .unwrap();
    let mut options = cancelled();
    options.limits.frames = 0;
    options.limits.instructions = 0;
    let fault = function
        .invoke(vec![Value::Int32(1)], options.clone())
        .unwrap_err();
    assert!(fault.message.starts_with("invocation argument 0:"));
    assert_eq!(fault.instruction, None);
    assert_cancelled(function.invoke(vec![Value::Byte(1)], options).unwrap_err());
    assert!(
        function
            .invoke(
                vec![Value::Byte(1)],
                Limits {
                    instructions: 0,
                    ..Limits::default()
                }
            )
            .unwrap_err()
            .message
            .contains("instruction limit")
    );
}

#[test]
fn cancellation_can_be_requested_from_another_thread() {
    let module = assemble(".module App\n.function Spin() -> Void\nLoop:\nbr Loop\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let function = program
        .resolve_function(&parse_function_ref("Spin()").unwrap())
        .unwrap();
    let token = CancellationToken::new();
    let options = ExecutionOptions {
        limits: Limits {
            instructions: 10_000_000,
            ..Limits::default()
        },
        cancellation: Some(token.clone()),
        ..ExecutionOptions::default()
    };
    // The instruction limit bounds this test even if cooperative polling regresses.
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| function.invoke(vec![], options).unwrap_err());
        std::thread::sleep(std::time::Duration::from_millis(10));
        token.cancel();
        let fault = worker.join().unwrap();
        assert_eq!(fault.function.as_deref(), Some("Spin"));
        assert_eq!(fault.instruction, Some(0));
        assert_cancelled(fault);
    });
    assert!(token.is_cancelled());
}

#[test]
fn uncancelled_options_preserve_guest_budgets_and_faults() {
    let module = assemble(".module App\n.function Echo() -> Void\nldvoid\nret\n.end\n.function Fail() -> Void\nfault \"guest failure\"\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo()").unwrap())
        .unwrap();
    let options = ExecutionOptions {
        limits: Limits {
            frames: 1,
            instructions: 2,
            ..Limits::default()
        },
        cancellation: Some(CancellationToken::new()),
        ..ExecutionOptions::default()
    };
    assert_eq!(
        echo.invoke(vec![], options.clone()).unwrap().value,
        Value::Void
    );
    let fail = program
        .resolve_function(&parse_function_ref("Fail()").unwrap())
        .unwrap();
    assert_eq!(
        fail.invoke(vec![], options).unwrap_err().message,
        "guest failure"
    );
}
