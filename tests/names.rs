use neoclr::{
    Limits, Value, assemble, load,
    metadata::{Case, Type},
    run,
};

#[test]
fn names_are_preserved_and_instructions_use_indices() {
    let module = assemble(include_str!("../examples/names.neoil")).unwrap();
    let json = serde_json::to_value(&module).unwrap();
    assert_eq!(
        json["functions"][0]["parameter_names"],
        serde_json::json!(["offset"])
    );
    assert_eq!(json["functions"][0]["body"][0]["arg"], 0);
    assert_eq!(json["functions"][0]["body"][5]["arg"], 1);
    assert_eq!(
        json["functions"][1]["local_names"],
        serde_json::json!(["point", "message"])
    );
    let loaded = load(&json.to_string()).unwrap();
    let result = run(&loaded, Limits::default()).unwrap();
    assert_eq!(result.output, ["42"]);
    assert_eq!(
        result.value,
        Value::result(Value::Void, Type::Void, Type::Error, Case::Ok)
    );
}

#[test]
fn numeric_and_named_operands_emit_identical_instruction_bodies() {
    let named = ".module Test\n.entry Main\n.function Add(left: int32, right: int32) -> int32\nldarg left\nldarg right\nadd\nret\n.end\n.function Main() -> int32\n.local result: int32\nldc.i4 1\nldc.i4 2\ncall Add(int32, int32)\nstloc result\nldloc result\nret\n.end";
    let numeric = named
        .replace("ldarg left", "ldarg 0")
        .replace("ldarg right", "ldarg 1")
        .replace("stloc result", "stloc 0")
        .replace("ldloc result", "ldloc 0");
    let a = assemble(named).unwrap();
    let b = assemble(&numeric).unwrap();
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(&b).unwrap()
    );
    assert_eq!(run(&a, Limits::default()).unwrap().value, Value::Int32(3));
}

#[test]
fn mixed_unnamed_and_named_slots_and_legacy_params_are_supported() {
    let source = ".module Test\n.entry Main\n.function F(Result<Void,Error>, p: Option<Ptr<Int32>>) -> Void\nldarg p\npop\nldvoid\nret\n.end\n.function Legacy -> Int32\n.param value: int32\n.local int32\n.local value: int32\nldarg value\nstloc value\nldloc 1\nret\n.end\n.function Main() -> Int32\nldc.i4 42\ncall Legacy(int32)\nret\n.end";
    let module = assemble(source).unwrap();
    assert_eq!(
        module.functions[0].parameter_names,
        [None, Some("p".into())]
    );
    assert_eq!(
        module.functions[1].local_names,
        [None, Some("value".into())]
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn names_do_not_distinguish_overloads_or_appear_in_call_signatures() {
    let source = ".module Test\n.function F(a: int32) -> Void\nldvoid\nret\n.end\n.function F(b: int32) -> Void\nldvoid\nret\n.end";
    assert!(assemble(source).unwrap_err().message.contains("duplicate"));
    assert!(neoclr::assembler::parse_function_ref("F(a: int32)").is_err());
}

#[test]
fn bad_names_unknown_names_and_duplicate_names_are_rejected() {
    for body in [
        ".local point: int32\nldloc missing\nret",
        ".local point: int32\nldarg point\nret",
        ".local point: int32\nstloc Point\nret",
        ".local 0: int32\nldvoid\nret",
        ".local : int32\nldvoid\nret",
        ".local p: int32\n.local p: string\nldvoid\nret",
    ] {
        assert!(
            assemble(&format!(
                ".module Test\n.entry Main\n.function Main() -> Void\n{body}\n.end"
            ))
            .is_err(),
            "{body}"
        );
    }
    for params in ["x: int32, x: string", "1: int32", "x:", "a.b: int32"] {
        assert!(
            assemble(&format!(
                ".module Test\n.function F({params}) -> Void\nldvoid\nret\n.end"
            ))
            .is_err()
        );
    }
    assert!(assemble(".module Test\n.type T\n.method instance F(this: int32) -> Void\nldvoid\nret\n.end\n.end").is_err());
}

#[test]
fn loader_validates_name_tables_and_accepts_omitted_legacy_tables() {
    let base =
        serde_json::to_value(assemble(include_str!("../examples/names.neoil")).unwrap()).unwrap();
    for names in [
        serde_json::json!(["too", "many"]),
        serde_json::json!([""]),
        serde_json::json!(["this"]),
        serde_json::json!(["12"]),
    ] {
        let mut json = base.clone();
        json["functions"][0]["parameter_names"] = names;
        assert!(load(&json.to_string()).is_err());
    }
    for names in [
        serde_json::json!(["short"]),
        serde_json::json!(["same", "same"]),
        serde_json::json!(["bad name", null]),
    ] {
        let mut json = base.clone();
        json["functions"][1]["local_names"] = names;
        assert!(load(&json.to_string()).is_err());
    }
    let mut json = base;
    for function in json["functions"].as_array_mut().unwrap() {
        function.as_object_mut().unwrap().remove("parameter_names");
        function.as_object_mut().unwrap().remove("local_names");
    }
    assert!(load(&json.to_string()).is_ok());
}
