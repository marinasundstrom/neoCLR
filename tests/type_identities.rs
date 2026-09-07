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
        resolve_type_identity(&module, &Type::Ref(Box::new(Type::Void))).unwrap(),
        TypeIdentity::Ref(Box::new(void.clone()))
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
