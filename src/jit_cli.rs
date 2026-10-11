//! Development CLI adapter. The optional compiler remains outside the runtime crate.
//! No implicit interpreter fallback: unsupported native programs must report their gap.
use std::{path::PathBuf, process::Command};

pub(crate) fn run(
    input: &str,
    modules: &[&str],
    system: Option<&str>,
    object_root: Option<&str>,
    host: Option<&str>,
    compiler: Option<&str>,
    arguments: &[String],
) -> Result<i32, String> {
    let windows = cfg!(all(target_os = "windows", target_arch = "x86_64"));
    if !windows && !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Err("JIT execution currently requires macOS ARM64 or Windows x64".into());
    }
    let host = host.ok_or("Development JIT requires an explicit trusted --jit-host library")?;
    let system = system.ok_or("JIT requires an explicit --system runtime seed")?;
    let object_root = object_root.ok_or("JIT requires an explicit --object-root dependency")?;
    if !modules.contains(&object_root) {
        return Err("JIT --object-root must also be supplied as --module".into());
    }
    let compiler = match compiler {
        Some(path) => PathBuf::from(path),
        None => std::env::current_exe()
            .map_err(|error| error.to_string())?
            .with_file_name(if windows {
                "neoclr-aot-poc.exe"
            } else {
                "neoclr-aot-poc"
            }),
    };
    let mut command = Command::new(&compiler);
    command.args([
        "--jit-run",
        "--closed-world",
        input,
        "@entry",
        host,
        "--system",
        system,
        "--object-root",
        object_root,
    ]);
    for module in modules {
        command.args(["--module", module]);
    }
    command.args([
        "--compile-system",
        "--bind-user-fault",
        "--reference-arena",
        "--native-gc",
        "--bind-console-read-byte",
        "--bind-console-write-line",
        "--bind-console-stream-output",
        "--bind-int32-to-string",
        "--bind-utf8-text",
        "--native-stack-budget",
        "--bind-integer-text",
        "--bind-character-text",
        "--bind-system-culture",
        "--bind-time-zone-rules",
        "--bind-task-queue",
        "--bind-scheduler",
        "--bind-timers",
        "--bind-socket-listener",
        "--bind-socket-accept",
        "--bind-socket-transfer",
        "--bind-socket-client",
    ]);
    if windows {
        command.args([
            "--target",
            "x86_64-pc-windows-msvc",
            "--windows-http-experiment",
        ]);
    }
    for argument in arguments {
        command.args(["--jit-entry-argument", argument]);
    }
    // Inherit all streams: HTTP readiness, interactive input and fault diagnostics
    // remain live. The backend's --jit-run returns the guest status without a footer.
    let status = command.status().map_err(|error| {
        format!(
            "Cannot execute JIT compiler {}: {error}",
            compiler.display()
        )
    })?;
    status
        .code()
        .ok_or_else(|| "JIT compiler terminated without an exit status".into())
}
