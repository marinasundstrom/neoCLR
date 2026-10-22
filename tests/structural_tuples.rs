//! Native structural tuple metadata and interpreter contracts.
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
    let ty = Type::Tuple(
        vec![
            Type::Int32,
            Type::String,
            Type::Tuple(vec![Type::Boolean, Type::Int32].into()),
        ]
        .into(),
    );
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
    let pair = Type::Tuple(vec![Type::Int32, Type::String].into());
    assert_ne!(pair, Type::Tuple(vec![Type::String, Type::Int32].into()));
    assert_ne!(
        Type::Tuple(vec![Type::Int32; 2].into()),
        Type::Tuple(vec![Type::Int32].into())
    );
    assert_ne!(Type::Tuple(vec![pair.clone()].into()), pair);
    assert_ne!(Type::Tuple(vec![Type::Int32].into()), Type::Int32);
}

#[test]
fn tuple_substitution_traverses_both_generic_scopes_and_self() {
    let open = Type::Tuple(
        vec![
            Type::TypeParameter(0),
            Type::Tuple(vec![Type::MethodTypeParameter(0), Type::SelfType].into()),
        ]
        .into(),
    );
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
        Type::Tuple(
            vec![
                Type::Int32,
                Type::Tuple(vec![Type::String, Type::Boolean].into())
            ]
            .into()
        )
    );
    assert!(!closed.contains_self());
    assert!(open.substitute_type_parameters(&[]).is_err());
    assert!(open.substitute_method_parameters(&[]).is_err());
}

#[test]
fn tuple_substitution_retains_existing_depth_budget() {
    let mut ty = Type::TypeParameter(0);
    for _ in 0..34 {
        ty = Type::Tuple(vec![ty].into());
    }
    assert!(ty.substitute_type_parameters(&[Type::Int32]).is_err());
}

#[test]
fn invalid_tuple_arity_fails_during_scope_normalization() {
    for parts in [vec![], vec![Type::Int32; 257]] {
        let image =
            metadata_container::write_module(&tuple_module(Type::Tuple(parts.into()))).unwrap();
        let error = metadata_container::load_envelope(&image).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("tuple signature requires one to 256 elements"),
            "{error}"
        );
    }
}

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

#[test]
fn tuple_reference_element_survives_collection_and_retains_identity() {
    let helpers = ".type class Cell\n.field Number Int32\n.end\n.function neoCLR.Runtime.GCCollect() -> void\n.methodimpl InternalCall\n.end";
    assert_eq!(
        execute(
            ".local tuple<Cell> pair\nldc.i4 40\nnewobj Cell\nnewobj tuple<Cell>\nstloc pair\ncall neoCLR.Runtime.GCCollect()\nldloc pair\nldfld 0\nldc.i4 42\nstfld 0\nldloc pair\nldfld 0\nldfld 0",
            helpers
        ),
        neoclr::Value::Int32(42)
    );
}

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
        if let Ok(program) = neoclr::LoadedProgram::new(&module) {
            assert!(program.verify().is_err());
        }
    }
}

#[test]
fn tuple_admission_rejects_void_borrows_and_open_elements() {
    for ty in [
        Type::Void,
        Type::ByRef(Box::new(Type::Int32)),
        Type::MethodTypeParameter(0),
    ] {
        let module = tuple_module(Type::Tuple(vec![ty].into()));
        let image = metadata_container::write_module(&module).unwrap();
        let error = metadata_container::load_envelope(&image).unwrap_err();
        assert!(
            error.to_string().contains("tuple elements require closed"),
            "{error}"
        );
    }
}

fn named_tuple(elements: Vec<Type>, names: &[Option<&str>]) -> Type {
    Type::Tuple(
        neoclr::metadata::TupleSignature::new(
            elements,
            names.iter().map(|name| name.map(str::to_owned)).collect(),
        )
        .unwrap(),
    )
}

#[test]
fn native_tuple_names_roundtrip_without_changing_equality_or_hashing() {
    use std::hash::{Hash, Hasher};
    let first = named_tuple(
        vec![Type::Int32, Type::String],
        &[Some("count"), Some("beskrivning")],
    );
    let renamed = named_tuple(
        vec![Type::Int32, Type::String],
        &[Some("size"), Some("text")],
    );
    let positional = Type::Tuple(vec![Type::Int32, Type::String].into());
    assert_eq!(first, renamed);
    assert_eq!(first, positional);
    let hash = |ty: &Type| {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        ty.hash(&mut hash);
        hash.finish()
    };
    assert_eq!(hash(&first), hash(&renamed));
    assert_eq!(hash(&first), hash(&positional));
    assert_ne!(
        first,
        named_tuple(
            vec![Type::String, Type::Int32],
            &[Some("count"), Some("beskrivning")]
        )
    );
    let nested = named_tuple(vec![first.clone(), renamed], &[Some("original"), None]);
    let image = metadata_container::write_module(&tuple_module(nested.clone())).unwrap();
    let decoded = metadata_container::decode_envelope(&image).unwrap();
    // Structural equality alone cannot detect loss of names.
    assert_eq!(
        serde_json::to_value(&decoded.functions[0].returns).unwrap(),
        serde_json::to_value(&nested).unwrap()
    );
    let json = serde_json::to_value(first).unwrap();
    assert_eq!(json["Tuple"]["names"][0], "count");
    assert_eq!(
        serde_json::to_value(positional).unwrap(),
        serde_json::json!({"Tuple":["Int32","String"]})
    );
}

#[test]
fn tuple_name_validation_and_substitution_preserve_native_metadata() {
    use neoclr::metadata::TupleSignature;
    for names in [
        vec![Some("x")],
        vec![Some(""), None],
        vec![Some("x"), Some("x")],
        vec![Some("Item2"), None],
        vec![Some("a\0b"), None],
    ] {
        assert!(
            TupleSignature::new(
                vec![Type::Int32, Type::Int32],
                names.into_iter().map(|n| n.map(str::to_owned)).collect()
            )
            .is_err()
        );
    }
    assert!(TupleSignature::new(vec![Type::Int32], vec![Some("x".repeat(1025))]).is_err());
    assert!(
        serde_json::from_value::<Type>(
            serde_json::json!({"Tuple":{"elements":["Int32"],"names":[] ,"unexpected":true}})
        )
        .is_err()
    );
    let open = named_tuple(
        vec![Type::TypeParameter(0), Type::SelfType],
        &[Some("value"), Some("owner")],
    );
    let closed = open
        .substitute_type_parameters(&[Type::String])
        .unwrap()
        .substitute_self(&Type::Int32)
        .unwrap();
    let Type::Tuple(shape) = closed else {
        panic!("tuple lost")
    };
    assert_eq!(shape.names(), &[Some("value".into()), Some("owner".into())]);
    assert_eq!(&*shape, &[Type::String, Type::Int32]);
}

#[test]
fn named_tuple_descriptors_keep_use_site_names_with_equal_identities() {
    let module = assemble(
        ".module NamedTupleInfo\n.entry Main\n.function Main() -> Int32\nldc.i4 42\nret\n.end",
    )
    .unwrap();
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    let a = named_tuple(vec![Type::Int32, Type::Int32], &[Some("x"), Some("y")]);
    let b = named_tuple(
        vec![Type::Int32, Type::Int32],
        &[Some("width"), Some("height")],
    );
    let a = program.describe_type(&a).unwrap();
    let b = program.describe_type(&b).unwrap();
    assert_eq!(a.identity, b.identity);
    assert_eq!(
        a.tuple_element_names,
        vec![Some("x".into()), Some("y".into())]
    );
    assert_eq!(
        b.tuple_element_names,
        vec![Some("width".into()), Some("height".into())]
    );
    assert_eq!(a.tuple_elements, b.tuple_elements);
}

#[test]
fn named_tuple_values_accept_renamed_parameter_signatures() {
    let mut module = assemble(".module NamedTupleValues\n.entry Main\n.function Sum(tuple<Int32,Int32> pair) -> Int32\nldarg pair\nldfld 0\nldarg pair\nldfld 1\nadd\nret\n.end\n.function Main() -> Int32\nldc.i4 40\nldc.i4 2\nnewobj tuple<Int32,Int32>\ncall Sum(tuple<Int32,Int32>)\nret\n.end").unwrap();
    for function in &mut module.functions {
        if function.name == "Sum" {
            function.parameters[0] = named_tuple(
                vec![Type::Int32, Type::Int32],
                &[Some("width"), Some("height")],
            );
        }
        for instruction in &mut function.body {
            if let neoclr::metadata::Instruction::New(ty) = instruction {
                *ty = named_tuple(vec![Type::Int32, Type::Int32], &[Some("x"), Some("y")]);
            }
        }
    }
    let image = metadata_container::write_module(&module).unwrap();
    let module = metadata_container::load_envelope(&image).unwrap();
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(42)
    );
}
