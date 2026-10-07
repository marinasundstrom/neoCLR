use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "neoclr-aot-inspection-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn inspect(bytes: &[u8]) -> serde_json::Value {
    inspect_mode(bytes, false)
}
fn inspect_mode(bytes: &[u8], closed: bool) -> serde_json::Value {
    let dir = Temp::new();
    let input = dir.0.join("input.pe");
    fs::write(&input, bytes).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    command.args(["--inspect", input.to_str().unwrap(), "@entry"]);
    if closed {
        command.arg("--closed-world");
    }
    let result = command.current_dir(&dir.0).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read(&input).unwrap(), bytes);
    assert_eq!(
        fs::read_dir(&dir.0).unwrap().count(),
        1,
        "inspection must not emit artifacts"
    );
    serde_json::from_slice(&result.stdout).unwrap()
}
#[test]
fn admitted_values_report_exact_overload_identity_without_artifacts() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/Overloads.pe");
    let report = inspect(bytes);
    assert_eq!(report["admission"]["accepted"], true);
    assert_eq!(report, inspect(bytes), "report must be deterministic");
    let original = neoclr::metadata_container::decode(bytes).unwrap();
    assert_eq!(
        report["functions"].as_array().unwrap().len(),
        original.functions.len()
    );
    for (function, original) in report["functions"]
        .as_array()
        .unwrap()
        .iter()
        .zip(original.functions)
    {
        assert_eq!(
            function["definition"],
            serde_json::to_value(original.definition).unwrap()
        );
        assert_eq!(
            function["parameters"],
            serde_json::to_value(original.parameters).unwrap()
        );
    }
}
#[test]
fn generated_union_inventory_retains_unsupported_types_and_calls() {
    let report = inspect(include_bytes!(
        "../../../docs/experiments/aot-values/Choice.pe"
    ));
    assert_eq!(report["admission"]["accepted"], false);
    assert!(
        report["admission"]["firstError"]
            .as_str()
            .unwrap()
            .contains("record")
    );
    assert_eq!(report["types"].as_array().unwrap().len(), 6);
    assert_eq!(report["functions"].as_array().unwrap().len(), 20);
    for op in ["box", "callvirt", "isinst", "castclass", "ldstr"] {
        assert!(report["opcodes"][op].as_u64().unwrap() > 0);
    }
    let calls: Vec<_> = report["functions"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|f| f["calls"].as_array().unwrap())
        .collect();
    assert!(
        calls
            .iter()
            .any(|c| c["target"]["name"] == "System.String.Concat")
    );
    assert!(
        calls.iter().any(
            |c| c["operation"] == "callvirt" && c["target"]["name"] == "System.Object.ToString"
        )
    );
}
#[test]
fn inspection_rejects_corrupt_metadata_instead_of_reporting_admission() {
    let dir = Temp::new();
    let input = dir.0.join("bad.pe");
    fs::write(&input, b"MZbroken").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .args(["--inspect", input.to_str().unwrap(), "@entry"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
}

#[test]
fn closed_world_inspection_uses_emission_preparation_and_keeps_full_inventory() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/ResultApp.pe");
    assert_eq!(inspect(bytes)["admission"]["accepted"], false);
    let report = inspect_mode(bytes, true);
    assert_eq!(report["admission"]["accepted"], true);
    assert_eq!(report["admissionMode"], "closed-world");
    assert_eq!(report, inspect_mode(bytes, true));
    assert_eq!(
        report["selection"]["specialization"]["types"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let original = neoclr::metadata_container::decode(bytes).unwrap();
    assert_eq!(
        report["functions"].as_array().unwrap().len(),
        original.functions.len()
    );
    assert!(report["selection"]["functions"].as_array().unwrap().len() < original.functions.len());

    let dir = Temp::new();
    let input = dir.0.join("input.pe");
    fs::write(&input, bytes).unwrap();
    let emitted = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg("--closed-world")
        .arg(input)
        .arg("@entry")
        .arg(dir.0.join("result.o"))
        .output()
        .unwrap();
    assert!(emitted.status.success());
    assert_eq!(
        report["selection"],
        serde_json::from_slice::<serde_json::Value>(&emitted.stdout).unwrap()
    );
}

#[test]
fn closed_world_inspection_distinguishes_selection_and_body_failures() {
    let bytes = include_bytes!("../../../docs/experiments/aot-values/LibraryResultApp.pe");
    let report = inspect_mode(bytes, true);
    assert_eq!(report["admission"]["accepted"], false);
    assert_eq!(report["admission"]["phase"], "selection");
    assert!(
        report["admission"]["firstError"]
            .as_str()
            .unwrap()
            .contains("FunctionRef")
    );
    assert!(report["selection"].is_null());
    assert_eq!(report["functions"].as_array().unwrap().len(), 3);

    let mut module = neoclr::metadata_container::decode(include_bytes!(
        "../../../docs/experiments/aot-values/ResultApp.pe"
    ))
    .unwrap();
    module
        .functions
        .iter_mut()
        .find(|f| f.name == module.entry)
        .unwrap()
        .body
        .push(neoclr::metadata::Instruction::String(
            "unsupported dead instruction".into(),
        ));
    let report = inspect_mode(
        &neoclr::metadata_container::write_module(&module).unwrap(),
        true,
    );
    assert_eq!(report["admission"]["accepted"], false);
    assert_eq!(report["admission"]["phase"], "compilation");
    assert!(report["selection"].is_object());
}
