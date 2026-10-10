use neoclr::{Limits, assemble, load, run};

#[test]
fn marker_attributes_roundtrip_without_executing_constructors() {
    let module = assemble(include_str!("../examples/attributes.neoil")).unwrap();
    assert_eq!(module.types[1].custom_attributes.len(), 1);
    assert_eq!(module.functions[1].custom_attributes.len(), 1);
    assert_eq!(module.functions[2].custom_attributes.len(), 1);
    let json = serde_json::to_value(&module).unwrap();
    assert!(json["types"][0].get("custom_attributes").is_none());
    let loaded = load(&json.to_string()).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), json);
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().output,
        ["Attribute metadata passed"]
    );
}

#[test]
fn runtime_library_union_marker_is_an_ordinary_attribute_type() {
    let source = ".module Test\n.entry Main\n.type Candidate\n.custom instance System.Runtime.CompilerServices.UnionAttribute::.ctor()\n.end\n.function Main() -> Candidate\nnewobj Candidate\nret\n.end";
    let module = assemble(source).unwrap();
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(
        result.value.ty(),
        neoclr::metadata::Type::Named("Candidate".into())
    );
    // There is no union-case validation or special carrier layout for this marker.
    assert_eq!(module.types[0].fields.len(), 0);
}

#[test]
fn repeated_markers_preserve_order_and_forward_references() {
    let source = ".module Test\n.type Target\n.custom instance Marker::.ctor()\n.custom instance Marker::.ctor()\n.end\n.type Marker\n.method instance .ctor() -> Void\nldvoid\nret\n.end\n.end";
    let module = assemble(source).unwrap();
    let attrs = &module.types[0].custom_attributes;
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0], attrs[1]);
    assert!(load(&serde_json::to_string(&module).unwrap()).is_ok());
}

#[test]
fn malformed_constructor_references_are_rejected_by_loader() {
    let module = assemble(include_str!("../examples/attributes.neoil")).unwrap();
    let original = serde_json::to_value(&module).unwrap();
    for (field, value) in [
        ("owner", serde_json::Value::Null),
        ("owner", serde_json::json!({"Named":"Missing"})),
        ("owner", serde_json::json!("Int32")),
        ("instance", serde_json::json!(false)),
        ("name", serde_json::json!("MarkerAttribute.Get")),
        ("parameters", serde_json::json!(["Int32"])),
    ] {
        let mut json = original.clone();
        json["types"][1]["custom_attributes"][0]["constructor"][field] = value;
        assert!(load(&json.to_string()).is_err(), "{field}");
    }
    let mut json = original.clone();
    json["functions"][0]["returns"] = serde_json::json!("Int32");
    assert!(load(&json.to_string()).is_err());
    let mut json = original;
    json["types"][1]["custom_attributes"][0]["arguments"] = serde_json::json!([42]);
    assert!(load(&json.to_string()).is_err());
}

#[test]
fn attribute_syntax_rejects_unsupported_locations_and_arguments() {
    for source in [
        ".custom instance Marker::.ctor()",
        ".function F() -> Void\nldvoid\n.custom instance Marker::.ctor()\nret\n.end",
        ".function F() -> Void\nHere:\n.custom instance Marker::.ctor()\nldvoid\nret\n.end",
        ".type Target\n.custom instance Marker::.ctor() = (01 00 00 00)\n.end",
        ".type Target\n.custom instance Marker::.ctor(Int32)\n.end",
        ".type Marker\n.method static .ctor() -> Void\nldvoid\nret\n.end\n.end",
    ] {
        assert!(
            assemble(&format!(".module Test\n{source}")).is_err(),
            "{source}"
        );
    }
}

#[test]
fn attribute_owner_is_closed_even_on_a_generic_definition() {
    let source = ".module Test\n.type Marker<T>\n.method instance .ctor() -> Void\nldvoid\nret\n.end\n.end\n.type Target<T>\n.custom instance Marker<Int32>::.ctor()\n.end";
    assert!(assemble(source).is_ok());
    for owner in ["Marker<!0>", "Marker<T>", "Marker", "Marker<Int32,String>"] {
        assert!(
            assemble(&source.replace("Marker<Int32>::", &format!("{owner}::"))).is_err(),
            "{owner}"
        );
    }
}

#[test]
fn scalar_attribute_arguments_roundtrip_without_constructor_execution() {
    let source = r#".module Attributes
.entry Main
.type Marker
.method instance .ctor(String text,Int32 number,Boolean flag) -> Void
fault "attribute constructor must not execute"
.end
.end
.type Target
.custom instance Marker::.ctor(String,Int32,Boolean) = [{"String":"Café = /items/{id}"},{"Int32":42},{"Boolean":true}]
.custom instance Marker::.ctor(String,Int32,Boolean) = [{"String":null},{"Int32":-7},{"Boolean":false}]
.end
.function Main() -> Int32
ldc.i4 42
ret
.end
"#;
    let module = assemble(source).unwrap();
    let json = serde_json::to_value(&module).unwrap();
    let loaded = load(&json.to_string()).unwrap();
    assert_eq!(json, serde_json::to_value(&loaded).unwrap());
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().value,
        neoclr::Value::Int32(42)
    );
    for arguments in [
        serde_json::json!([]),
        serde_json::json!([{"Int32":42},{"Int32":42},{"Boolean":true}]),
        serde_json::json!([{"String":"ok"},{"Int32":42},{"Boolean":true},{"Int32":0}]),
    ] {
        let mut altered = json.clone();
        altered["types"][1]["custom_attributes"][0]["arguments"] = arguments;
        assert!(load(&altered.to_string()).is_err());
    }
    let mut altered = json;
    altered["types"][1]["custom_attributes"][0]["target_token"] = serde_json::json!(0x08000001);
    assert!(load(&altered.to_string()).is_err());
}

#[test]
fn member_target_tokens_cannot_be_reused_as_type_annotations() {
    let source = r#".module App
.assembly {"name":"App","full_name":"App","modules":["App.dll"],"references":[]}
.entry Main
.type Target
.origin {"assembly":"App","module":"App.dll","name":"Target","token":33554433,"field_tokens":[67108865],"property_tokens":[],"parameter_tokens":[]}
.field Number Int32
.custom token 67108865 instance System.Runtime.CompilerServices.UnionAttribute::.ctor()
.end
.function Main() -> Boolean
ldtoken Target
ldc.i4 10
call neoCLR.Runtime.TypeShape(System.RuntimeTypeHandle,Int32)
ret
.end
"#;
    let module = assemble(source).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        neoclr::Value::Boolean(false)
    );
    let original = serde_json::to_value(&module).unwrap();
    for token in [0, 0x04000002, 0x08000001, 0x02000001] {
        let mut altered = original.clone();
        altered["types"][0]["custom_attributes"][0]["target_token"] = serde_json::json!(token);
        assert!(load(&altered.to_string()).is_err(), "{token}");
    }
}

#[test]
fn enum_and_named_attribute_data_validate_without_executing_assignments() {
    let source = r#".module Usage
.entry Main
.type Targets
.enum Int32
.field private Bits Int32
.literal Method 64
.end
.type UsageAttribute
.field public Tag String
.property instance Inherited() -> Boolean
.get instance UsageAttribute::get_Inherited()
.set instance UsageAttribute::set_Inherited(Boolean)
.end
.method instance get_Inherited() -> Boolean
fault "attribute getter must not execute"
.end
.method instance set_Inherited(Boolean value) -> Void
fault "attribute setter must not execute"
.end
.method instance .ctor(Targets targets) -> Void
fault "attribute constructor must not execute"
.end
.end
.type Candidate
.custom instance UsageAttribute::.ctor(Targets) = {"arguments":[{"Int32":64}],"named_arguments":[{"name":"Inherited","is_field":false,"value":{"Boolean":false}},{"name":"Tag","is_field":true,"value":{"String":"test"}}]}
.end
.function Main() -> Int32
ldc.i4 42
ret
.end
"#;
    let module = assemble(source).unwrap();
    let json = serde_json::to_value(&module).unwrap();
    let loaded = load(&json.to_string()).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), json);
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().value,
        neoclr::Value::Int32(42)
    );
    for (field, value) in [
        ("name", serde_json::json!("Missing")),
        ("name", serde_json::json!("")),
        ("is_field", serde_json::json!(true)),
        ("value", serde_json::json!({"Int32":0})),
    ] {
        let mut bad = json.clone();
        bad["types"][2]["custom_attributes"][0]["named_arguments"][0][field] = value;
        assert!(load(&bad.to_string()).is_err(), "{field}");
    }
    let mut bad = json.clone();
    let values = bad["types"][2]["custom_attributes"][0]["named_arguments"]
        .as_array_mut()
        .unwrap();
    values.push(values[0].clone());
    assert!(load(&bad.to_string()).is_err());
    // Identical Int32 storage does not make an ordinary record an enum parameter.
    let mut bad = json;
    bad["types"][0].as_object_mut().unwrap().remove("enum_info");
    assert!(load(&bad.to_string()).is_err());
}
