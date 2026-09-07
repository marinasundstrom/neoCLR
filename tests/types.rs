use neoclr::{
    Limits, Value, assemble,
    assembler::{parse_function_ref, parse_type},
    library, load,
    metadata::{Representation, Type},
    run,
};

#[test]
fn primitives_share_canonical_type_identity_and_library_definitions() {
    for spelling in ["int", "int32", "Int32", "System.Int32"] {
        let ty = parse_type(spelling).unwrap();
        assert_eq!(ty, Value::Int32(1).ty());
        let def = library::system().unwrap().type_definition(&ty).unwrap();
        assert_eq!(def.name, "System.Int32");
        assert_eq!(def.representation, Representation::Runtime);
        assert!(def.fields.is_empty());
    }
    for ty in [Type::Void, Type::Boolean, Type::String, Type::Error] {
        assert!(library::system().unwrap().type_definition(&ty).is_some());
    }
    assert_eq!(
        parse_function_ref("instance int::ToString()").unwrap(),
        parse_function_ref("instance System.Int32::ToString()").unwrap()
    );
}

#[test]
fn library_methods_have_real_owners_and_primitive_receiver_slot() {
    let lib = library::system().unwrap();
    let parse = lib
        .functions
        .iter()
        .find(|f| f.name == "System.Int32.Parse")
        .unwrap();
    assert_eq!(parse.owner, Some(Type::Int32));
    assert!(!parse.instance);
    let method = lib
        .functions
        .iter()
        .find(|f| f.name == "System.Int32.ToString")
        .unwrap();
    assert_eq!(method.owner, Some(Type::Int32));
    assert!(method.instance);
    assert!(method.parameters.is_empty());
    assert_eq!(method.argument_types(), [Type::Int32]);
}

#[test]
fn methods_sample_executes_and_round_trips() {
    let module = assemble(include_str!("../examples/types.neoil")).unwrap();
    let json = serde_json::to_value(&module).unwrap();
    let loaded = load(&json.to_string()).unwrap();
    let result = run(&loaded, Limits::default()).unwrap();
    assert_eq!(result.output, ["42", "0", "42"]);
    assert_eq!(result.value, Value::Void);
    assert!(result.heap.is_empty());
    let method = loaded
        .functions
        .iter()
        .find(|f| f.name == "Point.Sum")
        .unwrap();
    assert_eq!(method.owner, Some(Type::Named("Point".into())));
    assert!(method.instance);
}

#[test]
fn instance_receivers_are_typed_and_require_the_instance_call_form() {
    for body in [
        "call instance System.Int32::ToString()",
        "ldvoid\ncall instance System.Int32::ToString()",
    ] {
        let module = assemble(&format!(
            ".module Test\n.entry Main\n.function Main() -> string\n{body}\nret\n.end"
        ))
        .unwrap();
        assert!(run(&module, Limits::default()).is_err());
    }
    for call in [
        "System.Int32::ToString()",
        "instance System.Int32::Parse(string)",
        "instance System.Int32::Missing()",
    ] {
        assert!(
            assemble(&format!(
                ".module Test\n.entry Main\n.function Main() -> Void\ncall {call}\nret\n.end"
            ))
            .is_err()
        );
    }
}

#[test]
fn receiver_is_a_read_only_snapshot_not_an_implicit_shared_object() {
    let module = assemble(".module Test\n.entry Main\n.type Box\n.field X int32\n.method instance Changed() -> Box\nldarg 0\nldc.i4 99\nstfld 0\nret\n.end\n.end\n.function Main() -> int32\n.local Box\nldc.i4 7\nnewobj Box\nstloc 0\nldloc 0\ncall instance Box::Changed()\npop\nldloc 0\nldfld 0\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(7)
    );
}

#[test]
fn static_and_instance_overloads_resolve_independently() {
    let module = assemble(".module Test\n.entry Main\n.type Box\n.method static Value() -> int32\nldc.i4 1\nret\n.end\n.method instance Value() -> int32\nldc.i4 2\nret\n.end\n.end\n.function Main() -> int32\ncall Box::Value()\nnewobj Box\ncall instance Box::Value()\nadd\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(3)
    );
}

#[test]
fn malformed_owners_representations_and_entry_methods_are_rejected() {
    for source in [
        ".module Test\n.method instance M() -> Void\nldvoid\nret\n.end",
        ".module Test\n.type Box\n.method mutable M() -> Void\nldvoid\nret\n.end\n.end",
        ".module Test\n.type Box\n.method instance M() -> Void\nldarg 1\nret\n.end\n.end",
        ".module Test\n.entry Box.M\n.type Box\n.method instance M() -> Void\nldvoid\nret\n.end\n.end",
        ".module System\n.type System.Int32\n.field Value Int32\n.end",
        ".module Test\n.entry Main\n.function Main() -> Int32\nnewobj System.Int32\nret\n.end",
    ] {
        assert!(assemble(source).is_err(), "{source}");
    }
    let base =
        serde_json::to_value(assemble(include_str!("../examples/types.neoil")).unwrap()).unwrap();
    for owner in [
        serde_json::Value::Null,
        serde_json::json!("Int32"),
        serde_json::json!({"Named":"Missing"}),
        serde_json::json!({"Ref":{"Named":"Point"}}),
    ] {
        let mut module = base.clone();
        module["functions"][1]["owner"] = owner;
        assert!(load(&module.to_string()).is_err());
    }
    let mut lib = serde_json::to_value(library::system().unwrap()).unwrap();
    lib["types"][0]["representation"] = "Record".into();
    assert!(load(&lib.to_string()).is_err());
}

#[test]
fn pointer_signatures_are_distinct_from_ownership_and_support_nested_types() {
    assert_eq!(
        parse_type("int32*").unwrap(),
        Type::Ptr(Box::new(Type::Int32))
    );
    assert_eq!(
        parse_type("System.Int32 **").unwrap(),
        parse_type("Ptr<Ptr<int>>").unwrap()
    );
    assert_ne!(
        parse_type("Ptr<int32>").unwrap(),
        parse_type("Ref<int32>").unwrap()
    );
    let module = assemble(".module Test\n.entry Main\n.function Unused(Ptr<Int32>, Ref<Int32>) -> Void\nldvoid\nret\n.end\n.function Main() -> System.Option<Ptr<Int32>>\nnewobj instance System.Option.None::.ctor()\nnewobj instance System.Option<Ptr<Int32>>::.ctor(System.Option.None)\nret\n.end").unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().value.ty(),
        Type::Constructed {
            definition: "System.Option".into(),
            arguments: vec![Type::Ptr(Box::new(Type::Int32))]
        }
    );
    assert!(assemble(".module Test\n.function F(Missing*) -> Void\nldvoid\nret\n.end").is_err());
}
