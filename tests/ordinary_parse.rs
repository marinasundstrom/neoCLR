use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
    metadata::Type,
};

#[test]
fn parse_returns_ordinary_results_for_boundaries_and_invalid_input() {
    let module = assemble(".module App").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let parse = program
        .resolve_function(&parse_function_ref("System.Int32::Parse(String)").unwrap())
        .unwrap();
    let is_ok = program
        .resolve_function(
            &parse_function_ref("instance System.Result<Int32,Error>::get_IsOkCase()").unwrap(),
        )
        .unwrap();
    let ok = program
        .resolve_function(
            &parse_function_ref("instance System.Result<Int32,Error>::GetOkCase()").unwrap(),
        )
        .unwrap();
    let err = program
        .resolve_function(
            &parse_function_ref("instance System.Result<Int32,Error>::GetErrorCase()").unwrap(),
        )
        .unwrap();
    let ok_value = program
        .resolve_function(
            &parse_function_ref("instance System.Result.Ok<Int32>::get_Value()").unwrap(),
        )
        .unwrap();
    let err_value = program
        .resolve_function(
            &parse_function_ref("instance System.Result.Error<Error>::get_Value()").unwrap(),
        )
        .unwrap();
    for (text, expected) in [
        ("2147483647", Some(i32::MAX)),
        ("-2147483648", Some(i32::MIN)),
        ("+42", Some(42)),
        ("", None),
        (" 1", None),
        ("1 ", None),
        ("2147483648", None),
        ("1x", None),
    ] {
        let value = parse
            .invoke(vec![Value::String(text.into())], Limits::default())
            .unwrap()
            .value;
        assert_eq!(
            value.ty(),
            Type::Constructed {
                definition: "System.Result".into(),
                arguments: vec![Type::Int32, Type::Error]
            }
        );
        assert_eq!(
            is_ok
                .invoke_instance(value.clone(), vec![], Limits::default())
                .unwrap()
                .value,
            Value::Boolean(expected.is_some())
        );
        let (case, getter, payload) = if let Some(n) = expected {
            (&ok, &ok_value, Value::Int32(n))
        } else {
            (&err, &err_value, Value::Error("InvalidInt32".into()))
        };
        let wrapper = case
            .invoke_instance(value, vec![], Limits::default())
            .unwrap()
            .value;
        assert_eq!(
            getter
                .invoke_instance(wrapper, vec![], Limits::default())
                .unwrap()
                .value,
            payload
        );
    }
}

#[test]
fn native_parse_protocol_carries_only_erased_primitive_payloads() {
    let module = assemble(".module App\n.function Raw(String input) -> System.Value\nldarg input\ncall neoCLR.Runtime.ParseInt32(String)\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let native = program
        .resolve_function(&parse_function_ref("Raw(String)").unwrap())
        .unwrap();
    for (text, expected) in [
        ("42", Value::Int32(42)),
        ("bad", Value::Error("InvalidInt32".into())),
    ] {
        assert_eq!(
            native
                .invoke(vec![Value::String(text.into())], Limits::default())
                .unwrap()
                .value,
            Value::Erased(Box::new(expected))
        );
    }
    let mut library = neoclr::library::system().unwrap().clone();
    library
        .functions
        .iter_mut()
        .find(|f| f.name == "neoCLR.Runtime.ParseInt32")
        .unwrap()
        .returns = Type::Result(Box::new(Type::Int32), Box::new(Type::Error));
    assert!(
        LoadedProgram::with_library(&module, &library)
            .unwrap_err()
            .message
            .contains("runtime binding return type mismatch")
    );
}

#[test]
fn old_union_extraction_is_not_an_implicit_compatibility_bridge() {
    let module = assemble(".module App\n.entry Main\n.function Main() -> Int32\nldstr \"42\"\ncall System.Int32::Parse(String)\nldcase Ok\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_err());
}

#[test]
fn parse_reachability_includes_platform_construction_and_storage_services() {
    let module = assemble(".module App").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("System.Int32::Parse(String)").unwrap()],
            16,
        )
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ParseInt32)
    );
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ValueStorage)
    );
    assert!(
        !graph
            .required_services()
            .contains(&RuntimeService::NativeAllocation)
    );
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "System.Result..ctor")
    );
}
