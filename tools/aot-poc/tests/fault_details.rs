use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "aot-fault-details-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn compile(dir: &Temp, source: &str) -> Output {
    let path = dir.0.join("input.neoil");
    fs::write(&path, source).unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .args(["--fault-details"])
        .arg(path)
        .arg("Calculate")
        .arg(dir.0.join("app.o"))
        .output()
        .unwrap()
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn link(dir: &Temp, host: Option<&str>) -> PathBuf {
    let base =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-fault-details");
    let host = if let Some(source) = host {
        let p = dir.0.join("host.c");
        fs::write(&p, source).unwrap();
        p
    } else {
        base.join("host.c")
    };
    let exe = dir.0.join("app");
    let r = Command::new("clang")
        .args([
            "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(&base)
        .arg(host)
        .arg(base.join("render.c"))
        .arg(dir.0.join("app.o"))
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let imports = Command::new("nm")
        .arg("-u")
        .arg(dir.0.join("app.o"))
        .output()
        .unwrap();
    assert!(imports.status.success() && imports.stdout.is_empty());
    exe
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn message_trace_first_fault_and_reused_context() {
    let dir = Temp::new();
    let source = include_str!("../../../docs/experiments/aot-scalar/user-fault.neoil");
    let r = compile(&dir, source);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let exe = link(
        &dir,
        Some(
            r#"
#include "fault-details.h"
#include <string.h>
static int equal(const neoclr_aot_text *t, const char *s) { return t && t->length == strlen(s) && !memcmp(t->bytes,s,t->length); }
int main(void) {
    struct { uint64_t before; neoclr_aot_fault fault; uint64_t after; } storage;
    storage.before=123; storage.after=456;
    neoclr_aot_fault *f=&storage.fault;
    for (int n=0;n<3;n++) {
        int32_t result=12345;
        if (neoclr_entry_v3(1,&result,f)!=4 || result!=12345 || f->code!=4 || f->frame_count!=3) return 1;
        if (!equal(f->message,"input was rejected") || !equal(f->frames[0].function,"Check") || !equal(f->frames[1].function,"Forward") || !equal(f->frames[2].function,"Calculate")) return 2;
        if (f->frames[0].instruction!=2 || f->frames[1].instruction!=1 || f->frames[2].instruction!=1) return 3;
        if (neoclr_entry_v3(0,&result,f) || result!=42 || f->code || f->frame_count || f->message) return 4;
    }
    return storage.before!=123 || storage.after!=456;
}
"#,
        ),
    );
    assert!(Command::new(exe).env_clear().status().unwrap().success());
    let exe = link(&dir, None);
    let success = Command::new(&exe).arg("0").output().unwrap();
    assert_eq!(success.status.code(), Some(42));
    assert!(success.stdout.is_empty() && success.stderr.is_empty());
    let failure = Command::new(exe).arg("1").output().unwrap();
    assert_eq!(failure.status.code(), Some(1));
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn arithmetic_and_erased_faults_capture_leaf_and_caller() {
    for (body, expected) in [
        ("ldc.i4 1\nldc.i4 0\ndiv", "DivideByZero: Division by zero"),
        (
            "ldc.i4 2147483647\nldc.i4 1\nadd.ovf",
            "ArithmeticOverflow: Arithmetic overflow",
        ),
        (
            "ldc.i4 1\nvalue.pack Int32\nvalue.unpack Byte",
            "RuntimeError: Runtime error",
        ),
        (
            "fault \"Hej 🌍\\u0000message\"",
            "UserFault: Hej 🌍\0message",
        ),
    ] {
        let dir = Temp::new();
        let source = format!(
            ".module Details\n.function Leaf() -> Int32\n{body}\nfault \"later\"\n.end\n.function Calculate(Int32 value) -> Int32\ncall Leaf()\nret\n.end"
        );
        let r = compile(&dir, &source);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let exe = link(&dir, None);
        let r = Command::new(exe).env_clear().output().unwrap();
        assert_eq!(r.status.code(), Some(1));
        assert!(r.stdout.is_empty());
        let stderr = String::from_utf8(r.stderr).unwrap();
        assert!(stderr.starts_with(expected), "{stderr}");
        let module = neoclr::assemble(&source).unwrap();
        let program = neoclr::LoadedProgram::new(&module).unwrap();
        let method = program
            .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
            .unwrap();
        let fault = method
            .invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default())
            .unwrap_err();
        assert_eq!(stderr, fault.diagnostic().to_string());
        assert!(
            stderr.contains("   at Leaf [instruction ")
                && stderr.ends_with("   at Calculate [instruction 0]\n"),
            "{stderr}"
        );
        assert!(!stderr.contains("later"));
    }
}
fn binding(dir: &Temp, seed: &neoclr::Module, app: &str, flags: &[&str], inspect: bool) -> Output {
    fs::write(
        dir.0.join("seed.neox"),
        neoclr::metadata_container::write_module(seed).unwrap(),
    )
    .unwrap();
    fs::write(dir.0.join("app.neoil"), app).unwrap();
    fs::write(dir.0.join("lib.neoil"), ".module Helpers\n.references ()\n").unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    if inspect {
        c.arg("--inspect")
            .arg(dir.0.join("app.neoil"))
            .args(["Calculate", "--closed-world"]);
    } else {
        c.arg("--closed-world")
            .arg(dir.0.join("app.neoil"))
            .arg("Calculate")
            .arg(dir.0.join("app.o"));
    }
    c.arg("--system")
        .arg(dir.0.join("seed.neox"))
        .arg("--module")
        .arg(dir.0.join("lib.neoil"))
        .args(flags)
        .output()
        .unwrap()
}
const APP: &str = ".module App\n.function Calculate(Int32 value) -> Int32\nldstr \"bound failure 🌍\"\ncall System.Fail(String)\nldc.i4 42\nret\n.end";
fn seed() -> neoclr::Module {
    neoclr::assemble(".module System\n.references ()\n.function System.Fail(String message) -> noresult\nldarg message\ncall neoCLR.Runtime.Fault(String)\npop\nret\n.end\n.function neoCLR.Runtime.Fault(String message) -> Void\n.methodimpl InternalCall\n.end").unwrap()
}
#[test]
fn explicit_binding_preserves_wrapper_and_source_identity() {
    let seed = seed();
    let dir = Temp::new();
    let missing = binding(&dir, &seed, APP, &["--compile-system"], false);
    assert!(!missing.status.success() && !dir.0.join("app.o").exists());
    let flags = ["--compile-system", "--bind-user-fault"];
    let inspect = binding(&dir, &seed, APP, &flags, true);
    let report: serde_json::Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(report["admission"]["accepted"], true, "{report}");
    assert_eq!(report["capabilities"]["faultDetails"], true);
    let emit = binding(&dir, &seed, APP, &flags, false);
    assert!(
        emit.status.success(),
        "{}",
        String::from_utf8_lossy(&emit.stderr)
    );
    let selection: serde_json::Value = serde_json::from_slice(&emit.stdout).unwrap();
    assert_eq!(selection, report["selection"]);
    assert_eq!(
        selection["nativeBindings"][0]["definition"]["module"],
        "System"
    );
    assert!(
        selection["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"] == "System.Fail")
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        let exe = link(&dir, None);
        let result = Command::new(exe).env_clear().output().unwrap();
        assert_eq!(result.status.code(), Some(1));
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert_eq!(
            stderr,
            "UserFault: bound failure 🌍\n   at System.Fail [instruction 1]\n   at Calculate [instruction 1]\n"
        );
        let app = neoclr::assemble(APP).unwrap();
        let program = neoclr::LoadedProgram::with_modules(&app, &seed, &[]).unwrap();
        let method = program
            .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
            .unwrap();
        let fault = method
            .invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default())
            .unwrap_err();
        assert_eq!(stderr, fault.diagnostic().to_string());
    }
}
#[test]
fn binding_rejects_missing_capability_duplicates_and_altered_service() {
    for flags in [
        vec!["--bind-user-fault"],
        vec!["--compile-system", "--bind-user-fault", "--bind-user-fault"],
        vec!["--compile-system", "--fault-details", "--fault-details"],
    ] {
        let dir = Temp::new();
        let r = binding(&dir, &seed(), APP, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    for case in 0..3 {
        let mut seed = seed();
        match case {
            0 => {
                seed.functions[1].impl_flags = 0;
                seed.functions[1].body = vec![
                    neoclr::metadata::Instruction::Void,
                    neoclr::metadata::Instruction::Return,
                ];
            }
            1 => {
                seed.functions[1].visibility = neoclr::metadata::Visibility::Internal;
                seed.functions[0].body = vec![
                    neoclr::metadata::Instruction::Void,
                    neoclr::metadata::Instruction::Pop,
                    neoclr::metadata::Instruction::Return,
                ];
            }
            _ => {
                seed.functions[1].no_result = true;
                seed.functions[0].body.remove(2);
            }
        }
        let dir = Temp::new();
        let app = if case == 1 {
            APP.replace(
                "call System.Fail(String)",
                "call neoCLR.Runtime.Fault(String)\npop",
            )
        } else {
            APP.to_owned()
        };
        let r = binding(
            &dir,
            &seed,
            &app,
            &["--compile-system", "--bind-user-fault"],
            false,
        );
        assert!(!r.status.success(), "case {case} accepted");
        assert!(!dir.0.join("app.o").exists());
    }
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn maximum_native_trace_fits_caller_storage() {
    let mut source = String::from(
        ".module Deep\n.function Calculate(Int32 input) -> Int32\ncall F1()\nret\n.end\n",
    );
    for i in 1..128 {
        source.push_str(&format!(".function F{i}() -> Int32\n"));
        if i == 127 {
            source.push_str("fault \"deep\"\n");
        } else {
            source.push_str(&format!("call F{}()\nret\n", i + 1));
        }
        source.push_str(".end\n");
    }
    let dir = Temp::new();
    let r = compile(&dir, &source);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let exe = link(
        &dir,
        Some(
            "#include \"fault-details.h\"\nint main(void) { struct { uint64_t before; neoclr_aot_fault f; uint64_t after; } s; s.before=123;s.after=456;int32_t value=42;int32_t status=neoclr_entry_v3(0,&value,&s.f);return status!=4 || value!=42 || s.f.frame_count!=64 || s.f.truncated!=1 || s.before!=123 || s.after!=456; }",
        ),
    );
    assert!(Command::new(exe).env_clear().status().unwrap().success());
}

#[test]
fn source_owned_noresult_failure_uses_its_original_identity() {
    let dir = Temp::new();
    let source=neoclr::assemble(".module SourceFailure\n.function neoCLR.Runtime.Fail(String message) -> noresult\n.methodimpl InternalCall\n.end").unwrap();
    fs::write(
        dir.0.join("source.neox"),
        neoclr::metadata_container::write_module(&source).unwrap(),
    )
    .unwrap();
    fs::write(dir.0.join("seed.neoil"), ".module System\n.references ()\n").unwrap();
    let modules = neoclr::assembler::assemble_modules(&[
        &APP.replace("call System.Fail(String)","call neoCLR.Runtime.Fail(String)"),
        ".module SourceFailure\n.function neoCLR.Runtime.Fail(String message) -> noresult\n.methodimpl InternalCall\n.end"
    ]).unwrap();
    fs::write(
        dir.0.join("app.neox"),
        neoclr::metadata_container::write_module(&modules[0]).unwrap(),
    )
    .unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg("--closed-world")
        .arg(dir.0.join("app.neox"))
        .arg("Calculate")
        .arg(dir.0.join("app.o"))
        .arg("--module")
        .arg(dir.0.join("source.neox"))
        .arg("--system")
        .arg(dir.0.join("seed.neoil"))
        .args(["--compile-system", "--bind-user-fault"])
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(
        report["nativeBindings"][0]["definition"]["module"],
        "SourceFailure"
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        let exe = link(&dir, None);
        let r = Command::new(exe).output().unwrap();
        assert_eq!(r.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&r.stderr)
                .starts_with("UserFault: bound failure 🌍\n   at Calculate")
        );
    }
}
