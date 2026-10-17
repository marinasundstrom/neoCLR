use neoclr::assembler::parse_type;
use neoclr::metadata::{ConstraintKind, Type};
use neoclr::{Limits, LoadedProgram, Value, assemble, verify};

const SOURCE: &str = include_str!("../docs/validation/function-kind-constraint/Main.neoil");

#[test]
fn function_kind_preserves_signature_and_typed_invocation() {
    let source = assemble(SOURCE).unwrap();
    verify(&source).unwrap();
    assert_eq!(
        source.types[0].generic_constraints[0].kind,
        ConstraintKind::FunctionType
    );
    let json = serde_json::to_string(&source).unwrap();
    let loaded = neoclr::load(&json).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    source
        .instantiated_fields(&parse_type("Signal<fn<fn<Int32,Int32>,Int32>>").unwrap())
        .unwrap();
}

#[test]
fn nominal_types_and_containers_of_functions_are_not_function_types() {
    let source = assemble(SOURCE).unwrap();
    for argument in [
        Type::Int32,
        Type::String,
        parse_type("fn<Int32,Int32>[]").unwrap(),
    ] {
        let construction = Type::Constructed {
            definition: "Signal".into(),
            arguments: vec![argument],
        };
        assert!(source.instantiated_fields(&construction).is_err());
    }
    let invalid = SOURCE.replace("Signal<fn<Int32,Int32>>", "Signal<Int32>");
    assert!(assemble(&invalid).and_then(|m| verify(&m)).is_err());
}

#[test]
fn incompatible_function_kind_combinations_are_rejected() {
    for extra in ["reference", "value", "new"] {
        assert!(
            assemble(&SOURCE.replace(".constraint F func", &format!(".constraint F func {extra}")))
                .is_err()
        );
    }
}
