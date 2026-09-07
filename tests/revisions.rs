use neoclr::{
    Limits, LoadedProgram, Value,
    assembler::{assemble_modules, parse_type},
    library, load_modules,
    metadata::{Instruction, ModuleReference},
};

const APP: &str = ".module App\n.revision application-1\n.references (Models#build-1)\n.entry Main\n.function Main() -> Int32\ncall [Models]Point::Answer() @ Models#build-1:0\nret\n.end";
const MODELS: &str = ".module Models\n.revision build-1\n.references ()\n.type Point\n.method static Answer() -> Int32\nldc.i4 42\nret\n.end\n.end";

fn prepare(sources: &[&str]) -> Result<LoadedProgram, neoclr::Fault> {
    let modules = assemble_modules(sources)?;
    LoadedProgram::with_modules(&modules[0], library::system()?, &modules[1..])
}

#[test]
fn revision_pins_and_definition_ids_roundtrip_and_execute() {
    let modules = assemble_modules(&[APP, MODELS]).unwrap();
    assert_eq!(
        modules[0].references,
        Some(vec![ModuleReference::Exact {
            name: "Models".into(),
            revision: "build-1".into()
        }])
    );
    assert_eq!(
        modules[1].types[0]
            .definition
            .as_ref()
            .unwrap()
            .revision
            .as_deref(),
        Some("build-1")
    );
    assert_eq!(
        modules[1].functions[0]
            .definition
            .as_ref()
            .unwrap()
            .revision
            .as_deref(),
        Some("build-1")
    );
    let json: Vec<_> = modules
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();
    let loaded = load_modules(&json.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    let program =
        LoadedProgram::with_modules(&loaded[0], library::system().unwrap(), &loaded[1..]).unwrap();
    assert!(program.verify().is_ok());
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn replacement_artifact_with_matching_names_and_rows_must_match_the_pin() {
    for models in [
        MODELS.replace("build-1", "build-2"),
        MODELS.replace(".revision build-1\n", ""),
    ] {
        assert!(
            prepare(&[APP, &models])
                .unwrap_err()
                .message
                .contains("revision mismatch")
        );
    }
    // Pins are checked even if no code refers to the dependency.
    assert!(prepare(&[".module App\n.references (Models#build-2)", MODELS]).is_err());
}

#[test]
fn definition_revisions_are_exact_even_when_the_module_reference_is_unpinned() {
    for target in ["@ Models#build-2:0", "@ Models:0"] {
        let source = APP
            .replace("(Models#build-1)", "(Models)")
            .replace("@ Models#build-1:0", target);
        assert!(
            prepare(&[&source, MODELS])
                .unwrap_err()
                .message
                .contains("unknown function overload")
        );
    }
    let source = APP
        .replace("(Models#build-1)", "(Models)")
        .replace(" @ Models#build-1:0", "");
    let first = prepare(&[&source, MODELS]).unwrap();
    let second = prepare(&[&source, &MODELS.replace("build-1", "build-2")]).unwrap();
    let ty = parse_type("[Models]Point").unwrap();
    assert_ne!(
        first.resolve_type_identity(&ty).unwrap(),
        second.resolve_type_identity(&ty).unwrap()
    );
    assert_eq!(
        second.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn source_rows_cannot_claim_another_revision_and_missing_rows_are_derived() {
    let mut modules = assemble_modules(&[APP, MODELS]).unwrap();
    modules[1].types[0].definition.as_mut().unwrap().revision = Some("build-2".into());
    assert!(
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap_err()
            .message
            .contains("noncanonical type")
    );
    modules[1].types[0].definition = None;
    modules[1].functions[0].definition = None;
    let program =
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let Instruction::Call(target) = &mut modules[0].functions[0].body[0] else {
        panic!("call")
    };
    target.definition.as_mut().unwrap().revision = Some("build-2".into());
    assert!(
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .is_err()
    );
}

#[test]
fn revisions_reject_invalid_labels_duplicate_directives_and_ambiguous_sets() {
    for directive in [
        ".revision",
        ".revision build/1",
        ".revision build#1",
        ".revision one\n.revision two",
    ] {
        assert!(neoclr::assemble(&format!(".module App\n{directive}")).is_err());
    }
    for reference in ["Models#", "Models#build#1", "Models#build-1, Models"] {
        assert!(prepare(&[&format!(".module App\n.references ({reference})"), MODELS]).is_err());
    }
    assert!(
        prepare(&[APP, MODELS, &MODELS.replace("build-1", "build-2")])
            .unwrap_err()
            .message
            .contains("duplicate module")
    );
    let mut json = serde_json::to_value(neoclr::assemble(".module App").unwrap()).unwrap();
    json["revision"] = "bad/label".into();
    assert!(neoclr::load(&json.to_string()).is_err());
}

#[test]
fn system_can_be_pinned_and_unversioned_artifacts_keep_their_encoding() {
    let mut system = library::system().unwrap().clone();
    system.revision = Some("runtime-1".into());
    for ty in &mut system.types {
        ty.definition = None;
        if let Some(owner) = &mut ty.declaring_type {
            owner.revision = system.revision.clone();
        }
    }
    for function in &mut system.functions {
        function.definition = None;
    }
    let mut app = neoclr::assemble(".module App\n.references (System)").unwrap();
    app.references = Some(vec![ModuleReference::Exact {
        name: "System".into(),
        revision: "runtime-1".into(),
    }]);
    LoadedProgram::with_library(&app, &system).unwrap();
    assert!(LoadedProgram::with_library(&app, library::system().unwrap()).is_err());
    let json = serde_json::to_value(neoclr::assemble(".module App\n.references (System)").unwrap())
        .unwrap();
    assert!(json.get("revision").is_none());
    assert_eq!(json["references"], serde_json::json!(["System"]));
}
