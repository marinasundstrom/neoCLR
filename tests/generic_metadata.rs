use neoclr::{Limits, assemble, assembler::parse_type, load, metadata::Type, run};

#[test]
fn generic_parameter_names_map_to_indices_and_closed_fields_substitute() {
    let source = ".module Test\n.type Pair<Left, !1>\n.field First Left\n.field Second !1\n.field Wrapped Result<Option<Left>, !1>\n.end";
    let module = assemble(source).unwrap();
    assert_eq!(
        module.types[0].generic_parameters,
        [Some("Left".into()), None]
    );
    assert_eq!(module.types[0].fields[0].ty, Type::TypeParameter(0));
    assert_eq!(module.types[0].fields[1].ty, Type::TypeParameter(1));
    let fields = module
        .instantiated_fields(&parse_type("Pair<Void, Error>").unwrap())
        .unwrap();
    assert_eq!(fields[0].ty, Type::Void);
    assert_eq!(fields[1].ty, Type::Error);
    assert_eq!(
        fields[2].ty,
        parse_type("Result<Option<Void>, Error>").unwrap()
    );
    let numeric = assemble(
        &source
            .replace(".field First Left", ".field First !0")
            .replace("Option<Left>", "Option<!0>"),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(module).unwrap(),
        serde_json::to_value(numeric).unwrap()
    );
}

#[test]
fn nested_constructed_fields_substitute_without_expanding_recursive_definitions() {
    let module = assemble(".module Test\n.type Node<T>\n.field Next Node<T>*\n.field Value T\n.end\n.type Wrapper<U>\n.field Node Node<Option<U>>\n.end").unwrap();
    let fields = module
        .instantiated_fields(&parse_type("Node<Int32>").unwrap())
        .unwrap();
    assert_eq!(fields[0].ty, parse_type("Node<Int32>*").unwrap());
    let fields = module
        .instantiated_fields(&parse_type("Wrapper<Void>").unwrap())
        .unwrap();
    assert_eq!(fields[0].ty, parse_type("Node<Option<Void>>").unwrap());
}

#[test]
fn bad_arity_open_context_and_duplicate_parameters_are_rejected() {
    for source in [
        ".type Box<T,T>\n.end",
        ".type Box<Int32>\n.end",
        ".type Box<T>\n.field Value !1\n.end",
        ".type Box<T>\n.end\n.function F(Box value) -> Void\nldvoid\nret\n.end",
        ".type Box<T>\n.end\n.function F(Box<Int32,String> value) -> Void\nldvoid\nret\n.end",
        ".type Box<T>\n.end\n.function F(Box<!0> value) -> Void\nldvoid\nret\n.end",
        ".type Box\n.end\n.function F(Box<Int32> value) -> Void\nldvoid\nret\n.end",
        ".type System.Int32<T>\n.end",
        ".type Option<T>\n.end",
        ".type Box<T>\n.method static F() -> Void\nldvoid\nret\n.end\n.end",
        ".type Box<T>\n.end\n.function F() -> Void\nnewobj Box\npop\nldvoid\nret\n.end",
    ] {
        assert!(
            assemble(&format!(".module Test\n{source}")).is_err(),
            "{source}"
        );
    }
    assert!(parse_type("Unknown<Int32>").is_ok());
    assert!(assemble(".module Test\n.type Wrapper\n.field Value Unknown<Int32>\n.end").is_err());
}

#[test]
fn loader_checks_generic_metadata_and_accepts_legacy_definitions() {
    let module = assemble(".module Test\n.type Box<T>\n.field Value T\n.end").unwrap();
    let mut json = serde_json::to_value(&module).unwrap();
    assert!(load(&json.to_string()).is_ok());
    json["types"][0]["fields"][0]["ty"] = serde_json::json!({"TypeParameter":1});
    assert!(load(&json.to_string()).is_err());
    let mut json = serde_json::to_value(&module).unwrap();
    json["types"][0]["generic_parameters"] = serde_json::json!(["bad name"]);
    assert!(load(&json.to_string()).is_err());
    let plain = assemble(".module Test\n.type Plain\n.field Value Int32\n.end").unwrap();
    let json = serde_json::to_value(&plain).unwrap();
    assert!(json["types"][0].get("generic_parameters").is_none());
    assert!(load(&json.to_string()).is_ok());
}

#[test]
fn substitution_rejects_missing_arguments_and_does_not_mutate_definitions() {
    let ty = parse_type("Result<!0, Option<!1>>").unwrap();
    assert!(ty.substitute_type_parameters(&[Type::Int32]).is_err());
    assert_eq!(
        ty.substitute_type_parameters(&[Type::Int32, Type::Void])
            .unwrap(),
        parse_type("Result<Int32, Option<Void>>").unwrap()
    );
    assert_eq!(ty, parse_type("Result<!0, Option<!1>>").unwrap());
}

#[test]
fn generic_signature_sample_roundtrips_and_executes_without_generic_allocation() {
    let module = assemble(include_str!("../examples/generic-metadata.neoil")).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().output,
        ["Generic signatures passed"]
    );
}
