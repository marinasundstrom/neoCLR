use neoclr::{assemble, source::read_source};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-source-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("parts/nested")).unwrap();
        Self(path)
    }
    fn write(&self, path: &str, text: &str) {
        fs::write(self.0.join(path), text).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn nested_includes_resolve_beside_the_parent_and_preserve_declaration_order() {
    let fixture = Fixture::new();
    fixture.write("app.neoil", ".module App\n.entry Main\n.include \"parts/first.neoil\" ; a comment\n.include \"parts/second file.neoil\"");
    fixture.write("parts/first.neoil", ".include \"nested/helper.neoil\"");
    fixture.write(
        "parts/nested/helper.neoil",
        ".function Answer() -> Int32\nldc.i4 42\nret\n.end",
    );
    fixture.write(
        "parts/second file.neoil",
        ".function Main() -> Int32\ncall Answer()\nret\n.end",
    );
    let text = read_source(fixture.0.join("app.neoil")).unwrap();
    let module = assemble(&text).unwrap();
    assert_eq!(
        module
            .functions
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["Answer", "Main"]
    );
    assert_eq!(
        neoclr::run(&module, neoclr::Limits::default())
            .unwrap()
            .value,
        neoclr::Value::Int32(42)
    );
    assert!(assemble(".module App\n.include \"missing.neoil\"").is_err());
}

#[test]
fn malformed_missing_and_cyclic_includes_fail_with_source_context() {
    let fixture = Fixture::new();
    for line in [
        ".include unquoted",
        ".include \"\"",
        ".include \"a\" trailing",
    ] {
        fixture.write("app.neoil", &format!(".module App\n{line}\n"));
        let error = read_source(fixture.0.join("app.neoil"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("app.neoil:2"), "{error}");
    }
    fixture.write("app.neoil", ".include \"missing.neoil\"");
    assert!(
        read_source(fixture.0.join("app.neoil"))
            .unwrap_err()
            .to_string()
            .contains("missing.neoil")
    );
    fixture.write("app.neoil", ".include \"parts/cycle.neoil\"");
    fixture.write("parts/cycle.neoil", ".include \"../app.neoil\"");
    assert!(
        read_source(fixture.0.join("app.neoil"))
            .unwrap_err()
            .to_string()
            .contains("cycle")
    );
}

#[test]
fn system_manifest_matches_bundled_source_and_compiled_metadata() {
    let text = read_source("runtime/System.neoil").unwrap();
    assert_eq!(text, neoclr::library::system_source());
    let explicit = assemble(&text).unwrap();
    assert_eq!(
        serde_json::to_value(explicit).unwrap(),
        serde_json::to_value(neoclr::library::system().unwrap()).unwrap()
    );
}

#[test]
fn cli_assembles_manifest_to_a_standalone_artifact() {
    let fixture = Fixture::new();
    let artifact = fixture.0.join("System.neo.json");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["assemble", "runtime/System.neoil"])
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["run", "examples/hello.neoil", "--system"])
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Hello, world!"));
}
