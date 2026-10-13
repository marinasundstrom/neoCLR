//! The CLI and an embedding caller use the same backend implementation.
use std::{ffi::OsString, fs, path::PathBuf, process::Command};

struct Workspace(PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

#[test]
fn driver_returns_to_host_after_errors_compilation_and_jit_invocations() {
    let directory = Workspace(std::env::temp_dir().join(format!("neoclr-hosting-{}", std::process::id())));
    fs::create_dir(&directory.0).unwrap();
    let input = directory.0.join("input.neoil");
    fs::write(&input, ".module Hosted\n.function Calculate(Int32 value) -> Int32\nldc.i4 12\nldarg value\ndiv\nret\n.end\n").unwrap();
    assert!(neoclr_aot_poc::run(vec![]).is_err());
    let target = if cfg!(windows) { "x86_64-pc-windows-msvc" } else { "aarch64-apple-darwin" };
    let object = directory.0.join("library.o");
    let cli_object = directory.0.join("cli.o");
    let args = |output: &std::path::Path| -> Vec<OsString> {
        vec![input.as_os_str().into(), "Calculate".into(), output.as_os_str().into(), "--target".into(), target.into()]
    };
    assert_eq!(neoclr_aot_poc::run(args(&object)).unwrap(), 0);
    let original = fs::read(&object).unwrap();
    // A rejected write must leave the artifact intact and the caller usable.
    assert!(neoclr_aot_poc::run(args(&object)).is_err());
    assert_eq!(fs::read(&object).unwrap(), original);
    let cli = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc")).args(args(&cli_object)).output().unwrap();
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    assert_eq!(fs::read(&cli_object).unwrap(), original);

    #[cfg(any(all(target_os = "macos", target_arch = "aarch64"), all(target_os = "windows", target_arch = "x86_64")))]
    {
        // Diagnostic scalar execution reports guest faults in its result, not as a
        // driver error. A fresh request can follow the terminated invocation.
        for argument in ["4", "0", "3"] {
            assert_eq!(neoclr_aot_poc::run(vec!["--jit-scalar".into(), input.as_os_str().into(),
                "Calculate".into(), argument.into(), "--repetitions".into(), "3".into()]).unwrap(), 0);
        }
        let foreign = if cfg!(windows) { "aarch64-apple-darwin" } else { "x86_64-pc-windows-msvc" };
        let error = neoclr_aot_poc::run(["--jit-run", "--target", foreign, "--closed-world", "missing.dll", "@entry", "missing-host"]
            .into_iter().map(OsString::from).collect()).unwrap_err();
        assert!(error.to_string().contains("JIT target must match the executing host"));
    }
    assert_eq!(neoclr_aot_poc::run(args(&directory.0.join("after.o"))).unwrap(), 0);
}
