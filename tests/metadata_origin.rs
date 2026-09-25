use neoclr::{LoadedProgram, assemble, load};

fn source() -> String {
    r#".module Origin
.assembly {"name":"App","full_name":"App, Version=1.0.0.0","modules":["App.dll"],"references":["System.Runtime"]}
.type Item
.origin {"assembly":"App, Version=1.0.0.0","module":"App.dll","name":"Example.Item","token":33554434,"field_tokens":[67108865]}
.field Value Int32
.end
.function Read(Int32 value) -> Int32
.origin {"assembly":"App, Version=1.0.0.0","module":"App.dll","name":"Read","token":100663297,"parameter_tokens":[134217729]}
ldarg value
ret
.end
"#.into()
}
#[test]
fn source_identity_and_tokens_survive_artifact_roundtrip_and_linking() {
    let module = assemble(&source()).unwrap();
    let json = serde_json::to_string(&module).unwrap();
    let loaded = load(&json).unwrap();
    assert_eq!(loaded.assemblies[0].references, ["System.Runtime"]);
    assert_eq!(loaded.types[0].origin, module.types[0].origin);
    assert_eq!(loaded.functions[0].origin, module.functions[0].origin);
    LoadedProgram::new(&loaded).unwrap().verify().unwrap();
}
#[test]
fn source_identity_is_descriptive_and_does_not_change_execution_ids() {
    let module = assemble(&source()).unwrap();
    assert_eq!(
        module.types[0].definition.as_ref().unwrap().module,
        "Origin"
    );
    assert_eq!(
        module.functions[0].definition.as_ref().unwrap().module,
        "Origin"
    );
}
#[test]
fn malformed_scopes_tokens_and_duplicate_origins_are_rejected() {
    for text in [
        source().replace("33554434", "100663297"),
        source().replace("67108865", "67108864"),
        source().replace("\"field_tokens\":[67108865]", "\"field_tokens\":[]"),
        source().replace("\"module\":\"App.dll\"", "\"module\":\"Other.dll\""),
        source().replace(".field Value", ".origin {}\n.field Value"),
        source().replace("ldarg value", "ldarg value\n.origin {}"),
        source().replace(
            "\"modules\":[\"App.dll\"]",
            "\"modules\":[\"App.dll\",\"App.dll\"]",
        ),
    ] {
        assert!(assemble(&text).is_err(), "{text}");
    }
}
#[test]
fn artifact_origin_metadata_cannot_bypass_validation() {
    let module = assemble(&source()).unwrap();
    let mut json = serde_json::to_value(module).unwrap();
    json["functions"][0]["origin"]["token"] = serde_json::json!(33554434);
    assert!(load(&json.to_string()).is_err());
}

#[test]
fn identical_tokens_in_distinct_source_modules_are_valid() {
    let second = source()
        .replace(".module Origin\n", "")
        .replace("App.dll", "Other.dll")
        .replace("App, Version=1.0.0.0", "Other, Version=1.0.0.0")
        .replace("\"name\":\"App\"", "\"name\":\"Other\"")
        .replace(".type Item", ".type OtherItem")
        .replace(".function Read", ".function OtherRead");
    let module = assemble(&(source() + &second)).unwrap();
    LoadedProgram::new(&module).unwrap().verify().unwrap();
}

#[test]
fn duplicate_tokens_within_one_source_module_are_rejected() {
    let duplicate = r#".type OtherItem
.origin {"assembly":"App, Version=1.0.0.0","module":"App.dll","name":"OtherItem","token":33554434}
.end
"#;
    assert!(assemble(&(source() + duplicate)).is_err());
}

#[test]
fn nested_source_owner_survives_roundtrip_and_rejects_missing_or_cyclic_owners() {
    let nested = r#".type Inner
.origin {"assembly":"App, Version=1.0.0.0","module":"App.dll","name":"Example.Item.Inner","token":33554435,"declaring_type_token":33554434}
.end
"#;
    let text = source() + nested;
    let module = assemble(&text).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        loaded.types[1]
            .origin
            .as_ref()
            .unwrap()
            .declaring_type_token,
        Some(33554434)
    );
    for invalid in [
        text.replace(
            "\"declaring_type_token\":33554434",
            "\"declaring_type_token\":33554436",
        ),
        text.replace(
            "\"declaring_type_token\":33554434",
            "\"declaring_type_token\":33554435",
        ),
        text.replace(
            "\"field_tokens\":[67108865]",
            "\"field_tokens\":[67108865],\"declaring_type_token\":33554435",
        ),
    ] {
        assert!(assemble(&invalid).is_err());
    }
    let mut artifact = serde_json::to_value(module).unwrap();
    artifact["types"][1]["origin"]["declaring_type_token"] = serde_json::json!(33554436);
    assert!(load(&artifact.to_string()).is_err());
}

#[test]
fn reflection_access_metadata_roundtrips_and_is_scoped_to_definition_kind() {
    let mut json = serde_json::to_value(assemble(&source()).unwrap()).unwrap();
    json["types"][0]["origin"]["publicly_visible"] = serde_json::json!(true);
    json["functions"][0]["origin"]["member_access"] = serde_json::json!("Private");
    let loaded = load(&json.to_string()).unwrap();
    assert_eq!(loaded.types[0].origin.as_ref().unwrap().publicly_visible, Some(true));
    assert_eq!(loaded.functions[0].origin.as_ref().unwrap().member_access,
        Some(neoclr::metadata_origin::SourceAccess::Private));
    let mut wrong_kind = json.clone();
    wrong_kind["types"][0]["origin"]["member_access"] = serde_json::json!("Public");
    assert!(load(&wrong_kind.to_string()).is_err());
    let mut wrong_kind = json.clone();
    wrong_kind["functions"][0]["origin"]["publicly_visible"] = serde_json::json!(true);
    assert!(load(&wrong_kind.to_string()).is_err());
    json["functions"][0]["origin"]["member_access"] = serde_json::json!("Unknown");
    assert!(load(&json.to_string()).is_err());
}
