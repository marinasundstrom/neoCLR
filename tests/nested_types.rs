use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{assemble_modules, parse_type},
};
const SAMPLE: &str = include_str!("../examples/nested_types.neoil");

#[test]
fn nested_generic_case_round_trip_execution_and_host_identity() {
    let module = assemble(SAMPLE).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let companion = module
        .type_definition(&parse_type("Demo.Result").unwrap())
        .unwrap();
    let case = module
        .type_definition(&parse_type("Demo.Result.Ok<Int32>").unwrap())
        .unwrap();
    assert_eq!(case.declaring_type, companion.definition);
    assert_eq!(case.generic_parameters, vec![Some("T".into())]);
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["42"]);
    let read = program
        .resolve_function(
            &neoclr::assembler::parse_function_ref("instance Demo.Result.Ok<Int32>::Read()")
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        read.invoke_instance(
            Value::Object {
                ty: parse_type("Demo.Result.Ok<Int32>").unwrap(),
                fields: vec![Value::Int32(73)]
            },
            vec![],
            Limits::default()
        )
        .unwrap()
        .value,
        Value::Int32(73)
    );

    assert_ne!(
        program
            .resolve_type_identity(&parse_type("Demo.Result.Ok<Int32>").unwrap())
            .unwrap(),
        program
            .resolve_type_identity(&parse_type("Demo.Result<Int32,Int32>").unwrap())
            .unwrap()
    );
    assert_eq!(
        neoclr::memory::layout(&module, &parse_type("Demo.Result.Ok<Int32>").unwrap())
            .unwrap()
            .size,
        4
    );
}

#[test]
fn multiple_levels_distinct_owners_and_void_payload() {
    let source = ".module Nested\n.type A\n.type B\n.type Case<T>\n.field Value T\n.end\n.end\n.end\n.type B\n.type Case<T>\n.field Value T\n.end\n.end\n.function Main() -> A.B.Case<Void>\nldvoid\nnewobj A.B.Case<Void>\nret\n.end\n.entry Main";
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Object {
            ty: parse_type("A.B.Case<Void>").unwrap(),
            fields: vec![Value::Void]
        }
    );
    assert_ne!(
        module.types[0].declaring_type,
        module.types[3].declaring_type
    );
}

#[test]
fn enclosing_visibility_is_effective_across_module_linking() {
    let library =
        ".module Cases\n.type public Outer\n.type public Case<T>\n.field Value T\n.end\n.end";
    let app = ".module App\n.references (Cases)\n.entry Main\n.function Main() -> [Cases]Outer.Case<Int32>\nldc.i4 42\nnewobj [Cases]Outer.Case<Int32>\nret\n.end";
    let modules = assemble_modules(&[app, library]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap();
    assert!(
        assemble_modules(&[app, &library.replace("public Outer", "internal Outer")])
            .unwrap_err()
            .message
            .contains("access denied")
    );
}

#[test]
fn malformed_ownership_and_unsupported_contexts_are_rejected() {
    let module = assemble(SAMPLE).unwrap();
    for mode in 0..5 {
        let mut bad = module.clone();
        let index = bad
            .types
            .iter()
            .position(|d| d.name == "Demo.Result.Ok")
            .unwrap();
        match mode {
            0 => bad.types[index].declaring_type.as_mut().unwrap().index = 999,
            1 => bad.types[index].declaring_type = bad.types[index].definition.clone(),
            2 => bad.types[index].name = "Unrelated.Case".into(),
            3 => bad.types[index].declaring_type.as_mut().unwrap().module = "Other".into(),
            _ => {
                bad.types[index].declaring_type = bad
                    .types
                    .iter()
                    .find(|d| d.name == "Demo.Result" && d.generic_parameters.len() == 2)
                    .unwrap()
                    .definition
                    .clone()
            }
        }
        assert!(
            neoclr::load(&serde_json::to_string(&bad).unwrap()).is_err(),
            "mode {mode}"
        );
    }
    for source in [
        ".module Bad\n.type Outer<T>\n.type Case<U>\n.end\n.end",
        ".module Bad\n.type Outer\n.type A.B\n.end\n.end",
        ".module Bad\n.type Outer\n.type private Case\n.end\n.end",
    ] {
        assert!(assemble(source).is_err());
    }
}
