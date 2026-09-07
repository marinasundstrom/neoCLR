use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{parse_function_ref, parse_type},
    metadata::{Case, Type},
};

fn union(ty: Type, case: Case, payload: Value) -> Value {
    Value::Union {
        ty,
        case,
        payload: Box::new(payload),
    }
}
fn echo_source(ty: &str) -> String {
    format!(".module App\n.function Echo({ty} value) -> {ty}\nldarg value\nret\n.end")
}

#[test]
fn all_bootstrap_cases_round_trip_with_exact_storage_values() {
    for (ty, cases) in [
        (
            "Option<Byte>",
            vec![(Case::Some, Value::Byte(7)), (Case::None, Value::Void)],
        ),
        (
            "Option<Void>",
            vec![(Case::Some, Value::Void), (Case::None, Value::Void)],
        ),
        (
            "Result<Void,Error>",
            vec![
                (Case::Ok, Value::Void),
                (Case::Err, Value::Error("failed".into())),
            ],
        ),
        (
            "Result<Int32,Int32>",
            vec![(Case::Ok, Value::Int32(42)), (Case::Err, Value::Int32(42))],
        ),
        (
            "Result<Single,Boolean>",
            vec![
                (Case::Ok, Value::Single(1.5)),
                (Case::Err, Value::Boolean(false)),
            ],
        ),
    ] {
        let module = assemble(&echo_source(ty)).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        program.verify().unwrap();
        let echo = program
            .resolve_function(&parse_function_ref(&format!("Echo({ty})")).unwrap())
            .unwrap();
        for (case, payload) in cases {
            let value = union(parse_type(ty).unwrap(), case, payload);
            let result = echo
                .invoke(
                    vec![value.clone()],
                    Limits {
                        frames: 1,
                        instructions: 2,
                        ..Limits::default()
                    },
                )
                .unwrap();
            assert_eq!(result.value, value);
            assert_eq!(
                echo.invoke(vec![result.value], Limits::default())
                    .unwrap()
                    .value,
                value
            );
        }
    }
}

#[test]
fn invalid_tags_cases_and_payloads_fail_before_guest_code_runs() {
    let module = assemble(&echo_source("Option<Byte>")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo(Option<Byte>)").unwrap())
        .unwrap();
    let ty = parse_type("Option<Byte>").unwrap();
    for value in [
        union(ty.clone(), Case::Ok, Value::Byte(7)),
        union(ty.clone(), Case::Some, Value::Int32(7)),
        union(ty.clone(), Case::None, Value::Byte(7)),
        union(
            ty.clone(),
            Case::Some,
            Value::Object {
                ty: Type::Byte,
                fields: vec![],
            },
        ),
        union(
            parse_type("Option<Int32>").unwrap(),
            Case::Some,
            Value::Int32(7),
        ),
        union(Type::Byte, Case::Some, Value::Byte(7)),
        Value::Object {
            ty: ty.clone(),
            fields: vec![],
        },
        Value::Void,
    ] {
        let fault = echo
            .invoke(
                vec![value],
                Limits {
                    instructions: 0,
                    ..Limits::default()
                },
            )
            .unwrap_err();
        assert!(
            fault.message.starts_with("invocation argument 0:"),
            "{fault}"
        );
        assert_eq!(fault.function.as_deref(), Some("Echo"));
        assert_eq!(fault.instruction, None);
    }
    let module = assemble(&echo_source("Result<Byte,Error>")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo(Result<Byte,Error>)").unwrap())
        .unwrap();
    for (case, payload) in [
        (Case::Some, Value::Byte(1)),
        (Case::None, Value::Void),
        (Case::Ok, Value::Error("wrong".into())),
        (Case::Err, Value::Byte(1)),
    ] {
        assert!(
            echo.invoke(
                vec![union(
                    parse_type("Result<Byte,Error>").unwrap(),
                    case,
                    payload
                )],
                Limits::default()
            )
            .is_err()
        );
    }
}

#[test]
fn nested_union_record_tags_are_normalized_and_fields_checked() {
    let module = assemble(".module App\n.type Point\n.field X Byte\n.end\n.function Echo(Result<Option<Point>,Error> value) -> Result<Option<Point>,Error>\nldarg value\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo(Result<Option<Point>,Error>)").unwrap())
        .unwrap();
    let value = |module: &str, field| {
        union(
            parse_type(&format!("Result<Option<[{module}]Point>,Error>")).unwrap(),
            Case::Ok,
            union(
                parse_type(&format!("Option<[{module}]Point>")).unwrap(),
                Case::Some,
                Value::Object {
                    ty: parse_type(&format!("[{module}]Point")).unwrap(),
                    fields: vec![field],
                },
            ),
        )
    };
    let result = echo
        .invoke(vec![value("App", Value::Byte(7))], Limits::default())
        .unwrap()
        .value;
    let expected = union(
        parse_type("Result<Option<Point>,Error>").unwrap(),
        Case::Ok,
        union(
            parse_type("Option<Point>").unwrap(),
            Case::Some,
            Value::Object {
                ty: Type::Named("Point".into()),
                fields: vec![Value::Byte(7)],
            },
        ),
    );
    assert_eq!(result, expected);
    assert!(
        echo.invoke(vec![value("Wrong", Value::Byte(7))], Limits::default())
            .is_err()
    );
    let fault = echo
        .invoke(vec![value("App", Value::Int32(7))], Limits::default())
        .unwrap_err();
    assert!(
        fault
            .message
            .contains("case Ok payload: case Some payload: field 0:")
    );
}

#[test]
fn record_receivers_can_carry_union_fields() {
    let module = assemble(".module App\n.type Box<T>\n.field Value T\n.method instance Get() -> T\nldarg this\nldfld Box<T>::Value\nret\n.end\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let get = program
        .resolve_function(&parse_function_ref("instance Box<Option<Void>>::Get()").unwrap())
        .unwrap();
    let payload = union(parse_type("Option<Void>").unwrap(), Case::Some, Value::Void);
    let receiver = Value::Object {
        ty: parse_type("Box<Option<Void>>").unwrap(),
        fields: vec![payload.clone()],
    };
    assert_eq!(
        get.invoke_instance(receiver, vec![], Limits::default())
            .unwrap()
            .value,
        payload
    );
}

#[test]
fn every_variant_must_be_importable_even_if_not_selected() {
    for ty in [
        "Option<Int32*>",
        "Result<Void,Ref<Int32>>",
        "Result<Option<Byte>,Option<Byte*>>",
    ] {
        let module = assemble(&echo_source(ty)).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        let fault = program
            .resolve_function(&parse_function_ref(&format!("Echo({ty})")).unwrap())
            .unwrap_err();
        assert!(
            fault
                .message
                .contains("pointer and Ref inputs are not supported")
        );
    }
}

#[test]
fn recursion_and_schema_budgets_include_variant_payloads() {
    let recursive = ".module App\n.type Loop\n.field Next Option<Loop>\n.end\n.function Echo(Loop value) -> Loop\nldarg value\nret\n.end".to_string();
    let deep = format!(
        ".module App\n{}\n.function Echo(N0 value) -> N0\nldarg value\nret\n.end",
        (0..34)
            .map(|i| format!(
                ".type N{i}\n.field Next Option<{}>\n.end",
                if i == 33 {
                    "Void".into()
                } else {
                    format!("N{}", i + 1)
                }
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
    // Each record expands two complete alternative schemas; named signatures stay small.
    let broad = format!(
        ".module App\n{}\n.function Echo(N0 value) -> N0\nldarg value\nret\n.end",
        (0..14)
            .map(|i| {
                let next = if i == 13 {
                    "Void".into()
                } else {
                    format!("N{}", i + 1)
                };
                format!(".type N{i}\n.field Next Result<{next},{next}>\n.end")
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    for (source, target, expected) in [
        (recursive, "Echo(Loop)", "recursive"),
        (deep, "Echo(N0)", "limit"),
        (broad, "Echo(N0)", "limit"),
    ] {
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        assert!(
            program
                .resolve_function(&parse_function_ref(target).unwrap())
                .unwrap_err()
                .message
                .contains(expected)
        );
    }
}
