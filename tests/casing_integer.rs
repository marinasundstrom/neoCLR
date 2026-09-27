use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
    metadata::Type,
};

#[test]
fn int64_native_protocol_and_exact_binding() {
    let module = assemble(".module App\n.function Raw(String input) -> System.Value\nldarg input\ncall neoCLR.Runtime.ParseInt64(String)\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let raw = program
        .resolve_function(&parse_function_ref("Raw(String)").unwrap())
        .unwrap();
    for (text, expected) in [
        ("9223372036854775807", Value::Int64(i64::MAX)),
        ("-9223372036854775808", Value::Int64(i64::MIN)),
        ("+42", Value::Int64(42)),
        ("-0", Value::Int64(0)),
        ("9223372036854775808", Value::Byte(2)),
        ("-9223372036854775809", Value::Byte(2)),
        ("9999999999999999999999999999x", Value::Byte(1)),
        (" 1", Value::Byte(1)),
        ("１２", Value::Byte(1)),
        ("", Value::Byte(1)),
    ] {
        assert_eq!(
            raw.invoke(vec![Value::String(text.into())], Limits::default())
                .unwrap()
                .value,
            Value::Erased(Box::new(expected))
        );
    }
    let graph = program
        .analyze_reachability(&[parse_function_ref("Raw(String)").unwrap()], 16)
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ParseInt64)
    );
    let mut library = neoclr::library::system().unwrap().clone();
    library
        .functions
        .iter_mut()
        .find(|f| f.name == "neoCLR.Runtime.ParseInt64")
        .unwrap()
        .returns = Type::Int64;
    assert!(
        LoadedProgram::with_library(&module, &library)
            .unwrap_err()
            .message
            .contains("runtime binding return type mismatch")
    );
}

#[test]
fn casing_native_binding_rejects_wrong_signature() {
    let module = assemble(".module App").unwrap();
    for name in [
        "neoCLR.Runtime.StringToUpperInvariant",
        "neoCLR.Runtime.StringToLowerInvariant",
    ] {
        let mut library = neoclr::library::system().unwrap().clone();
        library
            .functions
            .iter_mut()
            .find(|f| f.name == name)
            .unwrap()
            .returns = Type::Char;
        assert!(
            LoadedProgram::with_library(&module, &library)
                .unwrap_err()
                .message
                .contains("runtime binding return type mismatch")
        );
    }
}
