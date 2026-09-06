use neoclr::{Limits, Value, assemble, assembler::parse_type, load, metadata::Type, run};

fn program(result: &str, body: &str) -> neoclr::Module {
    assemble(&format!(".module Test\n.entry Main\n.type Box<T>\n.field Value T\n.end\n.function Main() -> {result}\n{body}\nret\n.end")).unwrap()
}

#[test]
fn generic_values_sample_preserves_copies_and_roundtrips() {
    let source = include_str!("../examples/generic-values.neoil");
    let module = assemble(source).unwrap();
    let json = serde_json::to_string(&module).unwrap();
    let loaded = load(&json).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().output,
        ["Generic values passed"]
    );
    let indexed = source
        .replace("Pair<Int32, String>::First", "0")
        .replace("Pair<Int32, String>::Second", "1");
    assert_eq!(
        serde_json::to_value(module).unwrap(),
        serde_json::to_value(assemble(&indexed).unwrap()).unwrap()
    );
}

#[test]
fn closed_identity_and_nested_generic_fields_survive_execution() {
    let module = program(
        "Box<Box<Int32>>",
        "ldc.i4 42\nnewobj Box<Int32>\nnewobj Box<Box<Int32>>",
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Object {
            ty: parse_type("Box<Box<Int32>>").unwrap(),
            fields: vec![Value::Object {
                ty: parse_type("Box<Int32>").unwrap(),
                fields: vec![Value::Int32(42)]
            }],
        }
    );
    let wrong = program("Box<String>", "ldc.i4 42\nnewobj Box<Int32>");
    assert!(run(&wrong, Limits::default()).is_err());
    let wrong = program(
        "Box<Box<String>>",
        "ldc.i4 42\nnewobj Box<Int32>\nnewobj Box<Box<String>>",
    );
    assert!(run(&wrong, Limits::default()).is_err());
}

#[test]
fn substituted_fields_apply_storage_and_stack_conversions() {
    let module = program(
        "Int32",
        "ldc.i4 257\nnewobj Box<Byte>\nldc.i4 258\nstfld Box<Byte>::Value\nldfld 0",
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(2)
    );
    let module = program("Void", "ldvoid\nnewobj Box<Void>\nldfld 0");
    assert_eq!(run(&module, Limits::default()).unwrap().value, Value::Void);
    for body in [
        "newobj Box<Void>\nldfld 0",
        "ldstr \"bad\"\nnewobj Box<Int32>\nldfld 0",
        "ldc.i4 1\nnewobj Box<Int32>\nldstr \"bad\"\nstfld 0\nldfld 0",
        "ldc.i4 1\nnewobj Box<Int32>\nldfld 1",
    ] {
        assert!(
            run(&program("Int32", body), Limits::default()).is_err(),
            "{body}"
        );
    }
}

#[test]
fn empty_generic_records_and_recursive_pointer_fields_are_values() {
    let module = assemble(
        ".module Test\n.entry Main\n.type Empty<T>\n.end\n.type Node<T>\n.field Value T\n.field Next Node<T>*\n.end\n.function Main() -> Empty<Void>\nnewobj Empty<Void>\nnewobj Node<Empty<Void>>\nret\n.end",
    );
    // The missing pointer is an execution error, not a recursively expanded layout.
    assert!(run(&module.unwrap(), Limits::default()).is_err());
    let module = assemble(".module Test\n.entry Main\n.type Empty<T>\n.end\n.type Node<T>\n.field Value T\n.field Next Node<T>*\n.end\n.function Main() -> Empty<Void>\nnewobj Empty<Void>\nptr.null Node<Empty<Void>>\nnewobj Node<Empty<Void>>\nldfld 0\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Object {
            ty: parse_type("Empty<Void>").unwrap(),
            fields: vec![]
        }
    );
}

#[test]
fn loader_validates_construction_operands_and_preserves_legacy_encoding() {
    let module = program("Box<Int32>", "ldc.i4 42\nnewobj Box<Int32>");
    let json = serde_json::to_value(&module).unwrap();
    assert_eq!(
        json["functions"][0]["body"][1]["arg"],
        serde_json::to_value(parse_type("Box<Int32>").unwrap()).unwrap()
    );
    for ty in [
        Type::TypeParameter(0),
        Type::Named("Box".into()),
        parse_type("Box<Int32, String>").unwrap(),
        Type::Int32,
        parse_type("Missing<Int32>").unwrap(),
        parse_type("Box<!0>").unwrap(),
    ] {
        let mut invalid = json.clone();
        invalid["functions"][0]["body"][1]["arg"] = serde_json::to_value(ty).unwrap();
        assert!(load(&invalid.to_string()).is_err());
    }
    let module = assemble(".module Test\n.entry Main\n.type Plain\n.end\n.function Main() -> Plain\nnewobj Plain\nret\n.end").unwrap();
    let json = serde_json::to_value(&module).unwrap();
    assert_eq!(json["functions"][0]["body"][0]["arg"], "Plain");
    assert_eq!(
        run(&load(&json.to_string()).unwrap(), Limits::default())
            .unwrap()
            .value
            .ty(),
        Type::Named("Plain".into())
    );
}

#[test]
fn malformed_generic_field_owners_and_unsupported_native_fields_are_rejected() {
    for operand in [
        "Box::Value",
        "Box<!0>::Value",
        "Box<Int32,String>::Value",
        "Box<Int32>::Missing",
    ] {
        let source = format!(
            ".module Test\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Int32\nldc.i4 1\nnewobj Box<Int32>\nldfld {operand}\nret\n.end"
        );
        assert!(assemble(&source).is_err(), "{operand}");
    }
    assert!(assemble(".module Test\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Void\nldc.i4 1\nheap.alloc Box<String>\npop\nldvoid\nret\n.end").is_err());
}
