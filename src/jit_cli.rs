//! Development CLI adapter. The optional compiler remains outside the runtime crate.
//! No implicit interpreter fallback: unsupported native programs must report their gap.
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn same_path(left: &Path, right: &Path) -> bool {
    left == right || matches!((left.canonicalize(), right.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

// This adapter forwards the independent CLI options without changing their defaults.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    input: &str,
    modules: &[&str],
    system: Option<&str>,
    object_root: Option<&str>,
    host: Option<&str>,
    compiler: Option<&str>,
    reflection_roots: Option<&str>,
    arguments: &[String],
) -> Result<i32, String> {
    let windows = cfg!(all(target_os = "windows", target_arch = "x86_64"));
    if !windows && !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Err("JIT execution currently requires macOS ARM64 or Windows x64".into());
    }
    if let Some(path) = reflection_roots {
        if !Path::new(path).is_file() {
            return Err(format!("Missing reflection retention file: {path}"));
        }
    }
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let bin = executable
        .parent()
        .ok_or("Cannot locate runtime installation")?;
    let libraries = bin.join("../lib");
    let catalog_path = libraries.join("bundle.json");
    let catalog = if (system.is_none() || object_root.is_none() || modules.is_empty())
        && catalog_path.is_file()
    {
        Some(
            serde_json::from_slice::<serde_json::Value>(
                &std::fs::read(&catalog_path)
                    .map_err(|e| format!("Cannot read installed class-library catalog: {e}"))?,
            )
            .map_err(|e| format!("Invalid installed class-library catalog: {e}"))?,
        )
    } else {
        None
    };
    let mut modules: Vec<PathBuf> = modules.iter().map(PathBuf::from).collect();
    if let Some(catalog) = &catalog {
        for name in catalog["assemblyNames"]
            .as_array()
            .ok_or("Missing installed assemblyNames")?
        {
            let path = libraries.join(format!(
                "{}.dll",
                name.as_str().ok_or("Invalid installed assembly name")?
            ));
            if !modules.iter().any(|module| same_path(module, &path)) {
                modules.push(path);
            }
        }
    }
    let system = system
        .map(PathBuf::from)
        .or_else(|| {
            catalog
                .as_ref()
                .and_then(|c| c["runtimeSeed"].as_str())
                .map(|name| libraries.join(name))
        })
        .ok_or("JIT requires installed class libraries or an explicit --system runtime seed")?;
    let object_root = object_root
        .map(PathBuf::from)
        .or_else(|| {
            catalog
                .as_ref()
                .map(|_| libraries.join("System.Runtime.dll"))
        })
        .ok_or("JIT requires installed class libraries or an explicit --object-root dependency")?;
    if !modules.iter().any(|module| same_path(module, &object_root)) {
        return Err("JIT --object-root must also be supplied as --module".into());
    }
    let host = host.map(PathBuf::from).unwrap_or_else(|| {
        bin.join(if windows {
            "neoclr-jit-host.dll"
        } else {
            "neoclr-jit-host.dylib"
        })
    });
    let compiler = compiler.map(PathBuf::from).unwrap_or_else(|| {
        bin.join(if windows {
            "neoclr-aot-poc.exe"
        } else {
            "neoclr-aot-poc"
        })
    });
    for (role, path) in [
        ("compiler", &compiler),
        ("helper library", &host),
        ("runtime seed", &system),
        ("Object library", &object_root),
    ] {
        if !path.is_file() {
            return Err(format!(
                "Missing JIT {role}: {}. Install a matching JIT bundle or select --execution interpreter explicitly.",
                path.display()
            ));
        }
    }
    let mut command = Command::new(&compiler);
    command.args([
        "--jit-run",
        "--closed-world",
        input,
        "@entry",
        host.to_str().ok_or("JIT host path is not UTF-8")?,
        "--system",
        system.to_str().ok_or("JIT system path is not UTF-8")?,
        "--object-root",
        object_root.to_str().ok_or("JIT Object path is not UTF-8")?,
    ]);
    for module in modules {
        command.arg("--module").arg(module);
    }
    command.args([
        "--compile-system",
        "--bind-user-fault",
        "--bind-stack-trace",
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
        "--bind-cryptography",
        "--bind-tls",
        "--bind-task-queue",
        "--bind-scheduler",
        "--bind-timers",
        "--bind-socket-listener",
        "--bind-socket-accept",
        "--bind-socket-transfer",
        "--bind-socket-client",
    ]);
    if !windows {
        command.arg("--bind-runtime-async");
    }
    if windows {
        command.args([
            "--target",
            "x86_64-pc-windows-msvc",
            "--windows-http-experiment",
        ]);
    }
    if let Some(path) = reflection_roots {
        // Exact caller-supplied policy; the matching backend validates its schema.
        // Do not discover ambient roots or silently grant metadata retention.
        command.args(["--reflection-roots", path]);
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

/// Platforms with a qualified native JIT backend. Unsupported hosts retain the
/// interpreter default; explicit JIT requests still diagnose unsupported targets.
pub(crate) fn default_execution() -> &'static str {
    if cfg!(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )) {
        "jit"
    } else {
        "interpreter"
    }
}
