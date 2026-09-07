use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
};

#[test]
fn sample_handles_parse_and_domain_errors_and_continues() {
    let module = assemble(include_str!("../examples/errors.neoil")).unwrap();
    let program =
        LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
            .unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Void);
    assert_eq!(
        result.output,
        [
            "42",
            "InvalidInt32",
            "Expected a positive number: -1",
            "Execution continued"
        ]
    );
}

#[test]
fn owned_error_messages_preserve_empty_unicode_and_embedded_nul() {
    let program = LoadedProgram::new(&assemble(".module App").unwrap()).unwrap();
    let create = program
        .resolve_function(&parse_function_ref("System.Error::FromMessage(String)").unwrap())
        .unwrap();
    let message = program
        .resolve_function(&parse_function_ref("instance System.Error::get_Message()").unwrap())
        .unwrap();
    let format = program
        .resolve_function(&parse_function_ref("instance System.Error::ToString()").unwrap())
        .unwrap();
    for text in ["", "bad value", "é 🌍", "before\0after"] {
        let original = Value::String(text.into());
        let error = create
            .invoke(vec![original.clone()], Limits::default())
            .unwrap()
            .value;
        assert_eq!(error, Value::Error(text.into()));
        assert_eq!(
            message
                .invoke_instance(error.clone(), vec![], Limits::default())
                .unwrap()
                .value,
            original
        );
        assert_eq!(
            format
                .invoke_instance(error, vec![], Limits::default())
                .unwrap()
                .value,
            original
        );
    }
}

#[test]
fn result_error_payloads_can_be_imported_and_formatted_without_faults() {
    let program =
        LoadedProgram::new(&assemble(include_str!("../examples/errors.neoil")).unwrap()).unwrap();
    let read = program
        .resolve_function(&parse_function_ref("ReadPositive(String)").unwrap())
        .unwrap();
    let report = program
        .resolve_function(&parse_function_ref("Report(System.Result<Int32,Error>)").unwrap())
        .unwrap();
    let result = read
        .invoke(vec![Value::String("0".into())], Limits::default())
        .unwrap()
        .value;
    let get_error = program
        .resolve_function(
            &parse_function_ref("instance System.Result<Int32,Error>::GetErrorCase()").unwrap(),
        )
        .unwrap();
    let get_value = program
        .resolve_function(
            &parse_function_ref("instance System.Result.Error<Error>::get_Value()").unwrap(),
        )
        .unwrap();
    let wrapper = get_error
        .invoke_instance(result.clone(), vec![], Limits::default())
        .unwrap()
        .value;
    assert_eq!(
        get_value
            .invoke_instance(wrapper, vec![], Limits::default())
            .unwrap()
            .value,
        Value::Error("Expected a positive number: 0".into())
    );
    assert_eq!(
        report
            .invoke(vec![result], Limits::default())
            .unwrap()
            .output,
        ["Expected a positive number: 0"]
    );
    assert!(
        read.invoke(vec![Value::String("1".into())], Limits::default())
            .is_ok()
    );
}

#[test]
fn input_errors_and_execution_faults_stay_separate_from_error_values() {
    let program = LoadedProgram::new(&assemble(".module App").unwrap()).unwrap();
    let create = program
        .resolve_function(&parse_function_ref("System.Error::FromMessage(String)").unwrap())
        .unwrap();
    let invalid = create
        .invoke(vec![Value::Error("not a String".into())], Limits::default())
        .unwrap_err();
    assert!(invalid.stack_trace.is_none());
    let fault = create
        .invoke(
            vec![Value::String("message".into())],
            Limits {
                instructions: 0,
                ..Limits::default()
            },
        )
        .unwrap_err();
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].function.name,
        "System.Error.FromMessage"
    );
    assert_eq!(
        create
            .invoke(vec![Value::String("still valid".into())], Limits::default())
            .unwrap()
            .value,
        Value::Error("still valid".into())
    );
}

#[test]
fn error_methods_are_il_wrappers_over_declared_services() {
    let program = LoadedProgram::new(&assemble(".module App").unwrap()).unwrap();
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("instance System.Error::ToString()").unwrap()],
            3,
        )
        .unwrap();
    assert_eq!(
        graph
            .functions
            .iter()
            .map(|f| f.target.name.as_str())
            .collect::<Vec<_>>(),
        [
            "System.Error.ToString",
            "System.Error.get_Message",
            "neoCLR.Runtime.ErrorMessage"
        ]
    );
    assert_eq!(graph.required_services(), [RuntimeService::ErrorValues]);
    assert_eq!(graph.missing_services(&[])[0].instruction, None);
}
