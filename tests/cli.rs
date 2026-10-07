use std::process::Command;

#[test]
fn cli_assembles_checks_and_executes_hello_world() {
    let binary = env!("CARGO_BIN_EXE_neoclr");
    let path = std::env::temp_dir().join(format!(
        "neoclr-{}-{}.neo.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let status = Command::new(binary)
        .args(["assemble", "examples/hello.neoil"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(status.status.success(), "{:?}", status);
    let check = Command::new(binary)
        .arg("check")
        .arg(&path)
        .output()
        .unwrap();
    assert!(check.status.success());
    let output = Command::new(binary).arg("run").arg(&path).output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        "Hello, world!\n"
    );
    let repeated = Command::new(binary)
        .args(["assemble", "examples/hello.neoil"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(!repeated.status.success());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn cli_fault_has_failure_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["run", "examples/fault.neoil"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("at Main [instruction 0]")
    );
}

#[test]
fn cli_compiles_and_loads_platform_runtime_library() {
    let binary = env!("CARGO_BIN_EXE_neoclr");
    let path = std::env::temp_dir().join(format!(
        "neoclr-system-{}-{}.neo.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let compiled = Command::new(binary)
        .args(["assemble", "runtime/System.neoil"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let output = Command::new(binary)
        .args(["run", "examples/hello.neoil"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        "Hello, world!\n"
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn return_value_is_an_opt_in_stderr_diagnostic() {
    let binary = env!("CARGO_BIN_EXE_neoclr");
    let output = Command::new(binary)
        .args(["run", "examples/hello.neoil", "--show-result"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Hello, world!\n");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "=> Void\n");
    for args in [
        vec!["check", "examples/hello.neoil", "--show-result"],
        vec![
            "run",
            "examples/hello.neoil",
            "--show-result",
            "--show-result",
        ],
    ] {
        assert!(
            !Command::new(binary)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}

#[test]
fn cli_instruction_budget_is_explicit_and_preserves_default() {
    let path = std::env::temp_dir().join(format!("neoclr-budget-{}.neoil", std::process::id()));
    std::fs::write(
        &path,
        r#"
.module Budget
.entry Main
.function Main() -> Int32
.local Int32
ldc.i4 0
stloc 0
Loop:
ldloc 0
ldc.i4 1
add
stloc 0
ldloc 0
ldc.i4 30000
clt
brtrue Loop
ldc.i4 42
ret
.end
"#,
    )
    .unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .arg("run")
            .arg(&path)
            .args(extra)
            .output()
            .unwrap()
    };
    let default = run(&[]);
    assert_eq!(default.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&default.stderr).contains("InstructionLimitExceeded"));
    let small = run(&["--instructions", "1"]);
    assert_eq!(small.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&small.stderr).contains("InstructionLimitExceeded"));
    let large = run(&["--instructions", "1000000"]);
    assert_eq!(large.status.code(), Some(42), "{large:?}");
    assert!(large.stdout.is_empty() && large.stderr.is_empty());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn cli_rejects_invalid_instruction_budget_before_loading() {
    for args in [
        vec!["run", "missing.neoil", "--instructions"],
        vec!["run", "missing.neoil", "--instructions", "0"],
        vec!["run", "missing.neoil", "--instructions", "-1"],
        vec!["run", "missing.neoil", "--instructions", "+1"],
        vec!["run", "missing.neoil", "--instructions", "abc"],
        vec![
            "run",
            "missing.neoil",
            "--instructions",
            "999999999999999999999999999999",
        ],
        vec![
            "run",
            "missing.neoil",
            "--instructions",
            "1",
            "--instructions",
            "2",
        ],
        vec!["verify", "missing.neoil", "--instructions", "1"],
        vec!["debug", "missing.neoil", "--instructions", "1"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("--instructions") && !error.contains("Cannot read"),
            "{error}"
        );
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn cli_execution_faults_share_host_presentation_and_exit_one() {
    let path = std::env::temp_dir().join(format!("neoclr-common-fault-{}.neoil", std::process::id()));
    for body in ["fault \"Unicode 🌍\\u0000message\"", "ldc.i4 1\nldc.i4 0\ndiv", "ldc.i4 2147483647\nldc.i4 1\nadd.ovf", "ldc.i4 1\nvalue.pack Int32\nvalue.unpack Byte"] {
        let source=format!(".module CliFault\n.entry Main\n.function Leaf() -> Int32\n{body}\nret\n.end\n.function Main() -> Int32\ncall Leaf()\nret\n.end");
        std::fs::write(&path,&source).unwrap();
        let module=neoclr::assemble(&source).unwrap();
        let fault=neoclr::run(&module,neoclr::Limits::default()).unwrap_err();
        let output=Command::new(env!("CARGO_BIN_EXE_neoclr")).arg("run").arg(&path).output().unwrap();
        assert_eq!(output.status.code(),Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(String::from_utf8(output.stderr).unwrap().replace("\r\n","\n"),fault.diagnostic().to_string());
    }
    for code in [0, 1, 42] {
        std::fs::write(&path,format!(".module Exit\n.entry Main\n.function Main() -> Int32\nldc.i4 {code}\nret\n.end")).unwrap();
        let output=Command::new(env!("CARGO_BIN_EXE_neoclr")).arg("run").arg(&path).output().unwrap();
        assert_eq!(output.status.code(),Some(code));
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
    std::fs::remove_file(path).unwrap();
}
