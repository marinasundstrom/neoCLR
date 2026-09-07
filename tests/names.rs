use neoclr::{Limits, Value, assemble, load, metadata::Type, run};

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
    assert_eq!(result.value, ordinary_success());
}

#[test]
fn numeric_and_named_operands_emit_identical_instruction_bodies() {
    let named = ".module Test\n.entry Main\n.function Add(int32 left, int32 right) -> int32\nldarg left\nldarg right\nadd\nret\n.end\n.function Main() -> int32\n.local int32 result\nldc.i4 1\nldc.i4 2\ncall Add(int32, int32)\nstloc result\nldloc result\nret\n.end";
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
    let source = ".module Test\n.entry Main\n.function F(System.Result<Void,Error>, System.Option<Ptr<Int32>> p) -> Void\nldarg p\npop\nldvoid\nret\n.end\n.function Legacy -> Int32\n.param int32 value\n.local int32\n.local int32 value\nldarg value\nstloc value\nldloc 1\nret\n.end\n.function Main() -> Int32\nldc.i4 42\ncall Legacy(int32)\nret\n.end";
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
    let source = ".module Test\n.function F(int32 a) -> Void\nldvoid\nret\n.end\n.function F(int32 b) -> Void\nldvoid\nret\n.end";
    assert!(assemble(source).unwrap_err().message.contains("duplicate"));
    assert!(neoclr::assembler::parse_function_ref("F(int32 a)").is_err());
}

#[test]
fn bad_names_unknown_names_and_duplicate_names_are_rejected() {
    for body in [
        ".local int32 point\nldloc missing\nret",
        ".local int32 point\nldarg point\nret",
        ".local int32 point\nstloc Point\nret",
        ".local int32 0\nldvoid\nret",
        ".local int32 ''\nldvoid\nret",
        ".local int32 p\n.local string p\nldvoid\nret",
    ] {
        assert!(
            assemble(&format!(
                ".module Test\n.entry Main\n.function Main() -> Void\n{body}\n.end"
            ))
            .is_err(),
            "{body}"
        );
    }
    for params in ["int32 x, string x", "int32 1", "x:", "int32 a.b"] {
        assert!(
            assemble(&format!(
                ".module Test\n.function F({params}) -> Void\nldvoid\nret\n.end"
            ))
            .is_err()
        );
    }
    assert!(
        assemble(
            ".module Test\n.type T\n.method instance F(int32 this) -> Void\nldvoid\nret\n.end\n.end"
        )
        .is_err()
    );
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

#[test]
fn type_first_slots_preserve_nested_types_and_whitespace() {
    let source = ".module Test\n.function F(System.Result<Int32, Error> result, System.Option<Ptr<Int32>> , Int32 * pointer) -> Void\n.local System.Result<Int32, Error> copy\n.local System.Option<Ptr<Int32>>\n.local Int32 *   address\nldarg result\nstloc copy\nldarg pointer\nstloc address\nldvoid\nret\n.end";
    let module = assemble(source).unwrap();
    let f = &module.functions[0];
    assert_eq!(
        f.parameter_names,
        [Some("result".into()), None, Some("pointer".into())]
    );
    assert_eq!(
        f.local_names,
        [Some("copy".into()), None, Some("address".into())]
    );
    assert_eq!(f.parameters, f.locals);
    assert_eq!(
        f.parameters[0],
        Type::Constructed {
            definition: "System.Result".into(),
            arguments: vec![Type::Int32, Type::Error]
        }
    );
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(loaded.functions[0].parameter_names, f.parameter_names);
}

#[test]
fn former_name_first_syntax_and_extra_names_are_rejected() {
    for parameter in ["value: Int32", "Int32 first second", "Int32 'value'"] {
        assert!(
            assemble(&format!(
                ".module Test\n.function F({parameter}) -> Void\nldvoid\nret\n.end"
            ))
            .is_err()
        );
    }
    for slot in [
        ".local value: Int32",
        ".param value: Int32",
        ".local Int32 first second",
        ".local Int32 'value'",
    ] {
        assert!(
            assemble(&format!(
                ".module Test\n.function F -> Void\n{slot}\nldvoid\nret\n.end"
            ))
            .is_err()
        );
    }
}

#[test]
fn unnamed_type_first_parameters_and_locals_execute_by_index() {
    let source = ".module Test\n.entry Main\n.function Identity(Int32) -> Int32\n.local Int32\nldarg 0\nstloc 0\nldloc 0\nret\n.end\n.function Main() -> Int32\nldc.i4 42\ncall Identity(Int32)\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

fn ordinary_success() -> Value {
    let module = assemble(".module Expected\n.entry Main\n.function Main() -> System.Result<Void,Error>\nldvoid\nnewobj instance System.Result.Ok<Void>::.ctor(Void)\nnewobj instance System.Result<Void,Error>::.ctor(System.Result.Ok<Void>)\nret\n.end").unwrap();
    neoclr::run(&module, Limits::default()).unwrap().value
}
