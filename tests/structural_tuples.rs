//! Native signature migration foundation; execution remains deliberately unadmitted.
use neoclr::{assemble, metadata::Type, metadata_container};

fn tuple_module(ty: Type) -> neoclr::Module {
    let mut module = assemble(
        ".module StructuralTuples\n.entry Main\n.function Main() -> Int32\nldc.i4 0\nret\n.end",
    )
    .unwrap();
    module.functions[0].returns = ty;
    module
}

#[test]
fn tuple_signatures_roundtrip_without_nominal_tuple_declarations() {
    let ty = Type::Tuple(vec![
        Type::Int32,
        Type::String,
        Type::Tuple(vec![Type::Boolean, Type::Int32]),
    ]);
    let module = tuple_module(ty.clone());
    let image = metadata_container::write_module(&module).unwrap();
    let decoded = metadata_container::decode_envelope(&image).unwrap();
    assert_eq!(decoded.functions[0].returns, ty);
    assert!(decoded.types.iter().all(|t| !t.name.contains("Tuple")));
    assert_eq!(ty.definition_name(), None);
    let json = serde_json::to_string(&module).unwrap();
    let decoded: neoclr::Module = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.functions[0].returns, ty);
}

#[test]
fn tuple_signature_equality_preserves_position_multiplicity_and_nesting() {
    let pair = Type::Tuple(vec![Type::Int32, Type::String]);
    assert_ne!(pair, Type::Tuple(vec![Type::String, Type::Int32]));
    assert_ne!(
        Type::Tuple(vec![Type::Int32; 2]),
        Type::Tuple(vec![Type::Int32])
    );
    assert_ne!(Type::Tuple(vec![pair.clone()]), pair);
    assert_ne!(Type::Tuple(vec![Type::Int32]), Type::Int32);
}

#[test]
fn tuple_substitution_traverses_both_generic_scopes_and_self() {
    let open = Type::Tuple(vec![
        Type::TypeParameter(0),
        Type::Tuple(vec![Type::MethodTypeParameter(0), Type::SelfType]),
    ]);
    assert!(open.contains_self());
    let closed = open
        .substitute_type_parameters(&[Type::Int32])
        .unwrap()
        .substitute_method_parameters(&[Type::String])
        .unwrap()
        .substitute_self(&Type::Boolean)
        .unwrap();
    assert_eq!(
        closed,
        Type::Tuple(vec![
            Type::Int32,
            Type::Tuple(vec![Type::String, Type::Boolean])
        ])
    );
    assert!(!closed.contains_self());
    assert!(open.substitute_type_parameters(&[]).is_err());
    assert!(open.substitute_method_parameters(&[]).is_err());
}

#[test]
fn tuple_substitution_retains_existing_depth_budget() {
    let mut ty = Type::TypeParameter(0);
    for _ in 0..34 {
        ty = Type::Tuple(vec![ty]);
    }
    assert!(ty.substitute_type_parameters(&[Type::Int32]).is_err());
}

#[test]
fn structural_tuple_execution_is_explicitly_rejected_during_migration() {
    for ty in [
        Type::Tuple(vec![Type::Int32, Type::String]),
        Type::ArrayRef(Box::new(Type::Tuple(vec![Type::Int32]))),
    ] {
        let image = metadata_container::write_module(&tuple_module(ty)).unwrap();
        let error = metadata_container::load_envelope(&image).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("structural tuple execution is not yet supported"),
            "{error}"
        );
    }
}

#[test]
fn invalid_tuple_arity_fails_during_scope_normalization() {
    for parts in [vec![], vec![Type::Int32; 257]] {
        let image = metadata_container::write_module(&tuple_module(Type::Tuple(parts))).unwrap();
        let error = metadata_container::load_envelope(&image).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("tuple signature requires one to 256 elements"),
            "{error}"
        );
    }
}
