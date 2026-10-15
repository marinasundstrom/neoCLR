use neoclr::{Limits, LoadedProgram, Value, assemble};

fn image(declarations: &str) -> neoclr::Module {
    assemble(&format!(".module Variance\n.entry Main\n{declarations}\n.function Main() -> Int32\nldc.i4 42\nret\n.end")).unwrap()
}

#[test]
fn variance_metadata_roundtrip_and_nested_polarity() {
    let module = image(
        ".interface Read<T>\n.variance out\n.method instance Get() -> T\n.end\n.end\n.interface Write<T>\n.variance in\n.method instance Put(T) -> Int32\n.end\n.end\n.interface Nested<T>\n.variance out\n.implements Read<T>\n.method instance Supply(Write<T>) -> Read<T>\n.end\n.end",
    );
    let json = serde_json::to_string(&module).unwrap();
    assert!(json.contains("covariant"));
    let restored = serde_json::from_str(&json).unwrap();
    let p = LoadedProgram::new(&restored).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn malformed_and_unsafe_declarations_are_rejected() {
    for declaration in [
        ".interface Bad<T>\n.variance out in\n.end",
        ".type class Bad<T>\n.variance out\n.end",
        ".interface Bad<T>\n.variance out\n.method instance Put(T) -> Int32\n.end\n.end",
        ".interface Bad<T>\n.variance in\n.method instance Get() -> T\n.end\n.end",
        ".interface Bad<T>\n.variance out\n.method instance Get() -> arrayref<T>\n.end\n.end",
        ".interface Bad<T>\n.variance out\n.method instance Get() -> T&\n.end\n.end",
        ".type Result<T>\n.end\n.interface Bad<T>\n.variance out\n.method instance Get() -> Result<T>\n.end\n.end",
        ".interface Invariant<T>\n.end\n.interface Bad<T>\n.variance out\n.implements Invariant<T>\n.end",
        ".interface Sink<T>\n.variance in\n.end\n.interface Bad<T>\n.variance out\n.method instance Get() -> Sink<T>\n.end\n.end",
    ] {
        let module = assemble(&format!(
            ".module Variance\n.entry Main\n{declaration}\n.function Main() -> Int32\nldc.i4 42\nret\n.end"
        ));
        assert!(
            module
                .and_then(|m| LoadedProgram::new(&m))
                .and_then(|p| p.verify())
                .is_err(),
            "{declaration}"
        );
    }
    let mut json = serde_json::to_value(image(".interface Read<T>\n.end")).unwrap();
    json["types"][0]["generic_variance"] = serde_json::json!(["unknown"]);
    assert!(serde_json::from_value::<neoclr::Module>(json).is_err());
}

#[test]
fn omitted_variance_remains_invariant_metadata() {
    let module = image(".interface Read<T>\n.method instance Get() -> T\n.end\n.end");
    assert!(module.types[0].generic_variance.is_empty());
    assert!(
        !serde_json::to_string(&module)
            .unwrap()
            .contains("generic_variance")
    );
    LoadedProgram::new(&module).unwrap().verify().unwrap();
}
