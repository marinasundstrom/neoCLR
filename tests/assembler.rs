use neoclr::{assemble, assembler::parse_type, load, metadata::Type};

#[test]
fn nested_types_and_void_parse() {
    assert_eq!(
        parse_type("Result< Option<Void>, Ref<Int32> >").unwrap(),
        Type::Result(
            Box::new(Type::Option(Box::new(Type::Void))),
            Box::new(Type::Ref(Box::new(Type::Int32)))
        )
    );
    for text in [
        "Option<>",
        "Option<Void,Int32>",
        "Result<Void>",
        "Option<Void>>",
        "Ref<Option<Void>",
        "",
        "Array<Int32>",
    ] {
        assert!(parse_type(text).is_err(), "{text}");
    }
}

#[test]
fn invalid_source_is_rejected() {
    for body in [
        "br Missing",
        "Same:\nldvoid\nSame:\nret",
        "ldc.i4 2147483648\nret",
        "ldstr unquoted\nret",
        "ldnull\nret",
        "throw\nret",
        "ldvoid extra\nret",
        "ldarg 0\nret",
        "ldloc 0\nret",
        "call Missing()\nret",
        "newobj Missing\nret",
        "none Missing\nret",
        "ldvoid\n.local Void\nret",
        "br End\nEnd:",
    ] {
        let source = format!(".module Invalid\n.entry Main\n.function Main -> Void\n{body}\n.end");
        assert!(assemble(&source).is_err(), "{body}");
    }
    for source in [
        "",
        ".module Test\n.entry Main\n.function Main -> Void\nldvoid",
        ".module Test\n.module Twice",
        ".end",
        ".module Test\n.entry Main\n.function Main -> Void\n.param Void\nldvoid\nret\n.end",
    ] {
        assert!(assemble(source).is_err(), "{source}");
    }
}

#[test]
fn strings_keep_comment_characters_and_escape_sequences() {
    let module = assemble(".module Strings\n.entry Main\n.function Main -> String\nldstr \"semi;colon\\nquote\\\"\"\nret\n.end").unwrap();
    let value = neoclr::run(&module, neoclr::Limits::default())
        .unwrap()
        .value;
    assert_eq!(value, neoclr::Value::String("semi;colon\nquote\"".into()));
}

#[test]
fn diagnostics_include_source_lines() {
    let error =
        assemble(".module Test\n.entry Main\n.function Main -> Void\nldstr bad\n.end").unwrap_err();
    assert!(error.message.starts_with("line 4:"));
}

#[test]
fn loader_rejects_invalid_metadata_even_for_unused_functions() {
    let base =
        serde_json::to_value(assemble(include_str!("../examples/hello.neoil")).unwrap()).unwrap();
    let mut variants = vec![];
    let mut value = base.clone();
    value["format"] = 999.into();
    variants.push(value);
    let mut value = base.clone();
    value["extra"] = true.into();
    variants.push(value);
    let mut value = base.clone();
    value["functions"][0]["returns"] = serde_json::Value::Null;
    variants.push(value);
    let mut value = base.clone();
    value["functions"][0]["body"][0] = serde_json::json!({"op":"br","arg":999});
    variants.push(value);
    let mut value = base.clone();
    value["functions"]
        .as_array_mut()
        .unwrap()
        .push(base["functions"][0].clone());
    variants.push(value);
    let mut value = base.clone();
    value["functions"][0]["name"] = "System.Int32.Parse".into();
    variants.push(value);
    let mut value = base.clone();
    value["types"] =
        serde_json::json!([{"name":"Bad","fields":[{"name":"x","ty":{"Named":"Missing"}}]}]);
    variants.push(value);
    let mut value = base;
    value["functions"].as_array_mut().unwrap().push(
        serde_json::json!({"name":"Unused","returns":"Void","body":[{"op":"ldloc","arg":0}]}),
    );
    variants.push(value);
    for value in variants {
        assert!(load(&value.to_string()).is_err(), "{value}");
    }
}
