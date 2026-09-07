use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_type};

#[test]
fn non_generic_union_can_own_direct_nested_case() {
    let source = include_str!("../examples/non_generic_union.neoil");
    let module = assemble(source).unwrap();
    let shape = module
        .type_definition(&parse_type("Shape").unwrap())
        .unwrap();
    let circle = module
        .type_definition(&parse_type("Shape.Circle").unwrap())
        .unwrap();
    assert_eq!(circle.declaring_type, shape.definition);
    assert!(circle.generic_parameters.is_empty());

    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["7"]);
    let identity = program
        .resolve_type_identity(&parse_type("Shape.Circle").unwrap())
        .unwrap();
    assert!(
        matches!(identity, neoclr::TypeIdentity::Definition { definition, .. } if definition == circle.definition.clone().unwrap())
    );
    assert_eq!(
        program
            .resolve_function(
                &neoclr::assembler::parse_function_ref("instance Shape.Circle::Area()").unwrap()
            )
            .unwrap()
            .invoke_instance(
                Value::Object {
                    ty: parse_type("Shape.Circle").unwrap(),
                    fields: vec![Value::Int32(11)]
                },
                vec![],
                Limits::default()
            )
            .unwrap()
            .value,
        Value::Int32(11)
    );
}
