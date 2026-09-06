use neoclr::{
    Limits, LoadedProgram, TypeIdentity, Value,
    assembler::{assemble_modules, parse_type},
    library, load_modules,
    metadata::TypeDefId,
};

const SOURCES: &[&str] = &[
    include_str!("../examples/modules/app.neoil"),
    include_str!("../examples/modules/operations.neoil"),
    include_str!("../examples/modules/models.neoil"),
];

#[test]
fn source_group_resolves_generic_calls_and_cross_module_field_aliases() {
    let mut modules = assemble_modules(SOURCES).unwrap();
    // This test also asks the root to resolve a Models type directly.
    modules[0]
        .references
        .as_mut()
        .unwrap()
        .push("Models".into());
    let original = serde_json::to_value(&modules).unwrap();
    let program =
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["42"]);
    let report = program.verify().unwrap();
    let create = report
        .functions
        .iter()
        .find(|f| f.name == "Box.Create")
        .unwrap();
    assert_eq!(create.definition.as_ref().unwrap().module, "Models");
    assert_eq!(create.definition.as_ref().unwrap().index, 0);
    assert_eq!(serde_json::to_value(&modules).unwrap(), original);
    let TypeIdentity::Definition { definition, .. } = program
        .resolve_type_identity(&parse_type("Box<Int32>").unwrap())
        .unwrap()
    else {
        panic!("expected definition")
    };
    assert_eq!(
        definition,
        TypeDefId {
            module: "Models".into(),
            index: 0
        }
    );
}

#[test]
fn dependency_order_does_not_change_definition_rows_or_execution() {
    let modules = assemble_modules(SOURCES).unwrap();
    let dependencies = vec![modules[2].clone(), modules[1].clone()];
    let program =
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &dependencies)
            .unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["42"]);
    let reordered = assemble_modules(&[SOURCES[0], SOURCES[2], SOURCES[1]]).unwrap();
    assert_eq!(
        serde_json::to_value(&modules[2]).unwrap(),
        serde_json::to_value(&reordered[1]).unwrap()
    );
}

#[test]
fn artifacts_roundtrip_as_separate_modules_and_legacy_rows_are_derived() {
    let modules = assemble_modules(SOURCES).unwrap();
    let json: Vec<_> = modules
        .iter()
        .map(|module| {
            let mut json = serde_json::to_value(module).unwrap();
            for table in ["types", "functions"] {
                for row in json[table].as_array_mut().unwrap() {
                    row.as_object_mut().unwrap().remove("definition");
                }
            }
            json.to_string()
        })
        .collect();
    let loaded = load_modules(&json.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    let program =
        LoadedProgram::with_modules(&loaded[0], library::system().unwrap(), &loaded[1..]).unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["42"]);
    assert!(LoadedProgram::new(&loaded[0]).is_err());
}

#[test]
fn supplied_module_headers_and_row_provenance_are_checked() {
    let modules = assemble_modules(SOURCES).unwrap();
    for kind in 0..5 {
        let mut dependencies = modules[1..].to_vec();
        match kind {
            0 => dependencies[0].name = "Application".into(),
            1 => dependencies[0].name = "System".into(),
            2 => dependencies[0].format = 999,
            3 => dependencies[0].entry = "Operations.Compute".into(),
            _ => dependencies[1].types[0].definition.as_mut().unwrap().index = 999,
        }
        assert!(
            LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &dependencies)
                .is_err(),
            "{kind}"
        );
    }
}

#[test]
fn duplicate_symbols_and_missing_dependencies_fail_without_order_fallback() {
    for sources in [
        vec![
            ".module App\n.type Duplicate\n.end",
            ".module Other\n.type Duplicate\n.end",
        ],
        vec![
            ".module App\n.function F() -> Void\nldvoid\nret\n.end",
            ".module Other\n.function F() -> Void\nldvoid\nret\n.end",
        ],
        vec![SOURCES[0], SOURCES[1]],
    ] {
        assert!(assemble_modules(&sources).is_err());
    }
    assert!(assemble_modules(&[]).is_err());
    assert!(load_modules(&[]).is_err());
    assert!(load_modules(&["{}"]).is_err());
}

#[test]
fn mutually_referencing_dependencies_resolve_as_a_set() {
    let modules = assemble_modules(&[
        ".module App\n.entry Main\n.function Main() -> Int32\ncall B.Get()\nret\n.end",
        ".module A\n.function A.Unused() -> Int32\ncall B.Get()\nret\n.end\n.function A.Answer() -> Int32\nldc.i4 42\nret\n.end",
        ".module B\n.function B.Get() -> Int32\ncall A.Answer()\nret\n.end",
    ]).unwrap();
    let program =
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap();
    assert!(program.verify().is_ok());
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn metadata_cannot_attach_a_method_to_another_modules_type() {
    let mut modules = assemble_modules(SOURCES).unwrap();
    let method = modules[2].functions.remove(0);
    modules[1].functions.push(method);
    let moved = modules[1].functions.last_mut().unwrap();
    moved.definition.as_mut().unwrap().module = "Operations".into();
    moved.definition.as_mut().unwrap().index = 1;
    // Keep the explicit call consistent so rejection concerns the declaration owner.
    if let neoclr::metadata::Instruction::Call(target) = &mut modules[1].functions[0].body[1] {
        target.definition.as_mut().unwrap().module = "Operations".into();
        target.definition.as_mut().unwrap().index = 1;
    }
    assert!(
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap_err()
            .message
            .contains("same module")
    );
}
