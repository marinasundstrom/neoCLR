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

#[cfg(not(feature = "experimental-tuples"))]
#[test]
fn structural_tuple_execution_requires_explicit_opt_in() {
    for ty in [
        Type::Tuple(vec![Type::Int32, Type::String]),
        Type::ArrayRef(Box::new(Type::Tuple(vec![Type::Int32]))),
    ] {
        let image = metadata_container::write_module(&tuple_module(ty)).unwrap();
        let error = metadata_container::load_envelope(&image).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("structural tuple execution requires experimental-tuples"),
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

#[cfg(feature = "experimental-tuples")]
fn execute(body: &str, helpers: &str) -> neoclr::Value {
    let text = format!(
        ".module TupleValues\n.entry Main\n{helpers}\n.function Main() -> Int32\n{body}\nret\n.end"
    );
    let module = assemble(&text).unwrap();
    let image = metadata_container::write_module(&module).unwrap();
    let module = metadata_container::load_envelope(&image).unwrap();
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program.run(neoclr::Limits::default()).unwrap().value
}

#[cfg(feature = "experimental-tuples")]
#[test]
fn tuple_value_constructs_and_crosses_function_boundary() {
    assert_eq!(
        execute(
            "ldc.i4 40\nldc.i4 2\nnewobj tuple<Int32,Int32>\ncall Sum(tuple<Int32,Int32>)",
            ".function Sum(tuple<Int32,Int32> pair) -> Int32\nldarg pair\nldfld 0\nldarg pair\nldfld 1\nadd\nret\n.end"
        ),
        neoclr::Value::Int32(42)
    );
}

#[cfg(feature = "experimental-tuples")]
#[test]
fn tuple_value_mutation_preserves_original_copy() {
    assert_eq!(
        execute(
            ".local tuple<Int32,Int32> original\nldc.i4 40\nldc.i4 2\nnewobj tuple<Int32,Int32>\nstloc original\nldloc original\nldc.i4 7\nstfld 0\nldfld 0\nldloc original\nldfld 0\nadd",
            ""
        ),
        neoclr::Value::Int32(47)
    );
}

#[cfg(feature = "experimental-tuples")]
#[test]
fn tuple_nested_elements_and_synthetic_names_work() {
    assert_eq!(
        execute(
            ".local Int32 number\nldc.i4 40\nldc.i4 2\nnewobj tuple<Int32,Int32>\nnewobj tuple<tuple<Int32,Int32>>\nldfld tuple<tuple<Int32,Int32>>::Item1\ndup\nldfld 0\nstloc number\nldfld 1\nldloc number\nadd",
            ""
        ),
        neoclr::Value::Int32(42)
    );
}

#[cfg(feature = "experimental-tuples")]
#[test]
fn tuple_default_and_managed_element_address_preserve_storage() {
    assert_eq!(
        execute(
            ".local tuple<Int32,Int32> pair\nldloca pair\ninitobj tuple<Int32,Int32>\nldloca pair\nldflda 0\nldc.i4 42\nstobj Int32\nldloc pair\nldfld 0",
            ""
        ),
        neoclr::Value::Int32(42)
    );
}

#[cfg(feature = "experimental-tuples")]
#[test]
fn tuple_reference_element_survives_collection_and_retains_identity() {
    let helpers = ".type class Cell\n.field Number Int32\n.end\n.function neoCLR.Runtime.GCCollect() -> Void\n.methodimpl InternalCall\n.end";
    assert_eq!(
        execute(
            ".local tuple<Cell> pair\nldc.i4 40\nnewobj Cell\nnewobj tuple<Cell>\nstloc pair\ncall neoCLR.Runtime.GCCollect()\npop\nldloc pair\nldfld 0\nldc.i4 42\nstfld 0\nldloc pair\nldfld 0\nldfld 0",
            helpers
        ),
        neoclr::Value::Int32(42)
    );
}

#[cfg(feature = "experimental-tuples")]
#[test]
fn tuple_verifier_rejects_wrong_elements_and_positions() {
    for body in [
        "ldstr \"wrong\"\nnewobj tuple<Int32>\npop\nldc.i4 0",
        "ldc.i4 1\nnewobj tuple<Int32>\nldfld 1",
        "ldc.i4 1\nnewobj tuple<Int32>\nldstr \"wrong\"\nstfld 0\npop\nldc.i4 0",
    ] {
        let module = assemble(&format!(
            ".module InvalidTuple\n.entry Main\n.function Main() -> Int32\n{body}\nret\n.end"
        ))
        .unwrap();
        match neoclr::LoadedProgram::new(&module) {
            Ok(program) => assert!(program.verify().is_err()),
            Err(_) => (),
        }
    }
}

#[cfg(feature = "experimental-tuples")]
#[test]
fn tuple_admission_rejects_void_borrows_and_open_elements() {
    for ty in [
        Type::Void,
        Type::ByRef(Box::new(Type::Int32)),
        Type::MethodTypeParameter(0),
    ] {
        let module = tuple_module(Type::Tuple(vec![ty]));
        let image = metadata_container::write_module(&module).unwrap();
        let error = metadata_container::load_envelope(&image).unwrap_err();
        assert!(
            error.to_string().contains("tuple elements require closed"),
            "{error}"
        );
    }
}
