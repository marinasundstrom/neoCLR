use neoclr::assembler::parse_function_ref;
use neoclr::{Limits, LoadedProgram, Value, assemble, load, run, verify};

const IDENTITY: &str = ".function Identity<T>(T value) -> T\nldarg value\nret\n.end\n.function Forward<U>(U value) -> U\nldarg value\ncall Identity<U>(U)\nret\n.end";
fn program(body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n{IDENTITY}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap()
}

#[test]
fn explicit_function_arguments_roundtrip_and_normalize_storage() {
    for (body, returns, expected) in [
        (
            "ldc.i4 42\ncall Forward<Int32>(Int32)",
            "Int32",
            Value::Int32(42),
        ),
        (
            "ldc.i4 257\ncall Forward<Byte>(Byte)",
            "Int32",
            Value::Int32(1),
        ),
        ("ldvoid\ncall Forward<Void>(Void)", "Void", Value::Void),
    ] {
        let module = program(body, returns);
        verify(&module).unwrap();
        let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
        assert_eq!(run(&loaded, Limits::default()).unwrap().value, expected);
    }
}

#[test]
fn owner_and_method_parameters_are_independent() {
    let source = ".module Test\n.entry Main\n.type Box<T>\n.field Value T\n.method static Extract<U>(Box<U> value, T ignored) -> U\nldarg value\nldfld Box<U>::Value\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 42\nnewobj Box<Int32>\nldstr \"owner\"\ncall Box<String>::Extract<Int32>(Box<Int32>,String)\nret\n.end";
    let module = assemble(source).unwrap();
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn managed_references_preserve_identity_and_frame_return_rules() {
    let module = program(
        ".local Int32 value\nldc.i4 42\nstloc value\nldloca value\ncall Identity<Int32&>(Int32&)\nldobj Int32",
        "Int32",
    );
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let bad = program(
        ".local Int32 value\nldc.i4 42\nstloc value\nldloca value\ncall Identity<Int32&>(Int32&)",
        "Int32&",
    );
    assert!(
        run(&bad, Limits::default())
            .unwrap_err()
            .message
            .contains("current frame")
    );
}

#[test]
fn host_calls_and_closed_graphs_distinguish_argument_only_instantiations() {
    let module = assemble(&format!(
        ".module Test\n{IDENTITY}\n.function Size<T>() -> Int32\nsizeof T\nret\n.end"
    ))
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let roots = [
        parse_function_ref("Size<Int32>()").unwrap(),
        parse_function_ref("Size<Int64>()").unwrap(),
    ];
    let graph = program.analyze_reachability(&roots, 10).unwrap();
    assert_ne!(graph.roots[0], graph.roots[1]);
    for (root, expected) in roots.iter().zip([4, 8]) {
        assert_eq!(
            program
                .resolve_function(root)
                .unwrap()
                .invoke(vec![], Limits::default())
                .unwrap()
                .value,
            Value::Int32(expected)
        );
    }
}

#[test]
fn invalid_method_arguments_and_contexts_are_rejected() {
    for call in [
        "Identity(Int32)",
        "Identity<Int32,String>(Int32)",
        "Identity<!!0>(!!0)",
        "Identity<String>(Int32)",
    ] {
        assert!(
            assemble(&format!(
                ".module Test\n{IDENTITY}\n.function Main() -> Void\ncall {call}\nret\n.end"
            ))
            .is_err(),
            "{call}"
        );
    }
    for source in [
        ".module Test\n.entry Main\n.function Main<T>() -> Void\nldvoid\nret\n.end",
        ".module Test\n.function F<T,T>() -> Void\nldvoid\nret\n.end",
        ".module Test\n.function F<T>(!0 value) -> T\nldarg value\nret\n.end",
        ".module Test\n.function F<T>(!!1 value) -> T\nldarg value\nret\n.end",
        ".module Test\n.type C\n.method instance F<T>() -> Void\nldvoid\nret\n.end\n.end",
    ] {
        assert!(assemble(source).is_err(), "{source}");
    }
    let module = program("ldc.i4 42\ncall Identity<Int32>(Int32)", "Int32");
    let mut json = serde_json::to_value(&module).unwrap();
    json["functions"][0]["parameters"][0] = serde_json::json!({"MethodTypeParameter":1});
    assert!(load(&json.to_string()).is_err());
    let mut json = serde_json::to_value(&module).unwrap();
    json["functions"][0]["generic_parameters"][0] = serde_json::json!("Int32");
    assert!(load(&json.to_string()).is_err());
}

#[test]
fn symbolic_substitution_does_not_capture_caller_parameters() {
    let source = ".module Test\n.entry Main\n.type Box<T>\n.method static First<U>(T first, U second) -> T\nldarg first\nret\n.end\n.end\n.function Forward<A,B>(A first, B second) -> A\nldarg first\nldarg second\ncall Box<A>::First<B>(A,B)\nret\n.end\n.function Main() -> Int32\nldc.i4 42\nldstr \"second\"\ncall Forward<Int32,String>(Int32,String)\nret\n.end";
    let module = assemble(source).unwrap();
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn arity_distinguishes_overloads_and_invalid_substituted_shapes_fault() {
    let module = assemble(".module Test\n.entry Main\n.function F() -> Int32\nldc.i4 1\nret\n.end\n.function F<T>() -> Int32\nldc.i4 42\nret\n.end\n.function Main() -> Int32\ncall F<Int32>()\nret\n.end").unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert!(assemble(".module Test\n.function Address<T>(T value) -> T&\nldarga value\nret\n.end\n.function Main() -> Void\ncall Address<Int32&>(Int32&)\nret\n.end").is_err());
}

#[test]
fn method_parameter_wrappers_parse_and_substitute_recursively() {
    use neoclr::metadata::Type;
    let ty = neoclr::assembler::parse_type("!!0[]&").unwrap();
    assert_eq!(
        ty.substitute_method_parameters(&[Type::Int32]).unwrap(),
        Type::ByRef(Box::new(Type::Array(Box::new(Type::Int32))))
    );
}
