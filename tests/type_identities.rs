use neoclr::{
    TypeIdentity, assemble, library, load,
    metadata::{Type, TypeDefId},
    resolve_type_identity, resolve_type_identity_with_library,
};

fn application() -> neoclr::Module {
    assemble(".module App\n.type Box<T>\n.field Value T\n.end\n.type Marker\n.end").unwrap()
}

fn boxed(argument: Type) -> Type {
    Type::Constructed {
        definition: "Box".into(),
        arguments: vec![argument],
    }
}

#[test]
fn definition_rows_roundtrip_independently_of_function_rows() {
    let module = assemble(".module App\n.type Marker\n.method static Test() -> Void\nldvoid\nret\n.end\n.end\n.type Other\n.end").unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        loaded.types[0].definition,
        Some(TypeDefId {
            module: "App".into(),
            revision: None,
            index: 0
        })
    );
    assert_eq!(loaded.types[1].definition.as_ref().unwrap().index, 1);
    assert_eq!(loaded.functions[0].definition.as_ref().unwrap().index, 0);
    for id in [
        serde_json::json!({"module":"System","index":0}),
        serde_json::json!({"module":"App","index":1}),
    ] {
        let mut json = serde_json::to_value(&module).unwrap();
        json["types"][0]["definition"] = id;
        assert!(
            load(&json.to_string())
                .unwrap_err()
                .message
                .contains("noncanonical type definition identity")
        );
    }
}

#[test]
fn closed_keys_preserve_definition_and_distinguish_arguments() {
    let module = application();
    let before = serde_json::to_value(&module).unwrap();
    let int = resolve_type_identity(&module, &Type::Int32).unwrap();
    let boxed_int = resolve_type_identity(&module, &boxed(Type::Int32)).unwrap();
    assert_eq!(
        boxed_int,
        TypeIdentity::Definition {
            definition: TypeDefId {
                module: "App".into(),
                revision: None,
                index: 0
            },
            arguments: vec![int],
        }
    );
    assert_ne!(
        boxed_int,
        resolve_type_identity(&module, &boxed(Type::String)).unwrap()
    );
    assert_eq!(
        resolve_type_identity(&module, &boxed(boxed(Type::Int32))).unwrap(),
        TypeIdentity::Definition {
            definition: TypeDefId {
                module: "App".into(),
                revision: None,
                index: 0
            },
            arguments: vec![boxed_int],
        }
    );
    assert_eq!(serde_json::to_value(&module).unwrap(), before);
}

#[test]
fn system_rows_and_primitive_aliases_survive_linking() {
    let module = application();
    let system = library::system().unwrap();
    let int = resolve_type_identity(system, &Type::Int32).unwrap();
    for alias in ["int32", "int", "Int32", "System.Int32"] {
        assert_eq!(
            resolve_type_identity(&module, &Type::from_name(alias)).unwrap(),
            int
        );
    }
    let row = system
        .types
        .iter()
        .position(|d| d.name == "System.Int32")
        .unwrap();
    assert_eq!(
        int,
        TypeIdentity::Definition {
            definition: TypeDefId {
                module: "System".into(),
                revision: None,
                index: row as u32
            },
            arguments: vec![]
        }
    );
    assert_ne!(int, resolve_type_identity(&module, &Type::UInt32).unwrap());
}

#[test]
fn structural_wrappers_remain_distinct_and_support_void() {
    let module = application();
    let void = resolve_type_identity(&module, &Type::Void).unwrap();
    assert_eq!(
        resolve_type_identity(&module, &Type::Ptr(Box::new(Type::Void))).unwrap(),
        TypeIdentity::Ptr(Box::new(void.clone()))
    );
    assert_eq!(
        resolve_type_identity(&module, &Type::ByRef(Box::new(Type::Void))).unwrap(),
        TypeIdentity::ByRef(Box::new(void.clone()))
    );
}

#[test]
fn invalid_or_open_signatures_cannot_produce_closed_keys() {
    let module = application();
    for ty in [
        Type::Named("Missing".into()),
        Type::Named("Box".into()),
        Type::Named("System.Int32".into()),
        Type::TypeParameter(0),
        boxed(Type::TypeParameter(0)),
        Type::Constructed {
            definition: "Box".into(),
            arguments: vec![],
        },
        Type::Constructed {
            definition: "Marker".into(),
            arguments: vec![Type::Int32],
        },
    ] {
        assert!(resolve_type_identity(&module, &ty).is_err(), "{ty:?}");
    }
    let mut deep = Type::Int32;
    for _ in 0..34 {
        deep = Type::Ptr(Box::new(deep));
    }
    assert!(
        resolve_type_identity(&module, &deep)
            .unwrap_err()
            .message
            .contains("nesting")
    );
}

#[test]
fn legacy_rows_are_derived_in_their_original_modules() {
    let module = application();
    let expected = resolve_type_identity(&module, &boxed(Type::Int32)).unwrap();
    let strip = |module: &neoclr::Module| {
        let mut json = serde_json::to_value(module).unwrap();
        for definition in json["types"].as_array_mut().unwrap() {
            definition.as_object_mut().unwrap().remove("definition");
        }
        load(&json.to_string()).unwrap()
    };
    let legacy = strip(&module);
    let system = strip(library::system().unwrap());
    assert!(legacy.types.iter().all(|d| d.definition.is_none()));
    assert_eq!(
        resolve_type_identity_with_library(&legacy, &system, &boxed(Type::Int32)).unwrap(),
        expected
    );
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_keys_preserve_kind_constituents_and_generic_arguments() {
    use neoclr::{LoadedProgram, assembler::parse_type};
    let module = assemble(".module Compositions\n.revision first\n.interface Left\n.end\n.interface Right\n.end\n.type class Box<T>\n.end\n").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let identity = |text| {
        program
            .resolve_type_identity(&parse_type(text).unwrap())
            .unwrap()
    };
    let left = identity("Left");
    let right = identity("Right");
    let union = identity("union<Left,Right>");
    let intersection = identity("intersection<Left,Right>");
    assert_eq!(
        union,
        TypeIdentity::Union(vec![left.clone(), right.clone()])
    );
    assert_eq!(
        intersection,
        TypeIdentity::Intersection(vec![left.clone(), right.clone()])
    );
    assert_ne!(union, intersection);
    assert_eq!(union, identity("union<Right,Left>"));
    assert_eq!(union, identity("union<Left,union<Right,Left>>"));
    assert_eq!(left, identity("union<Left,Left>"));
    let TypeIdentity::Definition { arguments, .. } = identity("Box<union<Left,Right>>") else {
        panic!("expected wrapper")
    };
    assert_eq!(arguments, vec![union.clone()]);
    let mixed = identity("union<Left,intersection<Left,Right>>");
    let TypeIdentity::Union(parts) = mixed else {
        panic!("expected mixed union")
    };
    assert!(parts.contains(&left));
    assert!(parts.contains(&intersection));
    let other = assemble(".module Compositions\n.revision second\n.interface Left\n.end\n.interface Right\n.end\n.type class Box<T>\n.end\n").unwrap();
    assert_ne!(
        union,
        resolve_type_identity(&other, &parse_type("union<Left,Right>").unwrap()).unwrap()
    );
    assert!(
        program
            .resolve_type_identity(&Type::Union(vec![Type::Int32, Type::from_name("Left")]))
            .is_err()
    );
}

#[cfg(not(feature = "experimental-compositions"))]
#[test]
fn composition_identity_resolution_stays_opt_in() {
    let module =
        assemble(".module Compositions\n.interface Left\n.end\n.interface Right\n.end\n").unwrap();
    let parts = vec![Type::from_name("Left"), Type::from_name("Right")];
    assert!(resolve_type_identity(&module, &Type::Union(parts.clone())).is_err());
    assert!(resolve_type_identity(&module, &Type::Intersection(parts.clone())).is_err());
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    assert!(program.describe_type(&Type::Union(parts)).is_err());
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_descriptors_expose_recursive_constituents_separately_from_arguments() {
    use neoclr::{LoadedProgram, assembler::parse_type};
    let module = assemble(
        ".module Shapes\n.interface Left\n.end\n.interface Right\n.end\n.type class Box<T>\n.end\n",
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let describe = |text| program.describe_type(&parse_type(text).unwrap()).unwrap();
    for (text, union) in [
        ("union<Left,Right>", true),
        ("intersection<Left,Right>", false),
    ] {
        let descriptor = describe(text);
        assert_eq!(descriptor.name, text);
        assert_eq!(matches!(descriptor.identity, TypeIdentity::Union(_)), union);
        assert!(descriptor.declaring_type.is_none());
        assert!(descriptor.generic_arguments.is_empty());
        assert_eq!(
            descriptor.constituents,
            vec![describe("Left"), describe("Right")]
        );
    }
    let wrapper = describe("Box<union<Left,intersection<Left,Right>>>");
    assert!(matches!(wrapper.identity, TypeIdentity::Definition { .. }));
    assert!(wrapper.constituents.is_empty());
    assert_eq!(wrapper.generic_arguments.len(), 1);
    let argument = &wrapper.generic_arguments[0];
    assert!(matches!(argument.identity, TypeIdentity::Union(_)));
    assert_eq!(argument.constituents.len(), 2);
    assert!(argument.constituents.contains(&describe("Left")));
    assert!(
        argument
            .constituents
            .contains(&describe("intersection<Left,Right>"))
    );
    let generic_bound = describe("intersection<Box<Int32>,Left>");
    let bound = generic_bound
        .constituents
        .iter()
        .find(|p| p.name == "Box")
        .unwrap();
    assert_eq!(bound.generic_arguments, vec![describe("Int32")]);
    assert_eq!(describe("union<Left,Left>"), describe("Left"));
    assert_eq!(describe("union<Right,Left>"), describe("union<Left,Right>"));
    assert!(describe("Left").constituents.is_empty());
    assert!(
        program
            .describe_type(&Type::Union(vec![Type::Int32, Type::from_name("Left")]))
            .is_err()
    );
}
