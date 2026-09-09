use std::{fs, path::PathBuf, process::Command};

static NEXT_TEMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "neo-emit-{}-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        Self(dir)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn stdout_contains_only_reassemblable_il_with_original_source_mapping() {
    let path = "examples/source/enums.neo";
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["emit-il", path])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let il = String::from_utf8(output.stdout).unwrap();
    assert!(il.starts_with(".module SourceProgram\n"));
    assert!(il.contains(path));
    assert!(il.contains(".enum Int32 flags"));
    assert!(!il.contains("=> Int32"));
    let module = neoclr::assemble(&il).unwrap();
    let emitted = neoclr::LoadedProgram::new(&module)
        .unwrap()
        .run(neoclr::Limits::default())
        .unwrap();
    let direct = neoclr::frontend::compile_named(&fs::read_to_string(path).unwrap(), path).unwrap();
    let direct = neoclr::LoadedProgram::new(&direct)
        .unwrap()
        .run(neoclr::Limits::default())
        .unwrap();
    assert_eq!(emitted.value, direct.value);
    assert_eq!(emitted.output, direct.output);
}

#[test]
fn file_output_is_exact_and_existing_files_are_preserved() {
    let temp = Temp::new();
    let output_path = temp.0.join("output.neoil");
    let path = "examples/source/generic-receivers.neo";
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["emit-il", path])
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let il = fs::read_to_string(&output_path).unwrap();
    assert_eq!(
        il,
        neoclr::frontend::lower_to_il_named(&fs::read_to_string(path).unwrap(), path).unwrap()
    );
    assert!(il.contains("ldreceiver"));
    let repeated = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["emit-il", path])
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(!repeated.status.success());
    assert_eq!(fs::read_to_string(&output_path).unwrap(), il);
}

#[test]
fn invalid_inputs_do_not_create_output_and_emission_does_not_execute() {
    let temp = Temp::new();
    let source = temp.0.join("source.neo");
    let target = temp.0.join("result.neoil");
    fs::write(&source, "func Main() -> int { return missing }").unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("emit-il")
        .arg(&source)
        .arg(&target)
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert!(!target.exists());
    fs::write(&source, "func Main() -> int { return 1 / 0 }").unwrap();
    let emitted = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("emit-il")
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    for args in [
        vec!["emit-il"],
        vec!["emit-il", "examples/hello.neoil"],
        vec!["emit-il", "x.neo", "--gc-stats"],
        vec!["emit-il", "x.neo", "out", "extra"],
    ] {
        assert!(
            !Command::new(env!("CARGO_BIN_EXE_neoclr"))
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}
