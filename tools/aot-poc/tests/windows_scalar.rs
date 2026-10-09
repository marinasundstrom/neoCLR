use object::{Object, ObjectSection, ObjectSymbol};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let base = std::env::var_os("NEOCLR_WINDOWS_AOT_EVIDENCE")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        fs::create_dir_all(&base).unwrap();
        let path = base.join(format!(
            "neoclr-windows-scalar-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if std::env::var_os("NEOCLR_WINDOWS_AOT_EVIDENCE").is_none() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
const SCALAR: &str = include_str!("../../../docs/experiments/aot-scalar/scalar.neoil");
const FLOW: &str = include_str!("../../../docs/experiments/aot-scalar/control-flow.neoil");
const FAULTS: &str = include_str!("../../../docs/experiments/aot-scalar/faults.neoil");
const CONSOLE: &str = include_str!("../../../docs/experiments/aot-scalar/console.neoil");
fn compile(dir: &Temp, source: &str, flags: &[&str]) -> std::process::Output {
    fs::write(dir.0.join("input.neoil"), source).unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .args([
            dir.0.join("input.neoil"),
            PathBuf::from("Calculate"),
            dir.0.join("app.obj"),
        ])
        .args(["--target", "x86_64-pc-windows-msvc"])
        .args(flags)
        .output()
        .unwrap()
}
#[test]
fn scalar_flow_faults_and_console_emit_x64_coff() {
    for (source, flags) in [
        (SCALAR, vec![]),
        (FLOW, vec![]),
        (FAULTS, vec![]),
        (CONSOLE, vec!["--console"]),
    ] {
        let dir = Temp::new();
        let result = compile(&dir, source, &flags);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let bytes = fs::read(dir.0.join("app.obj")).unwrap();
        let obj = object::File::parse(bytes.as_slice()).unwrap();
        assert_eq!(obj.format(), object::BinaryFormat::Coff);
        assert_eq!(obj.architecture(), object::Architecture::X86_64);
        assert!(
            obj.symbols()
                .any(|s| s.name().ok() == Some("neoclr_entry_v2") && s.is_definition())
        );
        assert!(obj.sections().any(|s| s.relocations().next().is_some()));
        if source == CONSOLE {
            assert!(obj.symbols().any(|s| s.name().ok()
                == Some("neoclr_console_write_line_utf8_v1")
                && s.is_undefined()));
        }
        assert!(!compile(&dir, source, &flags).status.success());
        assert_eq!(fs::read(dir.0.join("app.obj")).unwrap(), bytes);
    }
}
#[test]
fn raven_metadata_hello_emits_windows_object() {
    let dir = Temp::new();
    let input = dir.0.join("RavenHello.pe");
    fs::write(
        &input,
        include_bytes!("../../../docs/experiments/aot-hello/RavenHello.pe"),
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(input)
        .arg("@entry")
        .arg(dir.0.join("app.obj"))
        .args(["--console", "--target", "x86_64-pc-windows-msvc"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = fs::read(dir.0.join("app.obj")).unwrap();
    let obj = object::File::parse(bytes.as_slice()).unwrap();
    assert_eq!(obj.format(), object::BinaryFormat::Coff);
    assert_eq!(obj.architecture(), object::Architecture::X86_64);
    assert!(
        obj.symbols()
            .any(|s| s.name().ok() == Some("neoclr_entry_v2") && s.is_definition())
    );
}

#[test]
fn unsupported_windows_profiles_and_targets_publish_nothing() {
    for mode in ["--inspect", "--closed-world"] {
        let dir = Temp::new();
        fs::write(dir.0.join("input.neoil"), SCALAR).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
            .arg(mode)
            .args([
                dir.0.join("input.neoil"),
                PathBuf::from("Calculate"),
                dir.0.join("app.obj"),
            ])
            .args(["--target", "x86_64-pc-windows-msvc"])
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("inspection and closed-world"));
        assert!(!dir.0.join("app.obj").exists());
    }
    let dir = Temp::new();
    let result = compile(
        &dir,
        ".module DoubleTest\n.function Calculate(Int32 value) -> Double\nldc.r8 1.0\nret\n.end\n",
        &[],
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("value/managed profiles"));
    assert!(!dir.0.join("app.obj").exists());
    for flags in [
        vec!["--fault-details"],
        vec!["--target", "aarch64-apple-darwin"],
        vec!["--inspect"],
    ] {
        let dir = Temp::new();
        let result = compile(&dir, SCALAR, &flags);
        assert!(!result.status.success());
        assert!(!dir.0.join("app.obj").exists());
    }
    let dir = Temp::new();
    fs::write(dir.0.join("input.neoil"), SCALAR).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .args([
            dir.0.join("input.neoil"),
            PathBuf::from("Calculate"),
            dir.0.join("app.obj"),
        ])
        .args(["--target", "x86_64-unknown-linux-gnu"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unsupported AOT target"));
    assert!(!dir.0.join("app.obj").exists());
}

#[test]
#[cfg_attr(
    not(all(target_os = "windows", target_arch = "x86_64")),
    ignore = "requires Windows x64 and MSVC"
)]
fn windows_c_consumer_executes_calls_branches_faults_and_utf8() {
    let mut outcomes = Vec::new();
    let save = |outcomes: &Vec<serde_json::Value>, passed| {
        if let Some(path) = std::env::var_os("NEOCLR_WINDOWS_AOT_EVIDENCE") {
            fs::write(
                PathBuf::from(path).join("windows-execution.json"),
                serde_json::to_vec_pretty(
                    &serde_json::json!({"passed": passed, "outcomes": outcomes}),
                )
                .unwrap(),
            )
            .unwrap();
        }
    };
    save(&outcomes, false);
    for (source, flags) in [
        (SCALAR, vec![]),
        (FLOW, vec![]),
        (FAULTS, vec![]),
        (CONSOLE, vec!["--console"]),
    ] {
        let dir = Temp::new();
        let compiled = compile(&dir, source, &flags);
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let host =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../native/windows-scalar-host.c");
        let linked = Command::new("cl")
            .current_dir(&dir.0)
            .args(["/nologo", "/W4", "/WX", "/std:c11", "/MT", "/Fe:app.exe"])
            .arg(host)
            .arg(dir.0.join("app.obj"))
            .output()
            .unwrap();
        fs::write(dir.0.join("link.stdout.log"), &linked.stdout).unwrap();
        fs::write(dir.0.join("link.stderr.log"), &linked.stderr).unwrap();
        assert!(
            linked.status.success(),
            "{}{}",
            String::from_utf8_lossy(&linked.stdout),
            String::from_utf8_lossy(&linked.stderr)
        );
        let module = neoclr::assemble(source).unwrap();
        let program = neoclr::LoadedProgram::new(&module).unwrap();
        let function = program
            .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
            .unwrap();
        for value in [i32::MIN, -2, -1, 0, 1, 20, 21, i32::MAX] {
            let interpreted =
                function.invoke(vec![neoclr::Value::Int32(value)], neoclr::Limits::default());
            let result = Command::new(dir.0.join("app.exe"))
                .arg(value.to_string())
                .output()
                .unwrap();
            outcomes.push(serde_json::json!({
                "directory": dir.0, "input": value, "exitCode": result.status.code(),
                "stdout": String::from_utf8_lossy(&result.stdout),
                "stderr": String::from_utf8_lossy(&result.stderr),
            }));
            save(&outcomes, false);
            if source == FAULTS && (value == -1 || value == i32::MAX) {
                let fault = interpreted.unwrap_err();
                assert_eq!(
                    fault.code,
                    if value == -1 {
                        neoclr::FaultCode::DivideByZero
                    } else {
                        neoclr::FaultCode::ArithmeticOverflow
                    }
                );
                assert_eq!(result.status.code(), Some(1));
                assert!(result.stdout.is_empty());
                assert_eq!(
                    result.stderr,
                    if value == -1 {
                        b"DivideByZero\n".as_slice()
                    } else {
                        b"ArithmeticOverflow\n".as_slice()
                    }
                );
                continue;
            }
            let expected = if source == SCALAR {
                value.wrapping_mul(2).wrapping_add(2)
            } else if source == FLOW {
                if value < 0 {
                    -1
                } else if value > 20 {
                    -2
                } else {
                    value * (value - 1)
                }
            } else if source == FAULTS {
                100 / (value + 1)
            } else {
                value
            };
            let prefix = if source == CONSOLE {
                format!(
                    "Hello from ARM64 AOT — hej, שלום, 🌍!\n{}\n\nNUL: A\0B; combining: é\n",
                    if value == 0 { "zero" } else { "nonzero" }
                )
            } else {
                String::new()
            };
            let execution = interpreted.unwrap();
            assert_eq!(execution.value, neoclr::Value::Int32(expected));
            assert_eq!(execution.stdout, prefix.as_bytes());
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(result.stdout, format!("{prefix}{expected}\n").into_bytes());
            assert!(result.stderr.is_empty());
        }
    }
    save(&outcomes, true);
}

#[test]
#[cfg_attr(
    not(all(target_os = "windows", target_arch = "x86_64")),
    ignore = "requires Windows x64 and MSVC"
)]
fn raven_hello_runs_as_standalone_windows_executable() {
    let fixture = include_bytes!("../../../docs/experiments/aot-hello/RavenHello.pe");
    assert_windows_hello(fixture, "raven-execution.json");
}

#[test]
#[cfg_attr(
    not(all(target_os = "windows", target_arch = "x86_64")),
    ignore = "requires Windows x64, MSVC and freshly compiled Raven input"
)]
fn fresh_raven_source_runs_as_standalone_windows_executable() {
    let input = std::env::var_os("NEOCLR_WINDOWS_FRESH_RAVEN")
        .expect("run scripts/validate-windows-aot.py with --raven-source");
    let bytes = fs::read(input).unwrap();
    assert_windows_hello(&bytes, "fresh-raven-execution.json");
}

fn assert_windows_hello(fixture: &[u8], marker: &str) {
    let module = neoclr::metadata_container::decode(fixture).unwrap();
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    let interpreted = program.run(neoclr::ExecutionOptions::default()).unwrap();
    assert_eq!(interpreted.stdout, b"Hello, world!\n");
    let mut outcomes = Vec::new();
    let save = |outcomes: &Vec<serde_json::Value>, passed| {
        if let Some(path) = std::env::var_os("NEOCLR_WINDOWS_AOT_EVIDENCE") {
            fs::write(
                PathBuf::from(path).join(marker),
                serde_json::to_vec_pretty(
                    &serde_json::json!({"passed": passed, "outcomes": outcomes}),
                )
                .unwrap(),
            )
            .unwrap();
        }
    };
    save(&outcomes, false);
    for (container, bytes) in [
        ("PE/#Neo", fixture.to_vec()),
        (
            "NEOX",
            neoclr::metadata_container::write_module(&module).unwrap(),
        ),
    ] {
        let dir = Temp::new();
        let input = dir.0.join("raven.bin");
        fs::write(&input, bytes).unwrap();
        let compiled = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
            .arg(&input)
            .arg("@entry")
            .arg(dir.0.join("app.obj"))
            .args(["--console", "--target", "x86_64-pc-windows-msvc"])
            .output()
            .unwrap();
        fs::write(dir.0.join("compile.stdout.log"), &compiled.stdout).unwrap();
        fs::write(dir.0.join("compile.stderr.log"), &compiled.stderr).unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let host =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../native/windows-scalar-host.c");
        let linked = Command::new("cl")
            .current_dir(&dir.0)
            .args([
                "/nologo",
                "/W4",
                "/WX",
                "/std:c11",
                "/MT",
                "/DNEOCLR_HELLO_HOST",
                "/Fe:app.exe",
            ])
            .arg(host)
            .arg(dir.0.join("app.obj"))
            .output()
            .unwrap();
        fs::write(dir.0.join("link.stdout.log"), &linked.stdout).unwrap();
        fs::write(dir.0.join("link.stderr.log"), &linked.stderr).unwrap();
        assert!(
            linked.status.success(),
            "{}{}",
            String::from_utf8_lossy(&linked.stdout),
            String::from_utf8_lossy(&linked.stderr)
        );
        // Deploy only the executable; no metadata, interpreter or compiler assets.
        let deployed = dir.0.join("standalone");
        fs::create_dir(&deployed).unwrap();
        fs::copy(dir.0.join("app.exe"), deployed.join("hello.exe")).unwrap();
        let result = Command::new(deployed.join("hello.exe"))
            .current_dir(&deployed)
            .output()
            .unwrap();
        outcomes.push(serde_json::json!({
            "container": container, "directory": deployed, "exitCode": result.status.code(),
            "stdout": String::from_utf8_lossy(&result.stdout),
            "stderr": String::from_utf8_lossy(&result.stderr),
        }));
        save(&outcomes, false);
        assert_eq!(result.status.code(), Some(0));
        assert_eq!(result.stdout, interpreted.stdout);
        assert_eq!(result.stderr, interpreted.stderr);
        assert_eq!(fs::read_dir(&deployed).unwrap().count(), 1);
    }
    save(&outcomes, true);
}
