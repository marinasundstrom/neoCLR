use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-cli-modules-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().into()
    }
    fn write(&self, name: &str, text: &str) -> String {
        let path = self.path(name);
        fs::write(&path, text).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(args)
        .output()
        .unwrap()
}
fn success(args: &[&str]) -> String {
    exits(args, 0)
}
fn exits(args: &[&str], status: i32) -> String {
    let output = cli(args);
    assert!(
        output.status.code() == Some(status),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .replace("\r\n", "\n")
}

#[test]
fn cli_runs_checks_and_verifies_a_source_set() {
    let args = [
        "examples/modules/app.neoil",
        "--module",
        "examples/modules/operations.neoil",
        "--module",
        "examples/modules/models.neoil",
    ];
    for command in ["run", "check", "verify"] {
        let mut invocation = vec![command];
        invocation.extend(args);
        let output = success(&invocation);
        match command {
            "run" => assert_eq!(output, "42\n"),
            "check" => assert!(output.contains("metadata valid")),
            _ => assert!(output.contains("typed-stack/control-flow verification passed")),
        }
    }
    assert!(!cli(&["run", args[0]]).status.success());
}

#[test]
fn cli_assembles_and_runs_mixed_sources_and_legacy_artifacts() {
    let fixture = Fixture::new();
    let models = fixture.path("models.neo.json");
    success(&["assemble", "examples/modules/models.neoil", &models]);
    // Simulate legacy rows to exercise field-alias resolution against imported metadata.
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&models).unwrap()).unwrap();
    for table in ["types", "functions"] {
        for row in json[table].as_array_mut().unwrap() {
            row.as_object_mut().unwrap().remove("definition");
        }
    }
    fs::write(&models, json.to_string()).unwrap();
    let before = fs::read(&models).unwrap();
    let operations = fixture.path("operations.neo.json");
    success(&[
        "assemble",
        "examples/modules/operations.neoil",
        &operations,
        "--module",
        &models,
    ]);
    let app = fixture.path("app.neo.json");
    success(&[
        "assemble",
        "examples/modules/app.neoil",
        &app,
        "--module",
        &operations,
        "--module",
        &models,
    ]);
    assert_eq!(
        success(&[
            "run",
            &app,
            "--module",
            &operations,
            "--module",
            "examples/modules/models.neoil"
        ]),
        "42\n"
    );
    assert_eq!(fs::read(&models).unwrap(), before);
    assert!(
        !cli(&[
            "assemble",
            "examples/modules/app.neoil",
            &app,
            "--module",
            &operations,
            "--module",
            &models
        ])
        .status
        .success()
    );
}

#[test]
fn cli_uses_selected_system_for_initial_resolution() {
    let fixture = Fixture::new();
    let system = fixture.write("system.neoil", ".module System\n.revision custom-1\n.references ()\n.function CustomAnswer() -> Int32\nldc.i4 42\nret\n.end");
    let app = fixture.write("app.neoil", ".module App\n.references (System#custom-1)\n.entry Main\n.function Main() -> Int32\ncall CustomAnswer()\nret\n.end");
    assert!(!cli(&["run", &app]).status.success());
    for command in ["run", "check", "verify"] {
        exits(
            &[command, &app, "--system", &system],
            if command == "run" { 42 } else { 0 },
        );
    }
    let compiled_system = fixture.path("System.neo.json");
    success(&["assemble", &system, &compiled_system]);
    assert_eq!(exits(&["run", &app, &compiled_system], 42), "");
    let compiled_app = fixture.path("app.neo.json");
    success(&[
        "assemble",
        &app,
        &compiled_app,
        "--system",
        &compiled_system,
    ]);
    assert_eq!(
        exits(&["run", &compiled_app, "--system", &compiled_system], 42),
        ""
    );
}

#[test]
fn cli_rejects_bad_flags_and_revision_mismatches_before_creating_output() {
    let fixture = Fixture::new();
    for options in [
        vec!["--module"],
        vec!["--system"],
        vec!["--unknown", "x"],
        vec!["--system", "x", "--system", "y"],
        vec!["--module", "--system", "x"],
    ] {
        let mut args = vec!["check", "examples/hello.neoil"];
        args.extend(options);
        let output = cli(&args);
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
    }
    assert_eq!(
        success(&[
            "run",
            "examples/revisions/app.neoil",
            "--module",
            "examples/revisions/answers.neoil"
        ]),
        "42\n"
    );
    let wrong = fixture.write(
        "answers.neoil",
        &fs::read_to_string("examples/revisions/answers.neoil")
            .unwrap()
            .replace("sample-1", "sample-2"),
    );
    let artifact = fixture.path("invalid.neo.json");
    let output = cli(&[
        "assemble",
        "examples/revisions/app.neoil",
        &artifact,
        "--module",
        &wrong,
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("revision mismatch"));
    assert!(!std::path::Path::new(&artifact).exists());
}

#[test]
fn assembler_emits_verified_native_assemblies_and_preserves_dependencies() {
    let fixture = Fixture::new();
    let models = fixture.path("models.neox");
    let operations = fixture.path("operations.neox");
    let app = fixture.path("app.neox");
    success(&[
        "assemble",
        "examples/modules/models.neoil",
        &models,
        "--format",
        "neox",
    ]);
    success(&[
        "assemble",
        "examples/modules/operations.neoil",
        &operations,
        "--format",
        "neox",
        "--module",
        &models,
    ]);
    success(&[
        "assemble",
        "examples/modules/app.neoil",
        &app,
        "--format",
        "neox",
        "--module",
        &operations,
        "--module",
        &models,
    ]);
    let bytes = fs::read(&app).unwrap();
    assert!(bytes.starts_with(b"NEOX"));
    assert_eq!(u16::from_le_bytes(bytes[18..20].try_into().unwrap()), 3);
    let module = neoclr::metadata_container::decode_envelope(&bytes).unwrap();
    assert_eq!(module.name, "Application");
    assert!(
        module.types.is_empty(),
        "dependencies must not be flattened into the output"
    );
    assert_eq!(
        success(&["run", &app, "--module", &operations, "--module", &models]),
        "42\n"
    );
    assert!(!cli(&["run", &app]).status.success());
    assert!(
        !cli(&[
            "assemble",
            "examples/modules/app.neoil",
            &app,
            "--format",
            "neox",
            "--module",
            &operations,
            "--module",
            &models
        ])
        .status
        .success()
    );
    assert_eq!(fs::read(&app).unwrap(), bytes);
}

#[test]
fn native_assembler_rejects_bad_bodies_and_options_before_output_creation() {
    let fixture = Fixture::new();
    let source = fixture.write("invalid.neoil", ".module Invalid\n.entry Main\n.function Main() -> Int32\nldstr \"wrong type\"\nret\n.end\n");
    let output = fixture.path("invalid.neox");
    assert!(
        !cli(&["assemble", &source, &output, "--format", "neox"])
            .status
            .success()
    );
    assert!(!std::path::Path::new(&output).exists());
    for options in [
        vec!["--format"],
        vec!["--format", "unknown"],
        vec!["--format", "neox", "--format", "neox"],
    ] {
        let mut args = vec!["assemble", "examples/hello.neoil", &output];
        args.extend(options);
        assert!(!cli(&args).status.success());
        assert!(!std::path::Path::new(&output).exists());
    }
    let json = fixture.path("explicit.json");
    success(&[
        "assemble",
        "examples/hello.neoil",
        &json,
        "--format",
        "json",
    ]);
    let legacy: serde_json::Value = serde_json::from_slice(&fs::read(json).unwrap()).unwrap();
    assert_eq!(legacy["name"], "HelloWorld");
}
