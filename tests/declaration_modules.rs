use neoclr::{Limits, Value, assemble, load, run};
use serde_json::json;

fn image() -> serde_json::Value {
    let module =
        assemble(".module Package\n.entry Main\n.function Main() -> int32\nldc.i4 42\nret\n.end")
            .unwrap();
    let mut image = serde_json::to_value(module).unwrap();
    image["functions"][0]["origin"] = json!({
        "assembly":image["assemblies"][0]["full_name"],
        "module":image["assemblies"][0]["modules"][0], "name":"Main", "token":100663297
    });
    image["functions"][0]["namespace"] = json!("Example.Math");
    image["assemblies"][0]["declaration_modules"] =
        json!({"version":1,"names":["Example.Math","Example.Empty"]});
    image
}

#[test]
fn module_ownership_does_not_change_execution() {
    let module = load(&image().to_string()).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert_eq!(
        module.assemblies[0]
            .declaration_modules
            .as_ref()
            .unwrap()
            .names
            .len(),
        2
    );
}

#[test]
fn reject_malformed_or_incomplete_module_tables() {
    for table in [
        json!({"version":2,"names":["Example.Math"]}),
        json!({"version":1,"names":["Example.Math","Example.Math"]}),
        json!({"version":1,"names":["Bad..Name"]}),
        json!({"version":1,"names":[]}),
    ] {
        let mut bad = image();
        bad["assemblies"][0]["declaration_modules"] = table;
        assert!(load(&bad.to_string()).is_err());
    }
}

#[test]
fn legacy_metadata_remains_readable() {
    let mut old = image();
    old["assemblies"][0]
        .as_object_mut()
        .unwrap()
        .remove("declaration_modules");
    assert!(
        load(&old.to_string()).unwrap().assemblies[0]
            .declaration_modules
            .is_none()
    );
}

#[test]
fn missing_module_diagnostic_identifies_owner_and_assembly() {
    let mut bad = image();
    bad["assemblies"][0]["declaration_modules"]["names"] = json!(["Example.Empty"]);
    let message = load(&bad.to_string()).unwrap_err().to_string();
    assert!(message.contains("Example.Math"), "{message}");
    assert!(
        message.contains(bad["assemblies"][0]["full_name"].as_str().unwrap()),
        "{message}"
    );
}
