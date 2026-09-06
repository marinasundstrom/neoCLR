use neoclr::{
    Limits, Value, assemble, library, load,
    metadata::{Instruction, MemberId},
    run, verify,
};

#[test]
fn symbolic_calls_bind_before_generic_substitution_and_roundtrip() {
    let module = assemble(include_str!("../examples/member-identities.neoil")).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let before = serde_json::to_value(&loaded).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().output,
        ["generic declaration", "integer declaration"]
    );
    assert!(verify(&loaded).is_ok());
    assert_eq!(serde_json::to_value(&loaded).unwrap(), before);
    assert_eq!(
        loaded.functions[0].definition,
        Some(MemberId {
            module: "MemberIdentities".into(),
            index: 0
        })
    );
}

#[test]
fn explicit_definition_selects_either_colliding_overload() {
    let source = include_str!("../examples/member-identities.neoil");
    let first = assemble(&source.replace("@ MemberIdentities:1", "@ MemberIdentities:0")).unwrap();
    assert_eq!(
        run(&first, Limits::default()).unwrap().output,
        ["generic declaration", "generic declaration"]
    );
    assert!(
        assemble(&source.replace(" @ MemberIdentities:1", ""))
            .unwrap_err()
            .message
            .contains("ambiguous")
    );
}

#[test]
fn module_local_identity_does_not_change_when_library_rows_are_appended() {
    let system = library::system().unwrap();
    let index = system
        .functions
        .iter()
        .position(|f| f.name == "System.Math.Abs")
        .unwrap();
    let module = assemble(&format!(".module App\n.entry Main\n.function Unused() -> Void\nldvoid\nret\n.end\n.function Main() -> Int32\nldc.i4 -42\ncall System.Math::Abs(Int32) @ System:{index}\nldcase Ok\nret\n.end")).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let report = verify(&module).unwrap();
    let function = report
        .functions
        .iter()
        .find(|f| f.name == "System.Math.Abs")
        .unwrap();
    assert_eq!(
        function.definition,
        Some(MemberId {
            module: "System".into(),
            index: index as u32
        })
    );
    assert_eq!(function.function_index, module.functions.len() + index);
}

#[test]
fn loader_checks_definition_rows_and_reference_signature_guards() {
    let module = assemble(include_str!("../examples/member-identities.neoil")).unwrap();
    let original = serde_json::to_value(&module).unwrap();
    for id in [
        serde_json::json!({"module":"Other","index":0}),
        serde_json::json!({"module":"MemberIdentities","index":999}),
    ] {
        let mut json = original.clone();
        json["functions"][0]["definition"] = id;
        assert!(load(&json.to_string()).is_err());
    }
    let call = module.functions[3]
        .body
        .iter()
        .position(|op| matches!(op, Instruction::Call(r) if r.definition.is_some()))
        .unwrap();
    for (field, value) in [
        (
            "definition",
            serde_json::json!({"module":"Missing","index":1}),
        ),
        (
            "definition",
            serde_json::json!({"module":"MemberIdentities","index":99}),
        ),
        ("name", serde_json::json!("Choice.Missing")),
        ("instance", serde_json::json!(true)),
        ("parameters", serde_json::json!(["String"])),
        ("owner", serde_json::json!({"Named":"Choice"})),
    ] {
        let mut json = original.clone();
        json["functions"][3]["body"][call]["arg"][field] = value;
        assert!(load(&json.to_string()).is_err(), "{field}");
    }
}

#[test]
fn legacy_definitions_get_identities_during_linking() {
    let module = assemble(include_str!("../examples/member-identities.neoil")).unwrap();
    let mut json = serde_json::to_value(&module).unwrap();
    for function in json["functions"].as_array_mut().unwrap() {
        function.as_object_mut().unwrap().remove("definition");
    }
    let loaded = load(&json.to_string()).unwrap();
    assert!(loaded.functions.iter().all(|f| f.definition.is_none()));
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().output,
        ["generic declaration", "integer declaration"]
    );
}

#[test]
fn malformed_identity_syntax_is_rejected() {
    let source = include_str!("../examples/member-identities.neoil");
    for suffix in [
        "@ MemberIdentities:-1",
        "@ MemberIdentities:4294967296",
        "@ :1",
        "@ MemberIdentities",
        "@ MemberIdentities:1 @ MemberIdentities:1",
    ] {
        assert!(
            assemble(&source.replace("@ MemberIdentities:1", suffix)).is_err(),
            "{suffix}"
        );
    }
}

#[test]
fn attribute_constructor_can_reference_a_definition_row() {
    let source = ".module App\n.type Marker\n.method instance .ctor() -> Void\nldvoid\nret\n.end\n.end\n.type Target\n.custom instance Marker::.ctor() @ App:0\n.end";
    assert!(load(&serde_json::to_string(&assemble(source).unwrap()).unwrap()).is_ok());
    assert!(assemble(&source.replace("@ App:0", "@ App:1")).is_err());
}
