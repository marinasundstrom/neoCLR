use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
    metadata::Type,
};

#[test]
fn parse_returns_typed_cases_for_boundaries_format_and_overflow() {
    let module = assemble(".module App").unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let resolve = |name: &str| {
        program
            .resolve_function(&parse_function_ref(name).unwrap())
            .unwrap()
    };
    let parse = resolve("System.Int32::Parse(String)");
    let ok = resolve("instance System.Result<Int32,System.Int32ParseError>::GetOkCase()");
    let err = resolve("instance System.Result<Int32,System.Int32ParseError>::GetErrorCase()");
    let ok_value = resolve("instance System.Result.Ok<Int32>::get_Value()");
    let err_value = resolve("instance System.Result.Error<System.Int32ParseError>::get_Value()");
    for (text, expected) in [
        ("2147483647", i32::MAX),
        ("-2147483648", i32::MIN),
        ("+42", 42),
        ("-0", 0),
        ("000000000000000000000000000000001", 1),
    ] {
        let value = parse
            .invoke(vec![Value::String(text.into())], Limits::default())
            .unwrap()
            .value;
        assert_eq!(
            value.ty(),
            neoclr::assembler::parse_type("System.Result<Int32,System.Int32ParseError>").unwrap()
        );
        let wrapper = ok
            .invoke_instance(value, vec![], Limits::default())
            .unwrap()
            .value;
        assert_eq!(
            ok_value
                .invoke_instance(wrapper, vec![], Limits::default())
                .unwrap()
                .value,
            Value::Int32(expected)
        );
    }
    for (text, case) in [
        ("", "InvalidFormat"),
        ("+", "InvalidFormat"),
        ("-", "InvalidFormat"),
        (" 1", "InvalidFormat"),
        ("1 ", "InvalidFormat"),
        ("1x", "InvalidFormat"),
        ("１２", "InvalidFormat"),
        ("1\0", "InvalidFormat"),
        ("2147483648", "Overflow"),
        ("-2147483649", "Overflow"),
        ("9999999999999999999999999999", "Overflow"),
        ("9999999999999999999999999999x", "InvalidFormat"),
    ] {
        let result = parse
            .invoke(vec![Value::String(text.into())], Limits::default())
            .unwrap()
            .value;
        let wrapper = err
            .invoke_instance(result, vec![], Limits::default())
            .unwrap()
            .value;
        let error = err_value
            .invoke_instance(wrapper, vec![], Limits::default())
            .unwrap()
            .value;
        let predicate = resolve(&format!("instance System.Int32ParseError::get_Is{case}()"));
        assert_eq!(
            predicate
                .invoke_instance(error.clone(), vec![], Limits::default())
                .unwrap()
                .value,
            Value::Boolean(true)
        );
        let get_case = resolve(&format!("instance System.Int32ParseError::Get{case}()"));
        let extracted = get_case
            .invoke_instance(error.clone(), vec![], Limits::default())
            .unwrap()
            .value;
        assert_eq!(
            extracted.ty(),
            neoclr::assembler::parse_type(&format!("System.Int32ParseError.{case}")).unwrap()
        );
        let other = if case == "Overflow" {
            "InvalidFormat"
        } else {
            "Overflow"
        };
        assert!(
            resolve(&format!("instance System.Int32ParseError::Get{other}()"))
                .invoke_instance(error.clone(), vec![], Limits::default())
                .is_err()
        );
        assert_eq!(
            resolve("instance System.Int32ParseError::ToString()")
                .invoke_instance(error, vec![], Limits::default())
                .unwrap()
                .value,
            Value::String(case.into())
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
        ("bad", Value::Byte(1)),
        ("2147483648", Value::Byte(2)),
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
