use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

const ROOT: &str = "tests/fixtures/metadata-container/raven-source-object-root.pe";
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-disassemble-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("disassemble")
        .args(args)
        .output()
        .unwrap()
}
fn success(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    output.stdout
}

#[test]
fn emitted_pe_shows_source_identity_slots_and_instructions_without_host_selection() {
    let text = String::from_utf8(success(run(&[ROOT]))).unwrap();
    assert!(text.starts_with("// NeoCLR native metadata disassembly v1\n"));
    for fact in [
        "System.Object",
        "ToString",
        "Equals",
        "source root override",
        ".method #",
        "I_0000:",
        "callvirt",
        "newobj",
    ] {
        assert!(text.contains(fact), "missing {fact}: {text}");
    }
    assert_eq!(text.as_bytes(), success(run(&[ROOT])));
}

#[test]
fn equivalent_containers_have_identical_listings() {
    let temp = Temp::new();
    let module = neoclr::metadata_container::decode(&fs::read(ROOT).unwrap()).unwrap();
    let json = temp.0.join("module.json");
    let neox = temp.0.join("module.neox");
    fs::write(&json, serde_json::to_vec(&module).unwrap()).unwrap();
    fs::write(
        &neox,
        neoclr::metadata_container::write_module(&module).unwrap(),
    )
    .unwrap();
    let expected = success(run(&[ROOT]));
    assert_eq!(expected, success(run(&[json.to_str().unwrap()])));
    assert_eq!(expected, success(run(&[neox.to_str().unwrap()])));
}

#[test]
fn unresolved_dependencies_and_unverified_bodies_remain_inspectable() {
    let temp = Temp::new();
    let mut module = neoclr::metadata_container::decode(&fs::read(ROOT).unwrap()).unwrap();
    module.entry = "missing-entry-for-debugging".into();
    let path = temp.0.join("unlinked.json");
    let mut value = serde_json::to_value(&module).unwrap();
    value["references"] =
        serde_json::json!([{ "name": "Missing.Contracts", "revision": "required-revision" }]);
    value["functions"][0]["body"] = serde_json::json!([{"op":"br","arg":99999}]);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let text = String::from_utf8(success(run(&[path.to_str().unwrap()]))).unwrap();
    assert!(text.contains("missing-entry-for-debugging"));
    assert!(text.contains("Missing.Contracts"));
    assert!(text.contains("required-revision"));
    assert!(text.contains("I_0000: br 99999"));
    assert!(text.contains("Dependencies are unresolved"));
}

#[test]
fn output_is_exact_and_never_overwrites_existing_files() {
    let temp = Temp::new();
    let path = temp.0.join("listing.txt");
    let output = path.to_str().unwrap();
    success(run(&[ROOT, output]));
    assert_eq!(fs::read(&path).unwrap(), success(run(&[ROOT])));
    let before = fs::read(&path).unwrap();
    assert!(!run(&[ROOT, output]).status.success());
    assert_eq!(before, fs::read(&path).unwrap());
    let source = temp.0.join("source.pe");
    fs::copy(ROOT, &source).unwrap();
    let before = fs::read(&source).unwrap();
    assert!(
        !run(&[source.to_str().unwrap(), source.to_str().unwrap()])
            .status
            .success()
    );
    assert_eq!(before, fs::read(&source).unwrap());
}

#[test]
fn malformed_or_unsupported_inputs_never_publish_output() {
    let temp = Temp::new();
    let input = temp.0.join("bad");
    let output = temp.0.join("absent");
    for bytes in [
        b"MZ".as_slice(),
        b"NEOX",
        b"{}",
        br#"{"format":4,"name":"old","functions":[]}"#,
        b".module source",
    ] {
        fs::write(&input, bytes).unwrap();
        let result = run(&[input.to_str().unwrap(), output.to_str().unwrap()]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
        assert!(!output.exists());
    }
    for args in [vec![], vec![ROOT, "--system", "missing"], vec!["--help"]] {
        let result = run(&args);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn generic_library_keeps_declaration_and_operand_facts() {
    let text = String::from_utf8(success(run(&[
        "tests/fixtures/metadata-container/models.neox",
    ])))
    .unwrap();
    for fact in [
        "Box",
        "generic_parameters",
        "TypeParameter",
        "Constructed",
        "Box.Create",
        "Value",
    ] {
        assert!(text.contains(fact), "missing {fact}: {text}");
    }
}
