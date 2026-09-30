use neoclr::{ExecutionOptions, LoadedProgram, Value, assembler, metadata_container};

// Emitted by the independent C# metadata API, including a top-level function.
const IMAGE: &[u8] = include_bytes!("fixtures/metadata-container/constant42.pe");

#[test]
fn api_produced_pe_loads_native_declarations_and_executes() {
    let module = metadata_container::load(IMAGE).unwrap();
    assert_eq!(module.assemblies[0].name, "ContainerRuntime");
    assert_eq!(module.functions.len(), 1);
    assert!(module.functions[0].owner.is_none());
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert!(matches!(
        program.run(ExecutionOptions::default()).unwrap().value,
        Value::Int32(42)
    ));
    let modules = assembler::read_modules(
        &[assembler::ModuleInput::MetadataPe(IMAGE)],
        neoclr::library::system().unwrap(),
    )
    .unwrap();
    assert_eq!(modules[0].name, module.name);
}

#[test]
fn truncated_and_changed_metadata_fail_before_admission() {
    for size in 0..IMAGE.len() {
        assert!(
            metadata_container::native_json(&IMAGE[..size]).is_err(),
            "length {size}"
        );
    }
    let mut changed = IMAGE.to_vec();
    let marker = changed.windows(7).rposition(|w| w == b"sha256=").unwrap();
    changed[marker + 7] = if changed[marker + 7] == b'0' {
        b'1'
    } else {
        b'0'
    };
    assert!(
        metadata_container::load(&changed)
            .unwrap_err()
            .to_string()
            .contains("binding mismatch")
    );
    let mut overlay = IMAGE.to_vec();
    overlay.push(0);
    assert!(metadata_container::native_json(&overlay).is_err());
}

#[test]
fn binary_and_json_containers_decode_to_identical_runtime_metadata() {
    let binary = include_bytes!("fixtures/metadata-container/constant42-binary.pe");
    assert!(
        metadata_container::native_json(binary)
            .unwrap_err()
            .to_string()
            .contains("binary payload")
    );
    let json = metadata_container::load(IMAGE).unwrap();
    let decoded = metadata_container::load(binary).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded).unwrap(),
        serde_json::to_value(&json).unwrap()
    );
    let program = LoadedProgram::new(&decoded).unwrap();
    program.verify().unwrap();
    assert!(matches!(
        program.run(ExecutionOptions::default()).unwrap().value,
        Value::Int32(42)
    ));
    let modules = assembler::read_modules(
        &[assembler::ModuleInput::MetadataPe(binary)],
        neoclr::library::system().unwrap(),
    )
    .unwrap();
    assert_eq!(modules[0].entry, decoded.entry);
    for length in 0..binary.len() {
        assert!(metadata_container::decode(&binary[..length]).is_err());
    }
}

#[test]
fn translated_native_envelope_preserves_generic_library_metadata() {
    let image = include_bytes!("fixtures/metadata-container/models.neox");
    let expected =
        serde_json::from_str::<neoclr::Module>(include_str!("fixtures/metadata-container/models.neo.json")).unwrap();
    let actual = metadata_container::decode_envelope(image).unwrap();
    assert_eq!(
        serde_json::to_value(expected).unwrap(),
        serde_json::to_value(actual).unwrap()
    );
    let system = neoclr::library::system().unwrap();
    let modules = assembler::read_modules(
        &[
            assembler::ModuleInput::Source(include_str!("../examples/modules/app.neoil")),
            assembler::ModuleInput::Source(include_str!("../examples/modules/operations.neoil")),
            assembler::ModuleInput::NativeEnvelope(image),
        ],
        system,
    )
    .unwrap();
    let program = LoadedProgram::with_modules(&modules[0], system, &modules[1..]).unwrap();
    program.verify().unwrap();
    program.run(ExecutionOptions::default()).unwrap();
    for length in 0..image.len() {
        assert!(metadata_container::decode_envelope(&image[..length]).is_err());
    }
    let mut overlay = image.to_vec();
    overlay.push(0);
    assert!(metadata_container::decode_envelope(&overlay).is_err());
    let mut schema = image.to_vec();
    schema[18] = 3;
    assert!(metadata_container::decode_envelope(&schema).is_err());
    let mut optional = image.to_vec();
    optional[20] = 0;
    assert!(metadata_container::decode_envelope(&optional).is_err());
    assert!(metadata_container::decode_envelope(&vec![0; 1024 * 1024 + 1]).is_err());
}
