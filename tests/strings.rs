use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble,
    assembler::parse_function_ref,
    metadata::{Case, Type},
};
fn program() -> LoadedProgram {
    LoadedProgram::new(&assemble(".module App").unwrap()).unwrap()
}
fn text(value: &str) -> Value {
    Value::String(value.into())
}
fn result(value: Value, case: Case) -> Value {
    Value::result(value, Type::String, Type::Error, case)
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
        .resolve_function(&parse_function_ref("instance System.String::Equals(String)").unwrap())
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
                .invoke_instance(text(left), vec![text(right)], Limits::default())
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
            result(text(expected), Case::Ok)
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
    for (start, length, error) in [
        (-1, 0, "ArgumentOutOfRange"),
        (0, -1, "ArgumentOutOfRange"),
        (0, 5, "ArgumentOutOfRange"),
        (5, 0, "ArgumentOutOfRange"),
        (i32::MAX, i32::MAX, "ArgumentOutOfRange"),
        (1, 1, "InvalidUtf8Boundary"),
        (0, 1, "InvalidUtf8Boundary"),
        (1, 0, "InvalidUtf8Boundary"),
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
            result(Value::Error(error.into()), Case::Err)
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
        result(text("🌍"), Case::Ok)
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
    assert!(equals.required_services().is_empty());
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("instance System.String::SliceUtf8(Int32,Int32)").unwrap()],
            2,
        )
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [RuntimeService::StringOperations]
    );
    assert_eq!(
        graph.functions[1].target.name,
        "neoCLR.Runtime.StringSliceUtf8"
    );
    assert!(
        graph
            .missing_services(&[RuntimeService::StringOperations])
            .is_empty()
    );
}
