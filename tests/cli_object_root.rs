use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct Context {
    directory: PathBuf,
    app: PathBuf,
    seed: PathBuf,
    root: PathBuf,
}
impl Context {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "neo-root-cli-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let root = directory.join("Root.pe");
        let app = directory.join("App.neoil");
        let seed = directory.join("System.neoil");
        let image = include_bytes!("fixtures/metadata-container/raven-source-object-root.pe");
        fs::write(&root, image).unwrap();
        let library = neoclr::metadata_container::decode(image).unwrap();
        let display = library
            .functions
            .iter()
            .find(|f| f.origin.as_ref().is_some_and(|o| o.name == "Display"))
            .unwrap();
        fs::write(&app, format!(".module App\n.references ({})\n.entry Main\n.function Main() -> String\ncall {}()\nret\n.end", library.name, display.name)).unwrap();
        fs::write(&seed, ".module System\n.references ()\n").unwrap();
        Self {
            directory,
            app,
            seed,
            root,
        }
    }
    fn run(&self, command: &str, select: bool) -> Output {
        let mut process = Command::new(env!("CARGO_BIN_EXE_neoclr"));
        process
            .arg(command)
            .arg(&self.app)
            .arg("--module")
            .arg(&self.root)
            .arg("--system")
            .arg(&self.seed);
        if select {
            process.arg("--object-root").arg(&self.root);
        }
        if command == "run" {
            process.arg("--show-result");
        }
        process.output().unwrap()
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn explicit_root_verifies_and_executes_raven_override() {
    let context = Context::new();
    let checked = context.run("verify", true);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let executed = context.run("run", true);
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert!(executed.stdout.is_empty());
    assert_eq!(
        String::from_utf8(executed.stderr).unwrap(),
        "=> String(\"source root override\")\n"
    );
}

#[test]
fn root_selection_rejects_missing_seed_unregistered_or_application_paths() {
    let context = Context::new();
    for root in [&context.app, &context.seed] {
        let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .arg("verify")
            .arg(&context.app)
            .arg("--module")
            .arg(&context.root)
            .arg("--system")
            .arg(&context.seed)
            .arg("--object-root")
            .arg(root)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("exactly one explicit --module"));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("verify")
        .arg(&context.app)
        .arg("--module")
        .arg(&context.root)
        .arg("--object-root")
        .arg(&context.root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("explicit --system"));
}

#[test]
fn incompatible_root_fails_before_assembly_publication() {
    let context = Context::new();
    let mut module = neoclr::metadata_container::decode(&fs::read(&context.root).unwrap()).unwrap();
    module
        .types
        .iter_mut()
        .find(|t| t.name == "System.Object")
        .unwrap()
        .is_abstract = false;
    fs::write(&context.root, serde_json::to_vec(&module).unwrap()).unwrap();
    let artifact = context.directory.join("must-not-exist.neox");
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("assemble")
        .arg(&context.app)
        .arg(&artifact)
        .args(["--format", "neox", "--module"])
        .arg(&context.root)
        .arg("--system")
        .arg(&context.seed)
        .arg("--object-root")
        .arg(&context.root)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!artifact.exists());
}
