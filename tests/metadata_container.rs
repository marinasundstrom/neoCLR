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
