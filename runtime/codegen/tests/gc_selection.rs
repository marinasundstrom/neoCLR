use std::process::Command;
#[test]
fn collector_selection_rejects_incompatible_and_unavailable_options_before_loading() {
    for (flags, message) in [
        (vec!["--gc"], "Missing collector"),
        (vec!["--gc", "concurrent"], "Unavailable garbage collector"),
        (vec!["--gc", "mark-sweep"], "requires --native-gc"),
        (
            vec!["--gc", "mark-sweep", "--gc", "mark-sweep"],
            "duplicate --gc",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
            .args(flags)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{output:?}"
        );
    }
}

#[test]
fn explicit_native_selection_preserves_generated_object() {
    use std::fs;
    let directory =
        std::env::temp_dir().join(format!("neoclr-gc-selection-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("input.neox");
    let core = directory.join("core.neox");
    let system = directory.join("system.neox");
    for (path, source) in [
        (
            &input,
            ".module Selected\n.references ()\n.function Calculate(Int32 value) -> Int32\nldarg value\nret\n.end\n",
        ),
        (
            &core,
            ".module Core\n.references ()\n.type class abstract System.Object\n.method instance virtual ToString() -> String\nldstr \"root\"\nret\n.end\n.method instance virtual Equals(System.Object other) -> Boolean\nldc.bool false\nret\n.end\n.method instance virtual GetHashCode() -> Int32\nldc.i4 -1\nret\n.end\n.end\n",
        ),
        (&system, ".module System\n.references ()\n"),
    ] {
        fs::write(
            path,
            neoclr::metadata_container::write_module(&neoclr::assemble(source).unwrap()).unwrap(),
        )
        .unwrap();
    }
    for explicit in [false, true] {
        let output = directory.join(if explicit { "selected.o" } else { "default.o" });
        let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        command
            .arg("--closed-world")
            .arg(&input)
            .args(["Calculate"])
            .arg(output)
            .args(["--compile-system", "--reference-arena", "--native-gc"])
            .arg("--system")
            .arg(&system)
            .arg("--module")
            .arg(&core)
            .arg("--object-root")
            .arg(&core);
        if explicit {
            command.args(["--gc", "mark-sweep"]);
        }
        let result = command.output().unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    assert_eq!(
        fs::read(directory.join("selected.o")).unwrap(),
        fs::read(directory.join("default.o")).unwrap()
    );
    fs::remove_dir_all(directory).unwrap();
}
