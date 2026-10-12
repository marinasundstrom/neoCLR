//! Adapter-level default/discovery tests; executable native qualification lives
//! in the development JIT bundle gate.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn installed_default_jit_discovers_trusted_siblings_and_preserves_arguments() {
    use std::{fs, os::unix::fs::PermissionsExt, process::Command};
    let root = std::env::temp_dir().join(format!("neoclr default jit {}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::create_dir_all(root.join("untrusted")).unwrap();
    let runtime = root.join("bin/neoclr");
    fs::copy(env!("CARGO_BIN_EXE_neoclr"), &runtime).unwrap();
    fs::write(
        root.join("lib/bundle.json"),
        r#"{"assemblyNames":["System.Runtime"],"runtimeSeed":"System.runtime.neox"}"#,
    )
    .unwrap();
    for name in [
        "lib/System.Runtime.dll",
        "lib/System.runtime.neox",
        "bin/neoclr-jit-host.dylib",
    ] {
        fs::write(root.join(name), []).unwrap();
    }
    let compiler = root.join("bin/neoclr-aot-poc");
    fs::write(
        &compiler,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$NEO_TEST_ARGS\"\nexit 37\n",
    )
    .unwrap();
    fs::set_permissions(&compiler, fs::Permissions::from_mode(0o755)).unwrap();
    let recorded = root.join("arguments");
    let result = Command::new(&runtime)
        .current_dir(root.join("untrusted"))
        .env("NEO_TEST_ARGS", &recorded)
        .args(["run", "app.dll", "--", "a b", "", "é", "--flag"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(37), "{result:?}");
    let arguments = fs::read_to_string(&recorded).unwrap();
    assert!(arguments.starts_with("--jit-run\n--closed-world\napp.dll\n@entry\n"));
    assert!(arguments.contains(root.join("bin/neoclr-jit-host.dylib").to_str().unwrap()));
    assert!(arguments.contains("--bind-runtime-async\n"));
    assert!(arguments.ends_with("--jit-entry-argument\na b\n--jit-entry-argument\n\n--jit-entry-argument\né\n--jit-entry-argument\n--flag\n"));
    let custom = root.join("custom.dll");
    fs::write(&custom, []).unwrap();
    let extended = Command::new(&runtime)
        .env("NEO_TEST_ARGS", &recorded)
        .args(["run", "app.dll", "--module"])
        .arg(&custom)
        .output()
        .unwrap();
    assert_eq!(extended.status.code(), Some(37));
    let extended_arguments = fs::read_to_string(&recorded).unwrap();
    assert!(extended_arguments.contains(custom.to_str().unwrap()));
    assert!(extended_arguments.contains("System.Runtime.dll"));
    let standard = root.join("lib/System.Runtime.dll");
    let duplicate = Command::new(&runtime)
        .env("NEO_TEST_ARGS", &recorded)
        .args(["run", "app.dll", "--module"])
        .arg(&standard)
        .output()
        .unwrap();
    assert_eq!(duplicate.status.code(), Some(37), "{duplicate:?}");
    let duplicate_arguments = fs::read_to_string(&recorded).unwrap();
    assert_eq!(duplicate_arguments.matches("--module\n").count(), 1);
    let roots = root.join("retained metadata.json");
    fs::write(
        &roots,
        r#"{"schemaVersion":5,"types":[],"moduleCatalogs":[],"moduleFunctions":[]}"#,
    )
    .unwrap();
    let retained = Command::new(&runtime)
        .env("NEO_TEST_ARGS", &recorded)
        .args(["run", "app.dll", "--reflection-roots"])
        .arg(&roots)
        .args(["--", "--reflection-roots", "guest policy"])
        .output()
        .unwrap();
    assert_eq!(retained.status.code(), Some(37), "{retained:?}");
    let forwarded = fs::read_to_string(&recorded).unwrap();
    assert!(forwarded.contains(&format!("--reflection-roots\n{}\n", roots.display())));
    assert!(forwarded.ends_with(
        "--jit-entry-argument\n--reflection-roots\n--jit-entry-argument\nguest policy\n"
    ));
    for options in [
        vec!["--reflection-roots"],
        vec!["--reflection-roots", "--execution", "jit"],
        vec![
            "--reflection-roots",
            roots.to_str().unwrap(),
            "--reflection-roots",
            roots.to_str().unwrap(),
        ],
        vec![
            "--execution",
            "interpreter",
            "--reflection-roots",
            roots.to_str().unwrap(),
        ],
        vec!["--reflection-roots", "missing-retention.json"],
    ] {
        fs::remove_file(&recorded).unwrap_or(());
        let rejected = Command::new(&runtime)
            .env("NEO_TEST_ARGS", &recorded)
            .args(["run", "app.dll"])
            .args(options)
            .output()
            .unwrap();
        assert!(!rejected.status.success(), "{rejected:?}");
        assert!(!recorded.exists(), "invalid CLI policy reached the backend");
    }
    fs::remove_file(&compiler).unwrap();
    let missing = Command::new(&runtime)
        .args(["run", "app.dll"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("Missing JIT compiler"));
    let guest = root.join("answer.neoil");
    fs::write(
        &guest,
        ".module Answer\n.entry Main\n.function Main() -> Int32\nldc.i4 42\nret\n.end\n",
    )
    .unwrap();
    let interpreted = Command::new(&runtime)
        .arg("run")
        .arg(&guest)
        .args(["--execution", "interpreter"])
        .output()
        .unwrap();
    assert_eq!(interpreted.status.code(), Some(42), "{interpreted:?}");
    fs::remove_dir_all(root).unwrap();
}
