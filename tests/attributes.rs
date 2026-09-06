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
