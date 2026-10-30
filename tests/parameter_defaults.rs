use neoclr::{Limits, Value, assemble, load, run};
use serde_json::{Value as Json, json};

fn fixture() -> Json {
    serde_json::to_value(assemble(".module Defaults\n.entry Main\n.function Read(int32 value) -> int32\nldarg 0\nret\n.end\n.function Main() -> int32\nldc.i4 17\ncall Read(int32)\nret\n.end").unwrap()).unwrap()
}

#[test]
fn defaults_round_trip_without_changing_execution_arity() {
    let mut json = fixture();
    json["functions"][0]["parameter_defaults"] = json!([{"kind":"Int32","value":"42"}]);
    let module = load(&json.to_string()).unwrap();
    assert_eq!(
        serde_json::to_value(&module).unwrap()["functions"][0]["parameter_defaults"],
        json["functions"][0]["parameter_defaults"]
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(17)
    );
}

#[test]
fn malformed_defaults_are_rejected_at_load() {
    for defaults in [
        json!([null, null]),
        json!([{"kind":"Int32","value":"042"}]),
        json!([{"kind":"Int32","value":"2147483648"}]),
        json!([{"kind":"Boolean","value":"true"}]),
        json!([{"kind":"Null","value":""}]),
        json!([{"kind":"Unknown","value":"0"}]),
        json!([{"kind":"Int32","value":"42","extra":true}]),
    ] {
        let mut json = fixture();
        json["functions"][0]["parameter_defaults"] = defaults;
        assert!(load(&json.to_string()).is_err(), "accepted {json}");
    }
}

#[test]
fn instance_defaults_exclude_the_receiver() {
    let module = assemble(include_str!("../examples/names.neoil")).unwrap();
    let mut json = serde_json::to_value(module).unwrap();
    json["functions"][0]["parameter_defaults"] = json!([{"kind":"Int32","value":"2"}]);
    let loaded = load(&json.to_string()).unwrap();
    assert!(loaded.functions[0].instance);
    assert_eq!(loaded.functions[0].parameter_defaults.len(), 1);
    assert_eq!(run(&loaded, Limits::default()).unwrap().output, ["42"]);
}

#[test]
fn nominal_value_defaults_are_metadata_not_null_arguments() {
    let source = format!(
        "{}\n.function Probe(Counter value) -> Int32\nldc.i4 0\nret\n.end\n",
        include_str!("../examples/value_initialization.neoil")
    );
    let module = assemble(&source).unwrap();
    let mut json = serde_json::to_value(module).unwrap();
    let methods = json["functions"].as_array_mut().unwrap();
    let probe = methods.iter_mut().find(|f| f["name"] == "Probe").unwrap();
    probe["parameter_defaults"] = json!([{"kind":"Null","value":""}]);
    let loaded = load(&json.to_string()).unwrap();
    let probe = loaded.functions.iter().find(|f| f.name == "Probe").unwrap();
    assert_eq!(probe.parameter_defaults[0].as_ref().unwrap().kind, "Null");
}
