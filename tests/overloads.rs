use neoclr::{Limits, Value, assemble, assembler::parse_function_ref, load, metadata::Type, run};

#[test]
fn overload_sample_resolves_by_type_and_arity_and_round_trips() {
    let module = assemble(include_str!("../examples/overloads.neoil")).unwrap();
    let json = serde_json::to_value(&module).unwrap();
    assert_eq!(json["format"], 3);
    assert_eq!(
        json["functions"][3]["body"][1]["arg"],
        serde_json::json!({"name":"Describe","parameters":["String"],"owner":null,"instance":false})
    );
    let loaded = load(&json.to_string()).unwrap();
    let result = run(&loaded, Limits::default()).unwrap();
    assert_eq!(
        result.output,
        [
            "string overload",
            "integer overload",
            "parameterless overload",
            "42"
        ]
    );
    assert_eq!(result.value, Value::Void);
}

#[test]
fn signatures_parse_aliases_and_nested_generic_commas() {
    let target =
        parse_function_ref("Choose ( Result<Option<void>, Error>, Ref<Int32>, string, bool, int )")
            .unwrap();
    assert_eq!(target.name, "Choose");
    assert_eq!(
        target.parameters,
        vec![
            Type::Result(
                Box::new(Type::Option(Box::new(Type::Void))),
                Box::new(Type::Error)
            ),
            Type::Ref(Box::new(Type::Int32)),
            Type::String,
            Type::Boolean,
            Type::Int32
        ]
    );
    assert_eq!(parse_function_ref("Empty( )").unwrap().parameters, []);
    assert_eq!(
        parse_function_ref("F(string)").unwrap(),
        parse_function_ref("F(String)").unwrap()
    );
}

#[test]
fn malformed_or_missing_call_signatures_are_rejected_with_lines() {
    for operand in [
        "Main",
        "Main(",
        "Main)",
        "Main(,)",
        "Main(string,)",
        "Main(,string)",
        "Main(string) junk",
        "Main((string))",
        "Main(Option<string)",
        "Main(string>)",
        "Main(string int32)",
    ] {
        let source =
            format!(".module Test\n.entry Main\n.function Main -> Void\ncall {operand}\nret\n.end");
        let fault = assemble(&source).unwrap_err();
        assert!(fault.message.starts_with("line 4:"), "{operand}: {fault}");
    }
}

#[test]
fn return_type_alone_cannot_distinguish_overloads() {
    for second in ["Int32", "String"] {
        let source = format!(
            ".module Test\n.entry Main\n.function F -> Int32\n.param int\nldc.i4 1\nret\n.end\n.function F -> {second}\n.param Int32\nldc.i4 2\nret\n.end\n.function Main -> Void\nldvoid\nret\n.end"
        );
        let fault = assemble(&source).unwrap_err();
        assert!(fault.message.contains("duplicate"));
    }
}

#[test]
fn missing_overloads_and_wrong_runtime_arguments_do_not_fall_back() {
    for target in [
        "System.Console.WriteLine()",
        "System.Console.WriteLine(bool)",
        "System.Int32.Divide(int32)",
        "System.Int32.Parse(int32)",
    ] {
        let source =
            format!(".module Test\n.entry Main\n.function Main -> Void\ncall {target}\nret\n.end");
        assert!(
            assemble(&source)
                .unwrap_err()
                .message
                .contains("unknown function overload")
        );
    }
    let module = assemble(".module Test\n.entry Main\n.function Main -> Void\nldc.i4 42\ncall System.Console.WriteLine(string)\nret\n.end").unwrap();
    assert!(
        run(&module, Limits::default())
            .unwrap_err()
            .message
            .contains("expected String")
    );
}

#[test]
fn entry_selects_parameterless_overload_regardless_of_definition_order() {
    let module = assemble(".module Test\n.entry Main\n.function Main -> Void\n.param int32\nfault \"wrong entry\"\n.end\n.function Main -> Void\nldvoid\nret\n.end").unwrap();
    assert_eq!(run(&module, Limits::default()).unwrap().value, Value::Void);
}

#[test]
fn parameter_order_and_constructed_types_are_part_of_identity() {
    let module = assemble(".module Test\n.entry Main\n.function Choose -> Int32\n.param Option<Void>\n.param Result<Int32,Error>\nldc.i4 1\nret\n.end\n.function Choose -> Int32\n.param Result<Int32,Error>\n.param Option<Void>\nldc.i4 2\nret\n.end\n.function Main -> Int32\nldc.i4 42\nok Error\nnone void\ncall Choose(Result<int32,Error>, Option<void>)\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(2)
    );
}

#[test]
fn loader_checks_structured_signatures_and_rejects_old_format() {
    let base =
        serde_json::to_value(assemble(include_str!("../examples/hello.neoil")).unwrap()).unwrap();
    for arg in [
        serde_json::json!("System.Console.WriteLine"),
        serde_json::json!({"name":"System.Console.WriteLine"}),
        serde_json::json!({"name":"System.Console.WriteLine","parameters":[]}),
        serde_json::json!({"name":"System.Console.WriteLine","parameters":[{"Named":"Missing"}]}),
        serde_json::json!({"name":"System.Console.WriteLine","parameters":["String"],"extra":true}),
    ] {
        let mut json = base.clone();
        json["functions"][0]["body"][1]["arg"] = arg;
        assert!(load(&json.to_string()).is_err());
    }
    let mut old = base;
    old["format"] = 1.into();
    assert!(
        load(&old.to_string())
            .unwrap_err()
            .message
            .contains("expected 3")
    );
}

#[test]
fn intrinsic_signatures_are_reserved_but_other_overloads_are_allowed() {
    let source = ".module Test\n.entry Main\n.function System.Console.WriteLine -> Void\n.param bool\nldvoid\nret\n.end\n.function Main -> Void\nldc.bool true\ncall System.Console.WriteLine(bool)\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Void
    );
    let source = source.replace(".param bool", ".param string");
    assert!(
        assemble(&source)
            .unwrap_err()
            .message
            .contains("reserved function signature")
    );
}

#[test]
fn inline_declaration_signature_matches_call_signature() {
    let module = assemble(".module Test\n.entry Main\n.function Describe(int32) -> string\nldstr \"integer overload\"\nret\n.end\n.function Describe(string) -> string\nldstr \"string overload\"\nret\n.end\n.function Main() -> string\nldc.i4 42\ncall Describe(int32)\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::String("integer overload".into())
    );
}

#[test]
fn inline_parameters_support_nested_types_and_reject_mixed_declarations() {
    let source = ".module Test\n.entry Main\n.function Accept(Result<Option<void>, Error>, Ref<int32>) -> Void\nldvoid\nret\n.end\n.function Main -> Void\nldvoid\nret\n.end";
    let module = assemble(source).unwrap();
    assert_eq!(module.functions[0].parameters.len(), 2);
    for header in ["Accept(int32)", "Accept()"] {
        let source = format!(
            ".module Test\n.entry Main\n.function {header} -> Void\n.param string\nldvoid\nret\n.end"
        );
        assert!(
            assemble(&source)
                .unwrap_err()
                .message
                .contains("cannot be combined")
        );
    }
    for header in ["Accept(int32,)", "Accept(int32", "Accept((int32))"] {
        let source =
            format!(".module Test\n.entry Main\n.function {header} -> Void\nldvoid\nret\n.end");
        assert!(assemble(&source).is_err());
    }
}
