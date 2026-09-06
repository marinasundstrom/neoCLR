use neoclr::{Limits, Value, assemble, load, metadata::Type, run};

const BOX: &str = ".type Box<T>\n.field Value T\n.method static Create(T value) -> Box<T>\nldarg value\nnewobj Box<T>\nret\n.end\n.method instance Get() -> T\nldarg this\nldfld Box<T>::Value\nret\n.end\n.method instance With(T value) -> Box<T>\n.local Box<T> copy\nldarg this\nldarg value\nstfld 0\nstloc copy\nldloc copy\nret\n.end\n.end";
fn program(body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n{BOX}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap()
}

#[test]
fn sample_roundtrips_with_indexed_method_signatures() {
    let module = assemble(include_str!("../examples/generic-methods.neoil")).unwrap();
    assert_eq!(module.functions[0].parameters, [Type::TypeParameter(0)]);
    assert_eq!(module.functions[1].returns, Type::TypeParameter(0));
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().output,
        ["42", "Generic methods passed"]
    );
}

#[test]
fn generic_receivers_locals_and_updates_preserve_copies() {
    let module = program(
        ".local Box<Int32> original\nldc.i4 42\ncall Box<Int32>::Create(Int32)\nstloc original\nldloc original\nldc.i4 99\ncall instance Box<Int32>::With(Int32)\ncall instance Box<Int32>::Get()\nldloc original\ncall instance Box<Int32>::Get()\nsub",
        "Int32",
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(57)
    );
    let wrong = program(
        "ldc.i4 42\ncall Box<Int32>::Create(Int32)\ncall instance Box<String>::Get()",
        "String",
    );
    assert!(run(&wrong, Limits::default()).is_err());
}

#[test]
fn void_nesting_and_storage_conversions_work_across_method_calls() {
    for (body, result, expected) in [
        (
            "ldvoid\ncall Box<Void>::Create(Void)\ncall instance Box<Void>::Get()",
            "Void",
            Value::Void,
        ),
        (
            "ldc.i4 257\ncall Box<Byte>::Create(Byte)\ncall instance Box<Byte>::Get()",
            "Int32",
            Value::Int32(1),
        ),
        (
            "ldc.i4 42\ncall Box<Int32>::Create(Int32)\ncall Box<Box<Int32>>::Create(Box<Int32>)\ncall instance Box<Box<Int32>>::Get()\ncall instance Box<Int32>::Get()",
            "Int32",
            Value::Int32(42),
        ),
    ] {
        assert_eq!(
            run(&program(body, result), Limits::default())
                .unwrap()
                .value,
            expected
        );
    }
}

#[test]
fn symbolic_calls_in_generic_bodies_substitute_and_recursion_is_bounded() {
    let source = ".module Test\n.entry Main\n.type Identity<T>\n.method static Echo(T value) -> T\nldarg value\nret\n.end\n.method static Forward(T value) -> T\n.local T copy\nldarg value\nstloc copy\nldloc copy\ncall Identity<T>::Echo(T)\nret\n.end\n.method static Loop() -> Void\ncall Identity<T>::Loop()\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 42\ncall Identity<Int32>::Forward(Int32)\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let source = source.replace(
        "ldc.i4 42\ncall Identity<Int32>::Forward(Int32)",
        "call Identity<Int32>::Loop()",
    );
    let limits = Limits {
        frames: 4,
        ..Limits::default()
    };
    assert!(
        run(&assemble(&source).unwrap(), limits)
            .unwrap_err()
            .message
            .contains("frame limit")
    );
}

#[test]
fn colliding_closed_overloads_are_rejected_instead_of_picking_first() {
    let source = ".module Test\n.type Choice<T>\n.method static F(T value) -> Void\nldvoid\nret\n.end\n.method static F(Int32 value) -> Void\nldvoid\nret\n.end\n.end\n.function Main() -> Void\nldc.i4 1\ncall Choice<Int32>::F(Int32)\nret\n.end";
    assert!(assemble(source).unwrap_err().message.contains("ambiguous"));
    assert!(
        assemble(&source.replace("Choice<Int32>::F(Int32)", "Choice<String>::F(Int32)")).is_ok()
    );
}

#[test]
fn loader_validates_open_method_context_and_closed_calls() {
    let module = program(
        "ldc.i4 42\ncall Box<Int32>::Create(Int32)\ncall instance Box<Int32>::Get()",
        "Int32",
    );
    let mut json = serde_json::to_value(&module).unwrap();
    json["functions"][0]["returns"] = serde_json::json!({"TypeParameter":1});
    assert!(load(&json.to_string()).is_err());
    let mut json = serde_json::to_value(&module).unwrap();
    json["functions"][0]["body"][1]["arg"] =
        serde_json::json!({"Constructed":{"definition":"Box","arguments":[{"TypeParameter":1}]}});
    assert!(load(&json.to_string()).is_err());
    for call in [
        "Box::Create(Int32)",
        "Box<!0>::Create(!0)",
        "Box<Int32,String>::Create(Int32)",
        "Box<Int32>::Create(String)",
    ] {
        let source =
            format!(".module Test\n{BOX}\n.function Main() -> Void\ncall {call}\nret\n.end");
        assert!(assemble(&source).is_err(), "{call}");
    }
    assert!(assemble(".module Test\n.entry Box.F\n.type Box<T>\n.method static F() -> Void\nldvoid\nret\n.end\n.end").is_err());
}

#[test]
fn type_operands_in_generic_bodies_use_the_closed_argument() {
    let source = ".module Test\n.entry Main\n.type Native<T>\n.method static Size() -> Int32\nsizeof T\nret\n.end\n.end\n.function Main() -> Int32\ncall Native<Int32>::Size()\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(4)
    );
    // Layout requirements on type parameters are checked when executed with a concrete type.
    assert!(
        run(
            &assemble(&source.replace("Native<Int32>", "Native<String>")).unwrap(),
            Limits::default()
        )
        .is_err()
    );
}
