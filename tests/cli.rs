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
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        "Hello, world!\n=> Void\n"
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
            .contains("at Main:0")
    );
}
