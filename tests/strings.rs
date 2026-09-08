use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
};
fn program() -> LoadedProgram {
    LoadedProgram::new(&assemble(".module App\n.function EqualText(String left, String right) -> Boolean\nldarga left\nldarg right\ncall instance System.String::Equals(String)\nret\n.end").unwrap()).unwrap()
}
fn text(value: &str) -> Value {
    Value::String(value.into())
}
fn record(name: &str, fields: Vec<Value>) -> Value {
    Value::Object {
        ty: neoclr::assembler::parse_type(name).unwrap(),
        fields,
    }
}
fn carrier(name: &str, case: Value) -> Value {
    record(name, vec![Value::Erased(Box::new(case))])
}
fn result(value: Value) -> Value {
    carrier(
        "System.Result<String,System.Text.Utf8SliceError>",
        record("System.Result.Ok<String>", vec![value]),
    )
}
fn error(name: &str) -> Value {
    let error = carrier(
        "System.Text.Utf8SliceError",
        record(&format!("System.Text.Utf8SliceError.{name}"), vec![]),
    );
    carrier(
        "System.Result<String,System.Text.Utf8SliceError>",
        record(
            "System.Result.Error<System.Text.Utf8SliceError>",
            vec![error],
        ),
    )
}

#[test]
fn ordinary_string_methods_execute_unicode_program_and_handle_errors() {
    let module = assemble(include_str!("../examples/strings.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().output,
        ["Hello, neoCLR!", "10", "🌍", "Invalid boundary handled"]
    );
}

#[test]
fn concat_preserves_utf8_embedded_nul_and_owned_inputs() {
    let program = program();
    let concat = program
        .resolve_function(&parse_function_ref("System.String::Concat(String,String)").unwrap())
        .unwrap();
    let left = text("café\0");
    let right = text("🌍");
    assert_eq!(
        concat
            .invoke(vec![left.clone(), right.clone()], Limits::default())
            .unwrap()
            .value,
        text("café\0🌍")
    );
    assert_eq!(left, text("café\0"));
    assert_eq!(right, text("🌍"));
    assert_eq!(
        concat
            .invoke(vec![text(""), text("")], Limits::default())
            .unwrap()
            .value,
        text("")
    );
}

#[test]
fn equality_is_ordinal_and_empty_is_not_absence() {
    let program = program();
    let equals = program
        .resolve_function(&parse_function_ref("EqualText(String,String)").unwrap())
        .unwrap();
    let empty = program
        .resolve_function(&parse_function_ref("instance System.String::IsEmpty()").unwrap())
        .unwrap();
    for (left, right, expected) in [
        ("🌍", "🌍", true),
        ("A", "a", false),
        ("é", "e\u{301}", false),
        ("\0", "", false),
    ] {
        assert_eq!(
            equals
                .invoke(vec![text(left), text(right)], Limits::default())
                .unwrap()
                .value,
            Value::Boolean(expected)
        );
    }
    for (value, expected) in [("", true), ("\0", false), (" ", false)] {
        assert_eq!(
            empty
                .invoke_instance(text(value), vec![], Limits::default())
                .unwrap()
                .value,
            Value::Boolean(expected)
        );
    }
}

#[test]
fn byte_counts_are_explicit_and_slices_preserve_codepoints() {
    let program = program();
    let count = program
        .resolve_function(
            &parse_function_ref("instance System.String::GetUtf8ByteCount()").unwrap(),
        )
        .unwrap();
    let slice = program
        .resolve_function(
            &parse_function_ref("instance System.String::SliceUtf8(Int32,Int32)").unwrap(),
        )
        .unwrap();
    for (value, expected) in [
        ("", 0),
        ("abc", 3),
        ("é", 2),
        ("e\u{301}", 3),
        ("🌍", 4),
        ("\0", 1),
    ] {
        assert_eq!(
            count
                .invoke_instance(text(value), vec![], Limits::default())
                .unwrap()
                .value,
            Value::Int32(expected)
        );
    }
    for (value, start, length, expected) in [
        ("café 🌍", 3, 2, "é"),
        ("café 🌍", 6, 4, "🌍"),
        ("café 🌍", 10, 0, ""),
        ("", 0, 0, ""),
        ("e\u{301}", 1, 2, "\u{301}"),
    ] {
        assert_eq!(
            slice
                .invoke_instance(
                    text(value),
                    vec![Value::Int32(start), Value::Int32(length)],
                    Limits::default()
                )
                .unwrap()
                .value,
            result(text(expected))
        );
    }
}

#[test]
fn invalid_ranges_and_boundaries_are_results_and_do_not_poison_invocation() {
    let program = program();
    let slice = program
        .resolve_function(
            &parse_function_ref("instance System.String::SliceUtf8(Int32,Int32)").unwrap(),
        )
        .unwrap();
    for (start, length, case) in [
        (-1, 0, "OutOfRange"),
        (0, -1, "OutOfRange"),
        (0, 5, "OutOfRange"),
        (5, 0, "OutOfRange"),
        (i32::MAX, i32::MAX, "OutOfRange"),
        (1, 1, "InvalidBoundary"),
        (0, 1, "InvalidBoundary"),
        (1, 0, "InvalidBoundary"),
        (1, 4, "OutOfRange"),
    ] {
        assert_eq!(
            slice
                .invoke_instance(
                    text("🌍"),
                    vec![Value::Int32(start), Value::Int32(length)],
                    Limits::default()
                )
                .unwrap()
                .value,
            error(case)
        );
    }
    assert_eq!(
        slice
            .invoke_instance(
                text("🌍"),
                vec![Value::Int32(0), Value::Int32(4)],
                Limits::default()
            )
            .unwrap()
            .value,
        result(text("🌍"))
    );
    let fault = slice
        .invoke_instance(
            text("x"),
            vec![Value::Int32(0), Value::Int32(1)],
            Limits {
                instructions: 1,
                ..Limits::default()
            },
        )
        .unwrap_err();
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].function.name,
        "System.String.SliceUtf8"
    );
}

#[test]
fn service_planning_distinguishes_il_members_from_string_runtime_helpers() {
    let program = program();
    let equals = program
        .analyze_reachability(
            &[parse_function_ref("instance System.String::Equals(String)").unwrap()],
            1,
        )
        .unwrap();
    assert!(
        equals
            .required_services()
            .contains(&RuntimeService::SlotReferences)
    );
    assert!(
        !equals
            .required_services()
            .contains(&RuntimeService::StringOperations)
    );
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("instance System.String::SliceUtf8(Int32,Int32)").unwrap()],
            16,
        )
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [
            RuntimeService::StringOperations,
            RuntimeService::ValueStorage
        ]
    );
    assert_eq!(
        graph.functions[1].target.name,
        "neoCLR.Runtime.StringSliceUtf8"
    );
    assert!(
        graph
            .missing_services(&[
                RuntimeService::StringOperations,
                RuntimeService::ValueStorage
            ])
            .is_empty()
    );
}

#[test]
fn slice_errors_have_checked_case_accessors_and_native_statuses() {
    let module = assemble(".module App\n.function Raw(String value, Int32 start, Int32 length) -> System.Value\nldarg value\nldarg start\nldarg length\ncall neoCLR.Runtime.StringSliceUtf8(String,Int32,Int32)\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let resolve = |name: &str| {
        program
            .resolve_function(&parse_function_ref(name).unwrap())
            .unwrap()
    };
    for (start, length, expected) in [
        (0, 4, Value::String("🌍".into())),
        (1, 4, Value::Byte(1)),
        (1, 0, Value::Byte(2)),
    ] {
        assert_eq!(
            resolve("Raw(String,Int32,Int32)")
                .invoke(
                    vec![text("🌍"), Value::Int32(start), Value::Int32(length)],
                    Limits::default()
                )
                .unwrap()
                .value,
            Value::Erased(Box::new(expected))
        );
    }
    for (start, length, name, other) in [
        (1, 4, "OutOfRange", "InvalidBoundary"),
        (1, 0, "InvalidBoundary", "OutOfRange"),
    ] {
        let result = resolve("instance System.String::SliceUtf8(Int32,Int32)")
            .invoke_instance(
                text("🌍"),
                vec![Value::Int32(start), Value::Int32(length)],
                Limits::default(),
            )
            .unwrap()
            .value;
        let case =
            resolve("instance System.Result<String,System.Text.Utf8SliceError>::GetErrorCase()")
                .invoke_instance(result, vec![], Limits::default())
                .unwrap()
                .value;
        let error =
            resolve("instance System.Result.Error<System.Text.Utf8SliceError>::get_Value()")
                .invoke_instance(case, vec![], Limits::default())
                .unwrap()
                .value;
        assert_eq!(
            resolve(&format!(
                "instance System.Text.Utf8SliceError::get_Is{name}()"
            ))
            .invoke_instance(error.clone(), vec![], Limits::default())
            .unwrap()
            .value,
            Value::Boolean(true)
        );
        assert_eq!(
            resolve(&format!("instance System.Text.Utf8SliceError::Get{name}()"))
                .invoke_instance(error.clone(), vec![], Limits::default())
                .unwrap()
                .value,
            record(&format!("System.Text.Utf8SliceError.{name}"), vec![])
        );
        assert!(
            resolve(&format!(
                "instance System.Text.Utf8SliceError::Get{other}()"
            ))
            .invoke_instance(error.clone(), vec![], Limits::default())
            .is_err()
        );
        assert_eq!(
            resolve("instance System.Text.Utf8SliceError::ToString()")
                .invoke_instance(error, vec![], Limits::default())
                .unwrap()
                .value,
            text(name)
        );
    }
    let function = neoclr::library::system()
        .unwrap()
        .functions
        .iter()
        .find(|f| f.name == "System.String.SliceUtf8")
        .unwrap();
    assert!(!function.body.is_empty());
}
