use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{assemble_modules, parse_function_ref, parse_type},
    metadata::Type,
};
const SAMPLE: &str = include_str!("../examples/type_arities.neoil");

#[test]
fn same_name_types_and_methods_resolve_by_arity_and_round_trip() {
    let module = assemble(SAMPLE).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().output,
        ["companion", "one parameter", "two parameters", "42"]
    );
    let owners: Vec<_> = module
        .functions
        .iter()
        .filter(|f| f.name == "Container.Describe")
        .map(|f| f.owner.clone())
        .collect();
    assert_eq!(owners.len(), 3);
    assert_eq!(
        owners[1],
        Some(Type::Constructed {
            definition: "Container".into(),
            arguments: vec![Type::TypeParameter(0)]
        })
    );
}

#[test]
fn host_identity_layout_and_input_schema_select_the_correct_arity() {
    let module = assemble(SAMPLE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let zero = parse_type("Container").unwrap();
    let one = parse_type("Container<Int32>").unwrap();
    assert_ne!(
        program.resolve_type_identity(&zero).unwrap(),
        program.resolve_type_identity(&one).unwrap()
    );
    assert_eq!(module.type_definition(&zero).unwrap().fields.len(), 0);
    assert_eq!(module.type_definition(&one).unwrap().fields.len(), 1);
    assert_eq!(neoclr::memory::layout(&module, &zero).unwrap().size, 0);
    assert_eq!(neoclr::memory::layout(&module, &one).unwrap().size, 4);
    let read = program
        .resolve_function(&parse_function_ref("instance Container<Int32>::Read()").unwrap())
        .unwrap();
    assert_eq!(
        read.invoke_instance(
            Value::Object {
                ty: one,
                fields: vec![Value::Int32(42)]
            },
            vec![],
            Limits::default()
        )
        .unwrap()
        .value,
        Value::Int32(42)
    );
    assert!(
        read.invoke_instance(
            Value::Object {
                ty: zero,
                fields: vec![]
            },
            vec![],
            Limits::default()
        )
        .is_err()
    );
}

#[test]
fn scope_and_visibility_follow_definition_arity_across_modules() {
    let a = ".module A\n.type public Shared\n.method static Describe() -> Int32\nldc.i4 1\nret\n.end\n.end";
    let b = ".module B\n.type public Shared<T>\n.method static Describe() -> Int32\nldc.i4 2\nret\n.end\n.end";
    let app = ".module App\n.references (A,B)\n.entry Main\n.function Main() -> Int32\ncall [A]Shared::Describe()\ncall [B]Shared<Int32>::Describe()\nadd\nret\n.end";
    let modules = assemble_modules(&[app, a, b]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(3)
    );
    assert!(
        assemble_modules(&[&app.replace("[B]Shared<Int32>", "[A]Shared<Int32>"), a, b]).is_err()
    );
    assert!(assemble_modules(&[app, a, &b.replace(".type public", ".type internal")]).is_err());
    assert!(assemble_modules(&[&app.replace("(A,B)", "(A)"), a, b]).is_err());
}

#[test]
fn same_name_does_not_grant_private_access_between_arities() {
    let src = ".module App\n.type Box\n.method private static Hidden() -> Int32\nldc.i4 1\nret\n.end\n.end\n.type Box<T>\n.method static Leak() -> Int32\ncall Box::Hidden()\nret\n.end\n.end";
    assert!(
        assemble(src)
            .unwrap_err()
            .message
            .contains("method access denied")
    );
}

#[test]
fn duplicate_arity_invalid_arity_and_closed_definition_owners_are_rejected() {
    assert!(assemble(".module App\n.type X<T>\n.end\n.type X<U>\n.end").is_err());
    let module = assemble(SAMPLE).unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .resolve_function(
                &parse_function_ref("Container<Int32,Int32,Int32>::Describe()").unwrap()
            )
            .is_err()
    );
    let mut module = module;
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "Container.Read")
        .unwrap()
        .owner = Some(parse_type("Container<Int32>").unwrap());
    assert!(
        LoadedProgram::new(&module)
            .unwrap_err()
            .message
            .contains("open type definition")
    );
}
