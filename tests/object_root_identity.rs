use neoclr::{
    Limits, LoadedProgram, Module, Value,
    assembler::{ModuleInput, read_modules},
};

const ROOT: &str = r#"
.module System
.references ()
.type class abstract System.Object
.method instance virtual ToString() -> String
ldstr "root"
ret
.end
.method instance virtual Equals(System.Object other) -> Boolean
ldc.bool false
ret
.end
.method instance virtual GetHashCode() -> Int32
ldc.i4 -1
ret
.end
.end
"#;

fn modules(body: &str, declarations: &str) -> (Module, Vec<Module>) {
    let seed = neoclr::assemble(ROOT).unwrap();
    let app = format!(
        ".module App\n.references (System)\n.entry Main\n{declarations}\n.function Main() -> String\n{body}\nret\n.end"
    );
    let modules = read_modules(&[ModuleInput::Source(&app)], &seed).unwrap();
    (seed, modules)
}

#[test]
fn boxed_display_uses_the_loaded_root_definition() {
    let (seed, modules) = modules(
        "ldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::ToString()",
        "",
    );
    let before = serde_json::to_value(&modules).unwrap();
    let program = LoadedProgram::with_modules(&modules[0], &seed, &modules[1..]).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("42".into())
    );
    assert_eq!(serde_json::to_value(&modules).unwrap(), before);
}

#[test]
fn rootless_override_uses_the_loaded_root_slot() {
    let (seed, modules) = modules(
        "newobj Named\ncallvirt instance [System]System.Object::ToString()",
        ".type class Named\n.method instance override ToString() -> String\nldstr \"named\"\nret\n.end\n.end",
    );
    let program = LoadedProgram::with_modules(&modules[0], &seed, &modules[1..]).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("named".into())
    );
}

#[test]
fn boxed_equality_and_hashing_use_the_loaded_root_definition() {
    for body in [
        "ldc.i4 42\nbox Int32\nldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::Equals(System.Object)\nbrfalse failed\nldstr \"passed\"\nret\nfailed:\nldstr \"failed\"",
        "ldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::GetHashCode()\nldc.i4 42\nceq\nbrfalse failed\nldstr \"passed\"\nret\nfailed:\nldstr \"failed\"",
    ] {
        let (seed, modules) = modules(body, "");
        let program = LoadedProgram::with_modules(&modules[0], &seed, &modules[1..]).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(Limits::default()).unwrap().value,
            Value::String("passed".into())
        );
    }
}

#[test]
fn root_slot_identity_does_not_relax_duplicate_or_forged_ownership() {
    let (seed, modules) = modules("ldstr \"unused\"", "");
    let duplicate = neoclr::assemble(&ROOT.replace(".module System", ".module Core")).unwrap();
    assert!(LoadedProgram::with_modules(&modules[0], &seed, &[duplicate]).is_err());
    let mut forged = seed.clone();
    forged.functions[0].definition.as_mut().unwrap().module = "Core".into();
    assert!(LoadedProgram::with_modules(&modules[0], &forged, &[]).is_err());
}

#[test]
fn root_slots_cannot_be_called_through_the_wrong_scope() {
    let seed = neoclr::assemble(ROOT).unwrap();
    let app = ".module App\n.references (System)\n.entry Main\n.function Main() -> String\nldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()\nret\n.end";
    assert!(read_modules(&[ModuleInput::Source(app)], &seed).is_err());
}

// Until the host/writer/runtime have an explicit root-ownership contract, a
// declaration's name is insufficient to opt it into intrinsic Object behavior.
#[test]
fn external_object_spelling_does_not_select_a_new_runtime_root() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let core = ROOT.replace(".module System", ".module Core");
    let app = ".module App\n.references (Core)\n.entry Main\n.function Main() -> String\nldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()\nret\n.end";
    let loaded = read_modules(
        &[ModuleInput::Source(app), ModuleInput::Source(&core)],
        &seed,
    )
    .unwrap();
    let program = LoadedProgram::with_modules(&loaded[0], &seed, &loaded[1..]).unwrap();
    program.verify().unwrap();
    assert!(program.run(Limits::default()).is_err());
}

#[test]
fn binary_load_set_preserves_selected_seed_root_dispatch() {
    let (seed, modules) = modules(
        "ldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::ToString()",
        "",
    );
    let decoded: Vec<_> = modules
        .iter()
        .map(|module| {
            let image = neoclr::metadata_container::write_module(module).unwrap();
            neoclr::metadata_container::decode_envelope(&image).unwrap()
        })
        .collect();
    let seed_image = neoclr::metadata_container::write_module(&seed).unwrap();
    let decoded_seed = neoclr::metadata_container::decode_envelope(&seed_image).unwrap();
    let program = LoadedProgram::with_modules(&decoded[0], &decoded_seed, &decoded[1..]).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("42".into())
    );
}

fn selected_root() -> neoclr::metadata::TypeDefId {
    neoclr::metadata::TypeDefId {
        module: "Core".into(),
        revision: None,
        index: 0,
    }
}

fn selected_sources(body: &str, declarations: &str) -> (Module, Vec<Module>) {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let core = ROOT.replace(".module System", ".module Core");
    let app = format!(
        ".module App\n.references (Core)\n.entry Main\n{declarations}\n.function Main() -> String\n{body}\nret\n.end"
    );
    let modules = neoclr::assembler::read_modules_with_object_root(
        &[ModuleInput::Source(&app), ModuleInput::Source(&core)],
        &seed,
        &selected_root(),
    )
    .unwrap();
    (seed, modules)
}

#[test]
fn explicit_external_root_executes_boxing_and_rootless_override_dispatch() {
    for (body, declarations, expected) in [
        (
            "ldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()",
            "",
            "42",
        ),
        (
            "newobj Named\ncallvirt instance [Core]System.Object::ToString()",
            ".type class Named\n.method instance override ToString() -> String\nldstr \"named\"\nret\n.end\n.end",
            "named",
        ),
        (
            "ldc.i4 42\nbox Int32\nldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::Equals(System.Object)\nbrfalse failed\nldstr \"equal\"\nret\nfailed:\nldstr \"failed\"",
            "",
            "equal",
        ),
        (
            "ldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::GetHashCode()\nldc.i4 42\nceq\nbrfalse failed\nldstr \"hashed\"\nret\nfailed:\nldstr \"failed\"",
            "",
            "hashed",
        ),
    ] {
        let (seed, modules) = selected_sources(body, declarations);
        let before = serde_json::to_value(&modules).unwrap();
        let program = LoadedProgram::with_modules_and_object_root(
            &modules[0],
            &seed,
            &modules[1..],
            &selected_root(),
        )
        .unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(Limits::default()).unwrap().value,
            Value::String(expected.into())
        );
        assert_eq!(serde_json::to_value(&modules).unwrap(), before);
    }
}

#[test]
fn explicit_root_rejects_wrong_identity_shape_and_incomplete_slots() {
    let (seed, modules) = selected_sources("ldstr \"unused\"", "");
    for root in [
        neoclr::metadata::TypeDefId {
            module: "Missing".into(),
            ..selected_root()
        },
        neoclr::metadata::TypeDefId {
            index: 1,
            ..selected_root()
        },
        neoclr::metadata::TypeDefId {
            revision: Some("wrong".into()),
            ..selected_root()
        },
        neoclr::metadata::TypeDefId {
            module: "App".into(),
            ..selected_root()
        },
    ] {
        assert!(
            LoadedProgram::with_modules_and_object_root(&modules[0], &seed, &modules[1..], &root)
                .is_err()
        );
    }
    for mutation in 0..6 {
        let mut invalid = modules.clone();
        match mutation {
            0 => invalid[1].types[0].is_abstract = false,
            1 => invalid[1].types[0].visibility = neoclr::metadata::Visibility::Internal,
            2 => {
                invalid[1].functions.pop();
            }
            3 => invalid[1].functions[1].is_virtual = false,
            4 => invalid[1].functions[1].returns = neoclr::metadata::Type::Int32,
            _ => invalid[1].functions[1].is_override = true,
        }
        assert!(
            LoadedProgram::with_modules_and_object_root(
                &invalid[0],
                &seed,
                &invalid[1..],
                &selected_root()
            )
            .is_err()
        );
    }
    let competing_seed = neoclr::assemble(ROOT).unwrap();
    assert!(
        LoadedProgram::with_modules_and_object_root(
            &modules[0],
            &competing_seed,
            &modules[1..],
            &selected_root()
        )
        .is_err()
    );
}

#[test]
fn binary_artifacts_require_host_root_selection_again() {
    let (seed, modules) = selected_sources(
        "ldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()",
        "",
    );
    let images: Vec<_> = modules
        .iter()
        .map(|m| neoclr::metadata_container::write_module(m).unwrap())
        .collect();
    let inputs: Vec<_> = images
        .iter()
        .map(|image| ModuleInput::NativeEnvelope(image))
        .collect();
    let decoded =
        neoclr::assembler::read_modules_with_object_root(&inputs, &seed, &selected_root()).unwrap();
    let program = LoadedProgram::with_modules_and_object_root(
        &decoded[0],
        &seed,
        &decoded[1..],
        &selected_root(),
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("42".into())
    );
    let legacy = LoadedProgram::with_modules(&decoded[0], &seed, &decoded[1..]).unwrap();
    assert!(legacy.run(Limits::default()).is_err());
}

#[test]
fn selected_root_allows_validated_seed_dependencies() {
    let (mut seed, modules) = selected_sources(
        "ldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()",
        "",
    );
    seed.references = Some(vec![neoclr::metadata::ModuleReference::Name("Core".into())]);
    let mut service = modules[0].functions[0].clone();
    service.name = "SeedDisplay".into();
    service.definition = None;
    seed.functions.push(service);
    let core = ROOT.replace(".module System", ".module Core");
    let app = ".module App\n.references (System)\n.entry Main\n.function Main() -> String\ncall SeedDisplay()\nret\n.end";
    let loaded = neoclr::assembler::read_modules_with_object_root(
        &[ModuleInput::Source(app), ModuleInput::Source(&core)],
        &seed,
        &selected_root(),
    )
    .unwrap();
    let program = LoadedProgram::with_modules_and_object_root(
        &loaded[0],
        &seed,
        &loaded[1..],
        &selected_root(),
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("42".into())
    );
    let mut missing_reference = seed.clone();
    missing_reference.references = Some(vec![]);
    assert!(
        LoadedProgram::with_modules_and_object_root(
            &loaded[0],
            &missing_reference,
            &loaded[1..],
            &selected_root()
        )
        .is_err()
    );
    let mut malformed = seed.clone();
    malformed.format = 4;
    assert!(
        LoadedProgram::with_modules_and_object_root(
            &loaded[0],
            &malformed,
            &loaded[1..],
            &selected_root()
        )
        .is_err()
    );
    // Selection never makes an inaccessible declaration callable from System.
    let core =
        format!("{core}\n.function internal Hidden() -> String\nldstr \"hidden\"\nret\n.end");
    seed.functions[0].body = vec![
        neoclr::metadata::Instruction::Call(
            neoclr::assembler::parse_function_ref("Hidden()").unwrap(),
        ),
        neoclr::metadata::Instruction::Return,
    ];
    assert!(
        neoclr::assembler::read_modules_with_object_root(
            &[ModuleInput::Source(app), ModuleInput::Source(&core)],
            &seed,
            &selected_root()
        )
        .is_err()
    );
}

#[test]
fn explicit_selection_honors_revision_and_cannot_be_injected_in_json() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let core = ROOT.replace(".module System", ".module Core\n.revision r1");
    let app = ".module App\n.references (Core)\n.entry Main\n.function Main() -> String\nldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()\nret\n.end";
    let root = neoclr::metadata::TypeDefId {
        revision: Some("r1".into()),
        ..selected_root()
    };
    let loaded = neoclr::assembler::read_modules_with_object_root(
        &[ModuleInput::Source(app), ModuleInput::Source(&core)],
        &seed,
        &root,
    )
    .unwrap();
    let program =
        LoadedProgram::with_modules_and_object_root(&loaded[0], &seed, &loaded[1..], &root)
            .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("42".into())
    );
    assert!(
        LoadedProgram::with_modules_and_object_root(
            &loaded[0],
            &seed,
            &loaded[1..],
            &selected_root()
        )
        .is_err()
    );
    let mut json = serde_json::to_value(&loaded[0]).unwrap();
    json["object_root"] = serde_json::to_value(&root).unwrap();
    // A parser may reject a contextual field or ignore it, but cannot apply it.
    if let Ok(injected) = serde_json::from_value::<Module>(json) {
        let legacy = LoadedProgram::with_modules(&injected, &seed, &loaded[1..]).unwrap();
        assert!(legacy.run(Limits::default()).is_err());
    }
}

#[test]
fn metadata_api_authored_root_slots_load_and_execute() {
    // Generated by ObjectSlotChecks.Create through the independent C# metadata API.
    let image = include_bytes!("fixtures/metadata-container/object-root-slots.pe");
    let root = neoclr::metadata_container::decode(image).unwrap();
    let selected = neoclr::metadata::TypeDefId {
        module: root.name.clone(),
        revision: root.revision.clone(),
        index: 0,
    };
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let boxed = root
        .functions
        .iter()
        .find(|f| f.origin.as_ref().is_some_and(|o| o.name == "BoxedDisplay"))
        .unwrap();
    for (body, result, expected) in [
        (
            "newobj Named\ncallvirt instance [{module}]System.Object::ToString()",
            "String",
            Value::String("authored root".into()),
        ),
        (
            "ldc.i4 42\nbox Int32\ncallvirt instance [{module}]System.Object::ToString()",
            "String",
            Value::String("42".into()),
        ),
        (
            "newobj Named\ncallvirt instance [{module}]System.Object::GetHashCode()",
            "Int32",
            Value::Int32(73),
        ),
        (
            "newobj Named\nnewobj Named\ncallvirt instance [{module}]System.Object::Equals(System.Object)",
            "Boolean",
            Value::Boolean(false),
        ),
        ("call {boxed}()", "String", Value::String("42".into())),
    ] {
        let body = body
            .replace("{module}", &root.name)
            .replace("{boxed}", &boxed.name);
        let source = format!(
            ".module App\n.references ({})\n.entry Main\n.type class Named\n.end\n.function Main() -> {result}\n{body}\nret\n.end",
            root.name
        );
        let modules = neoclr::assembler::read_modules_with_object_root(
            &[ModuleInput::Source(&source), ModuleInput::MetadataPe(image)],
            &seed,
            &selected,
        )
        .unwrap();
        let program = LoadedProgram::with_modules_and_object_root(
            &modules[0],
            &seed,
            &modules[1..],
            &selected,
        )
        .unwrap();
        program.verify().unwrap();
        assert_eq!(program.run(Limits::default()).unwrap().value, expected);
    }
}

#[test]
fn metadata_api_owned_root_overrides_execute() {
    let image = include_bytes!("fixtures/metadata-container/owned-object-overrides.pe");
    let library = neoclr::metadata_container::decode(image).unwrap();
    let selected = neoclr::metadata::TypeDefId {
        module: library.name.clone(),
        revision: library.revision.clone(),
        index: 0,
    };
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    for (name, result, expected) in [
        (
            "OverrideToString",
            "String",
            Value::String("derived override".into()),
        ),
        ("OverrideGetHashCode", "Int32", Value::Int32(93)),
        ("OverrideEquals", "Boolean", Value::Boolean(true)),
    ] {
        let method = library
            .functions
            .iter()
            .find(|f| f.origin.as_ref().is_some_and(|o| o.name == name))
            .unwrap();
        let source = format!(
            ".module App\n.references ({})\n.entry Main\n.function Main() -> {result}\ncall {}()\nret\n.end",
            library.name, method.name
        );
        let modules = neoclr::assembler::read_modules_with_object_root(
            &[ModuleInput::Source(&source), ModuleInput::MetadataPe(image)],
            &seed,
            &selected,
        )
        .unwrap();
        let program = LoadedProgram::with_modules_and_object_root(
            &modules[0],
            &seed,
            &modules[1..],
            &selected,
        )
        .unwrap();
        program.verify().unwrap();
        assert_eq!(program.run(Limits::default()).unwrap().value, expected);
    }
}
