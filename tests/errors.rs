use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

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
            "InvalidFormat",
            "Expected a positive number: -1",
            "Execution continued"
        ]
    );
}

#[test]
fn result_error_payloads_can_be_imported_and_formatted_without_faults() {
    let program =
        LoadedProgram::new(&assemble(include_str!("../examples/errors.neoil")).unwrap()).unwrap();
    let read = program
        .resolve_function(&parse_function_ref("ReadPositive(String)").unwrap())
        .unwrap();
    let report = program
        .resolve_function(&parse_function_ref("Report(System.Result<Int32,String>)").unwrap())
        .unwrap();
    let result = read
        .invoke(vec![Value::String("0".into())], Limits::default())
        .unwrap()
        .value;
    let get_error = program
        .resolve_function(
            &parse_function_ref("instance System.Result<Int32,String>::GetErrorCase()").unwrap(),
        )
        .unwrap();
    let get_value = program
        .resolve_function(
            &parse_function_ref("instance System.Result.Error<String>::get_Value()").unwrap(),
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
        Value::String("Expected a positive number: 0".into())
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
fn legacy_message_error_api_and_instruction_are_retired() {
    let program = LoadedProgram::new(&assemble(".module App").unwrap()).unwrap();
    assert!(
        neoclr::library::system()
            .unwrap()
            .types
            .iter()
            .all(|ty| ty.name != "System.Error")
    );
    assert!(
        program
            .resolve_function(&parse_function_ref("System.Error::FromMessage(String)").unwrap())
            .is_err()
    );
    assert!(assemble(".module App\n.function Main() -> Void\nerror \"old\"\nret\n.end").is_err());
}
