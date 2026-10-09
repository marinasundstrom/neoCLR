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
        let path = std::env::temp_dir().join(format!(
            "aot-console-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const SEED: &str = ".module System\n.references ()\n.function neoCLR.Runtime.ConsoleReadByte() -> System.Value\n.methodimpl InternalCall\n.end";
const APP: &str = r#"
.module App
.function Calculate() -> Int32
call neoCLR.Runtime.ConsoleReadByte()
dup
value.is Byte
brtrue Byte
dup
value.is Void
brtrue Eof
value.unpack Int32
ldc.i4 1000
add
ret
Byte:
value.unpack Byte
conv.i4
ret
Eof:
pop
ldc.i4 -1
ret
.end
"#;
fn compile(
    dir: &Temp,
    seed: &neoclr::Module,
    flags: &[&str],
    inspect: bool,
) -> std::process::Output {
    compile_source(dir, seed, APP, flags, inspect)
}
fn compile_source(
    dir: &Temp,
    seed: &neoclr::Module,
    app: &str,
    flags: &[&str],
    inspect: bool,
) -> std::process::Output {
    fs::write(
        dir.0.join("seed.neox"),
        neoclr::metadata_container::write_module(seed).unwrap(),
    )
    .unwrap();
    fs::write(dir.0.join("app.neoil"), app).unwrap();
    fs::write(dir.0.join("lib.neoil"), ".module Helpers\n.references ()\n").unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    c.arg(if inspect {
        "--inspect"
    } else {
        "--closed-world"
    })
    .arg(dir.0.join("app.neoil"))
    .arg("Calculate");
    if inspect {
        c.arg("--closed-world");
    } else {
        c.arg(dir.0.join("app.o"));
    }
    c.arg("--system")
        .arg(dir.0.join("seed.neox"))
        .arg("--module")
        .arg(dir.0.join("lib.neoil"))
        .args(flags)
        .output()
        .unwrap()
}
#[test]
fn input_binding_is_explicit_and_exact() {
    for flags in [
        vec![],
        vec!["--compile-system"],
        vec!["--bind-console-read-byte"],
        vec![
            "--compile-system",
            "--bind-console-read-byte",
            "--bind-console-read-byte",
        ],
    ] {
        let dir = Temp::new();
        let r = compile(&dir, &neoclr::assemble(SEED).unwrap(), &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    let mut seed = neoclr::assemble(SEED).unwrap();
    seed.functions[0].impl_flags = 0;
    seed.functions[0].body = vec![
        neoclr::metadata::Instruction::Void,
        neoclr::metadata::Instruction::PackValue(neoclr::metadata::Type::Void),
        neoclr::metadata::Instruction::Return,
    ];
    let dir = Temp::new();
    let r = compile(
        &dir,
        &seed,
        &["--compile-system", "--bind-console-read-byte"],
        false,
    );
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("exact neoCLR.Runtime.ConsoleReadByte"));
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn input_transport_preserves_all_bytes_and_distinct_outcomes() {
    let dir = Temp::new();
    let seed = neoclr::assemble(SEED).unwrap();
    let flags = ["--compile-system", "--bind-console-read-byte"];
    let r = compile(&dir, &seed, &flags, true);
    let inspection: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(inspection["admission"]["accepted"], true, "{inspection}");
    let r = compile(&dir, &seed, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let selection: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(selection, inspection["selection"]);
    fs::write(dir.0.join("host.c"), r#"
#include "fault-details.h"
#include "console.h"
static int32_t supplied;
int32_t neoclr_console_read_byte_v1(void) { return supplied; }
int main(void) {
    neoclr_aot_fault fault;
    for (int n=-5;n<=257;n++) {
        supplied=n; int32_t result=-99;
        int expected=n>=0 && n<=255 ? n : n==-1 ? -1 : n==-2 ? 1001 : n==-3 ? 1002 : 1000;
        if (neoclr_entry_v3(0,&result,&fault)!=0 || fault.code!=0 || fault.frame_count!=0 || result!=expected) return 1;
    }
    return 0;
}
"#).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    let r = Command::new("clang")
        .args([
            "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(base.join("aot-fault-details"))
        .arg("-I")
        .arg(base.join("aot-console"))
        .arg(dir.0.join("host.c"))
        .arg(dir.0.join("app.o"))
        .arg("-o")
        .arg(dir.0.join("app"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(
        Command::new(dir.0.join("app"))
            .env_clear()
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn duplicate_service_declarations_keep_the_callers_module_identity() {
    let dir = Temp::new();
    let mut modules = neoclr::assembler::assemble_modules(&[
        ".module App\n.function Calculate() -> Int32\ncall Read()\nvalue.unpack Byte\nconv.i4\nret\n.end",
        ".module ConsoleLibrary\n.function Read() -> System.Value\ncall neoCLR.Runtime.ConsoleReadByte()\nret\n.end\n.function neoCLR.Runtime.ConsoleReadByte() -> System.Value\n.methodimpl InternalCall\n.end",
    ]).unwrap();
    if let neoclr::metadata::Instruction::Call(target) = &mut modules[1].functions[0].body[0] {
        target.definition = None;
    } else {
        panic!("expected call");
    }
    for (name, module) in [
        ("app.neox", &modules[0]),
        ("lib.neox", &modules[1]),
        ("seed.neox", &neoclr::assemble(SEED).unwrap()),
    ] {
        fs::write(
            dir.0.join(name),
            neoclr::metadata_container::write_module(module).unwrap(),
        )
        .unwrap();
    }
    let r = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg("--inspect")
        .arg(dir.0.join("app.neox"))
        .args([
            "Calculate",
            "--closed-world",
            "--compile-system",
            "--bind-console-read-byte",
        ])
        .arg("--system")
        .arg(dir.0.join("seed.neox"))
        .arg("--module")
        .arg(dir.0.join("lib.neox"))
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["admission"]["accepted"], true, "{report}");
    assert_eq!(
        report["selection"]["nativeBindings"][0]["definition"]["module"],
        "ConsoleLibrary"
    );
    assert_eq!(
        report["selection"]["nativeBindings"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

const OUTPUT_SEED: &str = ".module System\n.references ()\n.function neoCLR.Runtime.WriteLine(String text) -> Void\n.methodimpl InternalCall\n.end";
const OUTPUT_APP: &str = r#"
.module App
.function Calculate() -> Int32
call Say()
ldc.i4 42
ret
.end
.function Say() -> noresult
ldstr "världen 🌍\u0000end"
call neoCLR.Runtime.WriteLine(String)
pop
ret
.end
"#;
#[test]
fn output_binding_is_explicit_and_exact() {
    for flags in [
        vec!["--compile-system"],
        vec!["--bind-console-write-line"],
        vec![
            "--compile-system",
            "--bind-console-write-line",
            "--bind-console-write-line",
        ],
    ] {
        let dir = Temp::new();
        let r = compile_source(
            &dir,
            &neoclr::assemble(OUTPUT_SEED).unwrap(),
            OUTPUT_APP,
            &flags,
            false,
        );
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    let mut seed = neoclr::assemble(OUTPUT_SEED).unwrap();
    seed.functions[0].impl_flags = 0;
    seed.functions[0].body = vec![
        neoclr::metadata::Instruction::Void,
        neoclr::metadata::Instruction::Return,
    ];
    let dir = Temp::new();
    let r = compile_source(
        &dir,
        &seed,
        OUTPUT_APP,
        &["--compile-system", "--bind-console-write-line"],
        false,
    );
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("exact neoCLR.Runtime.WriteLine"));
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn line_output_preserves_utf8_nul_and_fault_caller_frames() {
    let dir = Temp::new();
    let flags = ["--compile-system", "--bind-console-write-line"];
    let seed = neoclr::assemble(OUTPUT_SEED).unwrap();
    let r = compile_source(&dir, &seed, OUTPUT_APP, &flags, true);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let inspection: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(inspection["admission"]["accepted"], true, "{inspection}");
    let r = compile_source(&dir, &seed, OUTPUT_APP, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&r.stdout).unwrap(),
        inspection["selection"]
    );
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "fault-details.h"
#include <stddef.h>
#include <string.h>
static int failure=1;
int32_t neoclr_console_write_line_utf8_v1(const uint8_t *bytes, size_t length) {
    static const char text[]="världen 🌍\0end";
    if (length!=sizeof(text)-1 || memcmp(bytes,text,length)) return 99;
    return failure;
}
int main(void) {
    neoclr_aot_fault fault; int32_t result=12345;
    if (neoclr_entry_v3(0,&result,&fault)!=3 || result!=12345 || fault.code!=3 || fault.frame_count!=2) return 1;
    if (neoclr_aot_render_fault(stderr,&fault)) return 2;
    failure=0;
    if (neoclr_entry_v3(0,&result,&fault) || result!=42 || fault.code || fault.frame_count) return 3;
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args([
            "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(base.join("aot-fault-details"))
        .arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o"))
        .arg("-o")
        .arg(dir.0.join("stub"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("stub"))
        .env_clear()
        .output()
        .unwrap();
    assert!(r.status.success(), "{r:?}");
    assert_eq!(
        String::from_utf8_lossy(&r.stderr),
        "RuntimeError: Runtime error\n   at Say [instruction 1]\n   at Calculate [instruction 0]\n"
    );
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror"])
        .arg(base.join("aot-fault-details/host.c"))
        .arg(base.join("aot-fault-details/render.c"))
        .arg(base.join("aot-scalar/console.c"))
        .arg(dir.0.join("app.o"))
        .arg("-o")
        .arg(dir.0.join("stdio"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("stdio"))
        .env_clear()
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(42));
    assert_eq!(r.stdout, "världen 🌍\0end\n".as_bytes());
    assert!(r.stderr.is_empty());
}

const TEXT_SEED: &str = ".module System\n.references ()\n.function neoCLR.Runtime.Int32ToString(Int32 value) -> String\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.WriteLine(String text) -> Void\n.methodimpl InternalCall\n.end\n.function System.Fail(String text) -> noresult\nldarg text\ncall neoCLR.Runtime.Fault(String)\npop\nret\n.end\n.function neoCLR.Runtime.Fault(String text) -> Void\n.methodimpl InternalCall\n.end";
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn invocation_text_survives_nested_calls_faults_and_reclaims_on_next_entry() {
    let dir = Temp::new();
    let source = include_str!("../../../docs/experiments/aot-console/text-lifetime.neoil");
    let flags = [
        "--compile-system",
        "--bind-user-fault",
        "--bind-console-write-line",
        "--bind-int32-to-string",
    ];
    let seed = neoclr::assemble(TEXT_SEED).unwrap();
    let r = compile_source(&dir, &seed, source, &flags, true);
    let inspection: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(inspection["admission"]["accepted"], true, "{inspection}");
    assert_eq!(
        inspection["selection"]["nativeAbi"],
        "caller-owned-text-arena-v4"
    );
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"), r#"
#include "text-arena.h"
#include <string.h>
static int calls;
int32_t neoclr_console_write_line_utf8_v1(const uint8_t *bytes, size_t length) {
    static const char *expected[]={"-2147483648","7","2147483647","7"};
    if (calls>=4 || length!=strlen(expected[calls]) || memcmp(bytes,expected[calls],length)) return 1;
    calls++; return 0;
}
int main(void) {
    struct { uint64_t before; uint64_t bytes[8]; uint64_t after; } storage = { .before=123, .after=456 };
    neoclr_aot_context ctx={ .text={(unsigned char*)storage.bytes,sizeof(storage.bytes),0} };
    int32_t result=-99;
    if (neoclr_entry_v4(INT32_MIN,&result,&ctx) || result!=42 || ctx.text.used!=33) return 1;
    if (neoclr_entry_v4(INT32_MAX,&result,&ctx) || result!=42 || ctx.text.used!=33 || calls!=4) return 2;
    result=-99;
    if (neoclr_entry_v4(1,&result,&ctx)!=4 || result!=-99 || ctx.fault.code!=4) return 3;
    if (ctx.fault.message->length!=1 || ctx.fault.message->bytes[0]!='1') return 4;
    if (neoclr_aot_render_fault(stderr,&ctx.fault)) return 5;
    ctx.text.capacity=8; result=-99;
    if (neoclr_entry_v4(0,&result,&ctx)!=5 || result!=-99 || ctx.fault.code!=5 || ctx.text.used!=0 || ctx.fault.frame_count!=1) return 6;
    if (neoclr_aot_render_fault(stderr,&ctx.fault)) return 7;
    ctx.text.capacity=16;
    if (neoclr_entry_v4(0,&result,&ctx)!=5 || result!=-99 || ctx.text.used!=9) return 12;
    if (((neoclr_aot_text*)storage.bytes)->length!=1 || ((neoclr_aot_text*)storage.bytes)->bytes[0]!='0') return 13;
    ctx.text.capacity=8;ctx.text.used=0;
    if (storage.before!=123 || storage.after!=456) return 8;
    /* No write on exhausted or malformed arena, including alignment/overflow. */
    const neoclr_aot_text *out=(void*)123;
    if (neoclr_int32_to_string_v1(0,&ctx.text,&out)!=5 || out!=(void*)123) return 9;
    ctx.text.used=UINT64_MAX;
    if (neoclr_int32_to_string_v1(0,&ctx.text,&out)!=3 || out!=(void*)123) return 10;
    ctx.text.used=0;ctx.text.capacity=64;ctx.text.data++;
    if (neoclr_int32_to_string_v1(0,&ctx.text,&out)!=3 || out!=(void*)123) return 11;
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args([
            "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(base.join("aot-console"))
        .arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o"))
        .arg("-o")
        .arg(dir.0.join("app"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("app"))
        .env_clear()
        .output()
        .unwrap();
    assert!(r.status.success(), "{r:?}");
    let stderr = String::from_utf8(r.stderr).unwrap();
    assert!(
        stderr.starts_with("UserFault: 1\n   at System.Fail [instruction 1]\n"),
        "{stderr}"
    );
    assert!(stderr.contains("NativeMemoryLimitExceeded: Native memory limit exceeded\n   at Calculate [instruction 1]\n"), "{stderr}");
}

#[test]
fn text_producer_requires_explicit_capability_and_exact_contract() {
    let source = ".module App\n.function Calculate() -> Int32\nldc.i4 42\ncall neoCLR.Runtime.Int32ToString(Int32)\npop\nldc.i4 0\nret\n.end";
    for flags in [
        vec!["--compile-system"],
        vec!["--bind-int32-to-string"],
        vec![
            "--compile-system",
            "--bind-int32-to-string",
            "--bind-int32-to-string",
        ],
    ] {
        let dir = Temp::new();
        let r = compile_source(
            &dir,
            &neoclr::assemble(TEXT_SEED).unwrap(),
            source,
            &flags,
            false,
        );
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    let mut seed = neoclr::assemble(TEXT_SEED).unwrap();
    seed.functions[0].impl_flags = 0;
    seed.functions[0].body = vec![
        neoclr::metadata::Instruction::String("wrong".into()),
        neoclr::metadata::Instruction::Return,
    ];
    let dir = Temp::new();
    let r = compile_source(
        &dir,
        &seed,
        source,
        &["--compile-system", "--bind-int32-to-string"],
        false,
    );
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("exact neoCLR.Runtime.Int32ToString"));
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn mixed_text_records_preserve_snapshots_nested_borrows_and_output_neighbors() {
    let dir = Temp::new();
    let source = include_str!("../../../docs/experiments/aot-console/text-records.neoil");
    let seed = neoclr::assemble(TEXT_SEED).unwrap();
    let flags = [
        "--compile-system",
        "--bind-console-write-line",
        "--bind-int32-to-string",
    ];
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror"])
        .arg(base.join("aot-console/text-host.c"))
        .arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-scalar/console.c"))
        .arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o"))
        .arg("-o")
        .arg(dir.0.join("app"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let function = program
        .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
        .unwrap();
    for value in [i32::MIN, 0, i32::MAX] {
        let reference = function
            .invoke(vec![neoclr::Value::Int32(value)], neoclr::Limits::default())
            .unwrap();
        let r = Command::new(dir.0.join("app"))
            .arg(value.to_string())
            .env_clear()
            .output()
            .unwrap();
        assert_eq!(r.status.code(), Some(42));
        assert_eq!(r.stdout, format!("{value}\nchanged\nneighbor\n").as_bytes());
        assert_eq!(
            r.stdout,
            format!("{}\n", reference.output.join("\n")).as_bytes()
        );
        assert!(r.stderr.is_empty());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn default_text_and_null_native_arguments_match_interpreter() {
    for consumer in [
        "call neoCLR.Runtime.WriteLine(String)\npop",
        "call System.Fail(String)",
    ] {
        let dir = Temp::new();
        let source = format!(
            ".module NullText\n.function Calculate() -> Int32\n.local String text\nldloca text\ninitobj String\nldloc text\n{consumer}\nldc.i4 42\nret\n.end"
        );
        let seed = neoclr::assemble(TEXT_SEED).unwrap();
        let flags = [
            "--compile-system",
            "--bind-user-fault",
            "--bind-console-write-line",
        ];
        let r = compile_source(&dir, &seed, &source, &flags, false);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
        let r = Command::new("clang")
            .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(base.join("aot-fault-details/host.c"))
            .arg(base.join("aot-fault-details/render.c"))
            .arg(base.join("aot-scalar/console.c"))
            .arg(dir.0.join("app.o"))
            .arg("-o")
            .arg(dir.0.join("app"))
            .output()
            .unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let app = neoclr::assemble(&source).unwrap();
        let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
        let function = program
            .resolve_function(&neoclr::assembler::parse_function_ref("Calculate()").unwrap())
            .unwrap();
        let fault = function
            .invoke(vec![], neoclr::Limits::default())
            .unwrap_err();
        let r = Command::new(dir.0.join("app"))
            .env_clear()
            .output()
            .unwrap();
        assert_eq!(r.status.code(), Some(1));
        assert_eq!(
            String::from_utf8_lossy(&r.stderr),
            fault.diagnostic().to_string()
        );
        assert!(r.stdout.is_empty());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn bounded_reference_aliases_cycles_null_faults_and_reuse_match_interpreter() {
    for virtual_call in [false, true] {
        let dir = Temp::new();
        let original = include_str!("../../../docs/experiments/aot-console/reference-cell.neoil");
        let source = if virtual_call {
            original.replace(
                "call instance Cell::Bump()",
                "callvirt instance Cell::Bump()",
            )
        } else {
            original.to_owned()
        };
        let source = source.as_str();
        let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
        let denied = compile_source(&dir, &seed, source, &["--compile-system"], false);
        assert!(!denied.status.success() && !dir.0.join("app.o").exists());
        let r = compile_source(
            &dir,
            &seed,
            source,
            &["--compile-system", "--reference-arena"],
            false,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
        fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
int main(void) {
    struct { uint64_t before; uint64_t bytes[8]; uint64_t after; } storage={ .before=123, .after=456 };
    neoclr_aot_context ctx={ .text={(unsigned char*)storage.bytes,sizeof(storage.bytes),0} };
    for (int i=0;i<10;i++) {
        int32_t result=-99;
        if (neoclr_entry_v4(1,&result,&ctx) || result!=42 || ctx.text.used!=24) return 1;
    }
    int32_t result=-99;
    if (neoclr_entry_v4(0,&result,&ctx)!=6 || result!=-99 || ctx.text.used!=0) return 2;
    if (neoclr_aot_render_fault(stderr,&ctx.fault)) return 3;
    ctx.text.capacity=16;
    if (neoclr_entry_v4(1,&result,&ctx)!=5 || result!=-99 || ctx.text.used!=0) return 4;
    if (storage.before!=123 || storage.after!=456) return 5;
    return 0;
}
"#).unwrap();
        let r = Command::new("clang")
            .args([
                "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
            ])
            .arg(base.join("aot-console"))
            .arg(dir.0.join("host.c"))
            .arg(base.join("aot-console/text-arena.c"))
            .arg(base.join("aot-fault-details/render.c"))
            .arg(dir.0.join("app.o"))
            .arg("-o")
            .arg(dir.0.join("app"))
            .output()
            .unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(dir.0.join("app"))
            .env_clear()
            .output()
            .unwrap();
        assert!(r.status.success(), "{r:?}");
        let app = neoclr::assemble(source).unwrap();
        let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
        let function = program
            .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
            .unwrap();
        let result = function
            .invoke(vec![neoclr::Value::Int32(1)], neoclr::Limits::default())
            .unwrap();
        assert_eq!(result.value, neoclr::Value::Int32(42));
        let fault = function
            .invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default())
            .unwrap_err();
        assert_eq!(
            String::from_utf8_lossy(&r.stderr),
            fault.diagnostic().to_string()
        );
    }
}

#[test]
fn reference_arena_is_explicit_and_does_not_admit_virtual_calls() {
    let source = include_str!("../../../docs/experiments/aot-console/reference-cell.neoil");
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    for flags in [
        vec!["--reference-arena"],
        vec!["--compile-system", "--reference-arena", "--reference-arena"],
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    let dir = Temp::new();
    let virtual_source = source.replace(
        "call instance Cell::Bump()",
        "callvirt instance Cell::Bump()",
    );
    let virtual_source = virtual_source.replace(".method instance Bump()", ".method instance virtual Bump()");
    let r = compile_source(
        &dir,
        &seed,
        &virtual_source,
        &["--compile-system", "--reference-arena"],
        false,
    );
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn packed_byte_arrays_bounds_borrows_and_faults_match_interpreter() {
    let dir=Temp::new();
    let source=include_str!("../../../docs/experiments/aot-console/byte-array.neoil");
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let denied=compile_source(&dir,&seed,source,&["--compile-system"],false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
    uint64_t storage[16]; memset(storage,0xa5,sizeof(storage));
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,17,0} };
    int32_t result=-99;
    if (neoclr_entry_v4(42,&result,&ctx) || result!=255 || ctx.text.used!=17) return 90;
    for (unsigned i=17;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 91;
    ctx.text.capacity=sizeof(storage);result=-99;
    int input=argc>1 ? atoi(argv[1]) : 0;
    int status=neoclr_entry_v4(input,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"]).arg(base.join("aot-console"))
        .arg(dir.0.join("host.c")).arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for value in [-4,-3,-2,-1,0,1,2,3,4,42] {
        let reference=method.invoke(vec![neoclr::Value::Int32(value)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result) => {
                assert_eq!(r.status.code(),Some(0),"{value}: {r:?}");
                let neoclr::Value::Int32(result)=result.value else { panic!("expected Int32") };
                assert_eq!(r.stdout,format!("{result}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault) => {
                assert_eq!(r.status.code(),Some(1),"{value}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string());
            }
        }
    }
    for (value,message) in [(65536,"NativeMemoryLimitExceeded: Native memory limit exceeded"),(65537,"ArrayLimitExceeded: Array limit exceeded")] {
        let r=Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(1));
        assert!(String::from_utf8_lossy(&r.stderr).starts_with(message));
    }
}

const CHARACTER_SEED: &str = ".function neoCLR.Runtime.CharFromString(String value) -> Char\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.CharText(Char value) -> String\n.methodimpl InternalCall\n.end";

#[test]
fn character_binding_requires_exact_opted_in_services() {
    let seed = neoclr::assemble(&format!("{TEXT_SEED}\n{CHARACTER_SEED}")).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/characters.neoil");
    for flags in [
        vec!["--compile-system", "--bind-user-fault", "--bind-int32-to-string"],
        vec!["--bind-character-text"],
        vec!["--compile-system", "--bind-character-text", "--bind-character-text"],
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    let mut impostor = seed.clone();
    let f = impostor.functions.iter_mut().find(|f| f.name == "neoCLR.Runtime.CharText").unwrap();
    f.impl_flags = 0;
    f.body = vec![neoclr::metadata::Instruction::String("fake".into()), neoclr::metadata::Instruction::Return];
    let dir = Temp::new();
    let r = compile_source(&dir, &impostor, source, &["--compile-system", "--bind-user-fault", "--bind-int32-to-string", "--bind-character-text"], false);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("exact CharFromString"));
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn character_graphemes_defaults_borrows_and_faults_match_interpreter() {
    let dir = Temp::new();
    let seed = neoclr::assemble(&format!("{TEXT_SEED}\n{CHARACTER_SEED}")).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/characters.neoil");
    let r = compile_source(&dir, &seed, source, &["--compile-system", "--bind-user-fault", "--bind-int32-to-string", "--bind-character-text"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let base = root.join("docs/experiments");
    let r = Command::new("cargo").args(["build", "--locked", "--release", "--manifest-path"])
        .arg(root.join("tools/aot-native-text/Cargo.toml")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    fs::write(dir.0.join("host.c"), r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[8];
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,sizeof(storage),0} };
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if ((status!=3 && status!=4) || result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
    return neoclr_aot_render_fault(stderr,&ctx.fault);
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg(root.join("tools/aot-native-text/target/release/libneoclr_aot_native_text.a"))
        .arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for value in [-3,-2,-1,0,1,2,3,4,5,9,10,i32::MIN] {
        let fault = method.invoke(vec![neoclr::Value::Int32(value)],neoclr::Limits::default()).unwrap_err();
        let r = Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        assert!(r.status.success(), "{value}: {r:?}");
        assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(), "{value}");
    }
}

const INTEGER_SEED: &str = ".function neoCLR.Runtime.Int64ToString(Int64 value) -> String\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.UInt64ToString(UInt64 value) -> String\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.IntPtrToInt64(IntPtr value) -> Int64\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.UIntPtrToUInt64(UIntPtr value) -> UInt64\n.methodimpl InternalCall\n.end";

#[test]
fn wide_integer_services_require_exact_opt_in() {
    let seed = neoclr::assemble(&format!("{TEXT_SEED}\n{INTEGER_SEED}")).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/wide-integers.neoil");
    for flags in [
        vec!["--compile-system", "--bind-user-fault"],
        vec!["--bind-integer-text"],
        vec!["--compile-system", "--bind-integer-text", "--bind-integer-text"],
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    for service in ["neoCLR.Runtime.Int64ToString", "neoCLR.Runtime.UInt64ToString"] {
        let mut impostor = seed.clone();
        let f = impostor.functions.iter_mut().find(|f| f.name == service).unwrap();
        f.impl_flags = 0;
        f.body = vec![neoclr::metadata::Instruction::String("fake".into()), neoclr::metadata::Instruction::Return];
        let dir = Temp::new();
        let r = compile_source(&dir, &impostor, source, &["--compile-system", "--bind-user-fault", "--bind-integer-text"], false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("exact reserved"));
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn wide_integer_text_conversions_records_and_lifetime_match_interpreter() {
    let dir = Temp::new();
    let seed = neoclr::assemble(&format!("{TEXT_SEED}\n{INTEGER_SEED}")).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/wide-integers.neoil");
    let r = compile_source(&dir, &seed, source, &["--compile-system", "--bind-user-fault", "--bind-integer-text"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"), r#"
#include "text-arena.h"
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
    uint64_t storage[5]; memset(storage,0xa5,sizeof(storage));
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,27,0} };
    int32_t result=-99;
    if (neoclr_entry_v4(0,&result,&ctx)!=5 || result!=-99 || ctx.text.used) return 90;
    for (unsigned i=0;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 91;
    ctx.text.capacity=28;
    for (int iteration=0;iteration<2;iteration++) {
        int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
        if (status!=4 || result!=-99 || ctx.fault.code!=4 || ctx.text.used>28) return 92;
    }
    for (unsigned i=28;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 93;
    return neoclr_aot_render_fault(stderr,&ctx.fault);
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for value in [0,1,2,3,4,5,6,7,8,9,10,11,-1,i32::MIN,i32::MAX] {
        let fault = method.invoke(vec![neoclr::Value::Int32(value)],neoclr::Limits::default()).unwrap_err();
        let r = Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        assert!(r.status.success(), "{value}: {r:?}");
        assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(), "{value}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn interface_views_preserve_identity_nulls_and_invalid_cast_faults() {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/interface-views.neoil");
    let denied = compile_source(&dir,&seed,source,&["--compile-system"],false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r = compile_source(&dir,&seed,source,&["--compile-system","--reference-arena"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[32];
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,sizeof(storage),0} };
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (status!=3 || result!=-99 || ctx.fault.code!=3) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..8 {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result) => {
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value) = result.value else { panic!("expected Int32") };
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault) => {
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn interface_calls_dispatch_two_implementations_and_preserve_fault_frames() {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/interface-calls.neoil");
    let denied = compile_source(&dir,&seed,source,&["--compile-system"],false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r = compile_source(&dir,&seed,source,&["--compile-system","--reference-arena"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["interfaceDispatch"].as_array().unwrap().len(), 1);
    assert_eq!(report["interfaceDispatch"][0]["targets"].as_array().unwrap().len(), 2);
    assert!(!report["functions"].as_array().unwrap().iter().any(|f| f["name"] == "Unused.Read"));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[32];
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,sizeof(storage),0} };
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if ((status!=4 && status!=6) || result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..4 {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result) => {
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value) = result.value else { panic!("expected Int32") };
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault) => {
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

const STREAM_SEED: &str = ".module System\n.references ()\n";
const STREAM_APP: &str = r#"
.module StreamOutput
.function neoCLR.Runtime.ConsoleWriteBytes(Boolean error, arrayref<Byte> bytes, Int32 offset, Int32 count) -> System.Value
.methodimpl InternalCall
.end
.function neoCLR.Runtime.ConsoleFlush(Boolean error) -> System.Value
.methodimpl InternalCall
.end
.function Calculate(Int32 mode) -> Int32
.local arrayref<Byte> bytes
ldarg mode
ldc.i4 2
beq Flush
ldarg mode
ldc.i4 3
beq Null
ldc.i4 3
newarr Byte
stloc bytes
ldloc bytes
ldc.i4 1
ldc.i4 255
conv.u1
stelem Byte
ldloc bytes
ldc.i4 2
ldc.i4 65
conv.u1
stelem Byte
br Write
Null:
ldloca bytes
initobj arrayref<Byte>
Write:
ldarg mode
ldc.i4 1
ceq
ldloc bytes
ldc.i4 0
ldc.i4 3
call neoCLR.Runtime.ConsoleWriteBytes(Boolean, arrayref<Byte>, Int32, Int32)
br Decode
Flush:
ldc.bool true
call neoCLR.Runtime.ConsoleFlush(Boolean)
Decode:
dup
value.is Byte
brfalse Count
value.unpack Byte
conv.i4
ldc.i4 1000
add
ret
Count:
value.unpack Int32
ret
.end
"#;
#[test]
fn stream_output_services_require_exact_opt_in() {
    for flags in [
        vec!["--compile-system","--reference-arena"],
        vec!["--compile-system","--bind-console-stream-output"],
        vec!["--bind-console-stream-output"],
        vec!["--compile-system","--reference-arena","--bind-console-stream-output","--bind-console-stream-output"],
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir,&neoclr::assemble(STREAM_SEED).unwrap(),STREAM_APP,&flags,false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    for service in 0..2 {
        let mut source=STREAM_APP.to_owned();
        let at=source.match_indices(".methodimpl InternalCall").nth(service).unwrap().0;
        source.replace_range(at..at+".methodimpl InternalCall".len(), "ldc.i4 0\nvalue.pack Int32\nret");
        let dir=Temp::new();
        let r=compile_source(&dir,&neoclr::assemble(STREAM_SEED).unwrap(),&source,&["--compile-system","--reference-arena","--bind-console-stream-output"],false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("exact ConsoleWriteBytes"), "{}", String::from_utf8_lossy(&r.stderr));
    }
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn stream_output_transports_bytes_statuses_and_null_faults() {
    let dir=Temp::new();
    let seed=neoclr::assemble(STREAM_SEED).unwrap();
    let r=compile_source(&dir,&seed,STREAM_APP,&["--compile-system","--reference-arena","--bind-console-stream-output"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include "console.h"
#include <limits.h>
#include <stdlib.h>
static int32_t supplied, channel, calls;
int32_t neoclr_console_write_bytes_v1(int32_t error,const unsigned char *bytes,uint64_t length,int32_t offset,int32_t count) {
    if (error!=channel || length!=3 || offset!=0 || count!=3 || bytes[0]!=0 || bytes[1]!=255 || bytes[2]!=65) abort();
    calls++; return supplied;
}
int32_t neoclr_console_flush_v1(int32_t error) {
    if (error!=1) abort(); calls++; return supplied;
}
int main(void) {
    uint64_t storage[32];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t values[]={INT_MIN,-11,-10,-8,-7,-1,0,1,3,4,INT_MAX};
    for (int mode=0;mode<3;mode++) for (unsigned i=0;i<sizeof(values)/sizeof(values[0]);i++) {
        supplied=values[i];channel=mode;calls=0;int32_t result=-99;
        int expected=mode==2 ? (supplied==0 ? 0 : 1010) :
            supplied>=0 && supplied<=3 ? supplied : supplied==-7 ? 1007 : supplied==-8 ? 1008 : 1010;
        if (neoclr_entry_v4(mode,&result,&ctx)!=0 || ctx.fault.code || result!=expected || calls!=1) return 1;
    }
    calls=0;int32_t result=-99;
    if (neoclr_entry_v4(3,&result,&ctx)!=3 || result!=-99 || ctx.fault.code!=3 || calls!=0) return 2;
    return neoclr_aot_render_fault(stderr,&ctx.fault);
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let r=Command::new(dir.0.join("app")).env_clear().output().unwrap();
    assert!(r.status.success(),"{r:?}");
    let app=neoclr::assemble(STREAM_APP).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let function=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    let fault=function.invoke(vec![neoclr::Value::Int32(3)],neoclr::Limits::default()).unwrap_err();
    assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn linked_stream_adapter_preserves_ranges_limits_and_channels() {
    let dir=Temp::new();
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"),r#"
#include "console.h"
#include <limits.h>
#include <signal.h>
#include <unistd.h>
static unsigned char data[65537];
int main(int argc, char **argv) {
    if (argc>1) {
        int fds[2];
        if (pipe(fds)!=0 || signal(SIGPIPE,SIG_IGN)==SIG_ERR) return 4;
        close(fds[0]);
        int error=argv[1][0]=='e';
        if (dup2(fds[1],error ? 2 : 1)<0) return 5;
        close(fds[1]);
        const unsigned char bytes[]={'A',0,'B','\n'};
        if (argv[1][0]=='f') {
            if (neoclr_console_write_bytes_v1(0,bytes,4,0,3)!=3) return 6;
            return neoclr_console_flush_v1(0)==-10 ? 0 : 7;
        }
        return neoclr_console_write_bytes_v1(error,bytes,4,0,error ? 3 : 4)==-10 ? 0 : 8;
    }
    if (neoclr_console_write_bytes_v1(0,data,3,-1,1)!=-7 ||
        neoclr_console_write_bytes_v1(0,data,3,0,-1)!=-7 ||
        neoclr_console_write_bytes_v1(0,data,3,4,0)!=-7 ||
        neoclr_console_write_bytes_v1(0,data,3,2,2)!=-7 ||
        neoclr_console_write_bytes_v1(0,data,3,INT_MAX,INT_MAX)!=-7 ||
        neoclr_console_write_bytes_v1(0,data,3,0,65537)!=-7 ||
        neoclr_console_write_bytes_v1(0,data,65537,0,65537)!=-8 ||
        neoclr_console_write_bytes_v1(0,data,3,3,0)!=0) return 1;
    if (neoclr_console_write_bytes_v1(0,data,65537,0,65536)!=65536 ||
        neoclr_console_flush_v1(0)!=0) return 2;
    data[0]='A';data[1]=0;data[2]=255;
    if (neoclr_console_write_bytes_v1(1,data,3,0,3)!=3 ||
        neoclr_console_flush_v1(1)!=0) return 3;
    return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(&base).arg(dir.0.join("host.c")).arg(base.join("console.c")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let r=Command::new(dir.0.join("app")).env_clear().output().unwrap();
    assert!(r.status.success());
    assert_eq!(r.stdout,vec![0;65536]);
    assert_eq!(r.stderr,b"A\0\xff");
    for mode in ["line", "error", "flush"] {
        let r=Command::new(dir.0.join("app")).arg(mode).env_clear().output().unwrap();
        assert!(r.status.success(),"{mode}: {r:?}");
        assert!(r.stdout.is_empty() && r.stderr.is_empty());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn closed_generic_classes_and_inherited_interface_views_preserve_shapes() {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/generic-views.neoil");
    let denied = compile_source(&dir,&seed,source,&["--compile-system"],false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r = compile_source(&dir,&seed,source,&["--compile-system","--reference-arena"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[32];
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,sizeof(storage),0} };
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if ((status!=3 && status!=6) || result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..4 {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result) => {
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value) = result.value else { panic!("expected Int32") };
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault) => {
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn reserved_bytes_keep_unwritten_reads_checked_and_preserve_service_ordering() {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/reserved-bytes.neoil");
    let denied = compile_source(&dir,&seed,source,&["--compile-system"],false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r = compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-console-stream-output"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
#include "console.h"
int32_t neoclr_console_write_bytes_v1(int32_t error, const unsigned char *bytes, uint64_t length, int32_t offset, int32_t count) {
    if (error) abort();
    if (offset<0 || count<0 || (uint64_t)offset>length || (uint64_t)count>length-(uint64_t)offset) return -7;
    if (count && bytes[offset]!=42) abort();
    return count;
}
int main(int argc, char **argv) {
    uint64_t storage[64];
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,sizeof(storage),0} };
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("app")).arg("9").env_clear().output().unwrap();
    assert_eq!(r.status.code(),Some(1));
    assert!(String::from_utf8_lossy(&r.stderr).starts_with("NativeMemoryLimitExceeded:"));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in (0..15).filter(|m| *m!=9) {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result) => {
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value) = result.value else { panic!("expected Int32") };
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault) => {
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
fn reserved_array_borrows_are_rejected_before_emission() {
    let source=include_str!("../../../docs/experiments/aot-console/reserved-bytes.neoil")
        .replacen("ldelem Byte", "ldelema Byte\nldobj Byte", 1);
    let dir=Temp::new();
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let r=compile_source(&dir,&seed,&source,&["--compile-system","--reference-arena","--bind-console-stream-output"],false);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("initialization-aware addresses"));
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn thirty_two_lane_reference_allocation_respects_arena_canaries() {
    let dir=Temp::new();
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let fields=(0..16).map(|i| format!(".field F{i} Pair\n")).collect::<String>();
    let source=format!(".module WideObject\n.type Pair\n.field X Int32\n.field Y Int32\n.end\n.type class Wide\n{fields}.method instance .ctor() -> noresult\nldarg 0\nldc.i4 0\nldc.i4 42\nnewobj Pair\nstfld 15\nret\n.end\n.end\n.function Calculate() -> Int32\n.local Wide value\nnewobj.ctor instance Wide::.ctor()\nstloc value\nldloc value\nldfld 0\nldfld 0\nldloc value\nldfld 15\nldfld 1\nadd\nret\n.end");
    let r=compile_source(&dir,&seed,&source,&["--compile-system","--reference-arena"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <string.h>
int main(void) {
    uint64_t storage[34];memset(storage,0xa5,sizeof(storage));
    neoclr_aot_context ctx={.text={(unsigned char*)storage,264,0}};
    int32_t result=-99;
    if (neoclr_entry_v4(0,&result,&ctx) || result!=42 || ctx.text.used!=264) return 1;
    for (unsigned i=264;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 2;
    ctx.text.capacity=263;result=-99;
    if (neoclr_entry_v4(0,&result,&ctx)!=5 || result!=-99 || ctx.text.used!=0) return 3;
    return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(&base).arg(dir.0.join("host.c")).arg(base.join("text-arena.c")).arg(dir.0.join("app.o"))
        .arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    assert!(Command::new(dir.0.join("app")).env_clear().status().unwrap().success());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn erased_text_preserves_pointer_payloads_nulls_lifetime_and_faults() {
    let dir = Temp::new();
    let seed = neoclr::assemble(TEXT_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/erased-text.neoil");
    let denied = compile_source(&dir,&seed,source,&["--compile-system"],false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r = compile_source(&dir,&seed,source,&["--compile-system","--bind-user-fault","--bind-int32-to-string"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[32];
    neoclr_aot_context ctx={ .text={(unsigned char*)storage,sizeof(storage),0} };
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if ((status!=3 && status!=4) || result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..6 {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result) => {
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value) = result.value else { panic!("expected Int32") };
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault) => {
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

const UTF8_SEED: &str = r#"
.module System
.references ()
.function neoCLR.Runtime.Utf8Encode(String) -> Byte[]
.methodimpl InternalCall
.end
.function neoCLR.Runtime.StringByteCount(String) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.StringSliceUtf8(String, Int32, Int32) -> Value
.methodimpl InternalCall
.end
.function neoCLR.Runtime.Fault(String) -> Void
.methodimpl InternalCall
.end
"#;

#[test]
fn utf8_text_services_require_exact_opt_in() {
    let source=include_str!("../../../docs/experiments/aot-console/utf8-text.neoil");
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    for flags in [vec!["--compile-system","--reference-arena","--bind-user-fault"],
        vec!["--bind-utf8-text"],vec!["--compile-system","--bind-utf8-text"],
        vec!["--compile-system","--reference-arena","--bind-utf8-text","--bind-utf8-text"]] {
        let dir=Temp::new();
        let r=compile_source(&dir,&seed,source,&flags,false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(),"{r:?}");
    }
    let mut impostor=seed.clone();
    impostor.functions[1].impl_flags=0;
    impostor.functions[1].body=vec![neoclr::metadata::Instruction::Int(0),neoclr::metadata::Instruction::Return];
    let dir=Temp::new();
    let r=compile_source(&dir,&impostor,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("exact StringByteCount"),"{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn utf8_text_slices_preserve_bytes_boundaries_and_faults() {
    let dir=Temp::new();
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/utf8-text.neoil");
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[32];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in -3..132 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn utf8_text_arena_bounds_and_failure_publication() {
    let dir=Temp::new();
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <string.h>
int main(void) {
    const struct { uint64_t length; unsigned char bytes[4]; } input={4,{0xf0,0x9f,0x98,0x80}};
    const neoclr_aot_text *text=(const neoclr_aot_text *)&input;
    uint64_t storage[4], result[2]={99,99};memset(storage,0xa5,sizeof(storage));
    neoclr_aot_text_arena arena={(unsigned char *)storage,12,0};
    if (neoclr_string_slice_utf8_v1(text,0,4,&arena,result) || (uint32_t)result[0]!=4 || arena.used!=12) return 1;
    const neoclr_aot_text *slice=(const neoclr_aot_text *)(uintptr_t)result[1];
    if (slice->length!=4 || memcmp(slice->bytes,input.bytes,4)) return 2;
    for (unsigned i=12;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 3;
    uint64_t saved[2];memcpy(saved,result,sizeof(result));
    if (neoclr_string_slice_utf8_v1(text,0,0,&arena,result)!=5 || memcmp(saved,result,sizeof(result)) || arena.used!=12) return 4;
    arena.used=0;arena.capacity=11;
    if (neoclr_string_slice_utf8_v1(text,0,4,&arena,result)!=5 || arena.used || memcmp(saved,result,sizeof(result))) return 5;
    arena.data=0;arena.capacity=0;
    if (neoclr_string_slice_utf8_v1(text,1,INT32_MAX,&arena,result) || (uint32_t)result[0]!=2 || result[1]!=1 || arena.used) return 6;
    if (neoclr_string_slice_utf8_v1(text,1,0,&arena,result) || result[1]!=2 || arena.used) return 7;
    memcpy(saved,result,sizeof(result));
    if (neoclr_string_slice_utf8_v1(0,0,0,&arena,result)!=3 || memcmp(saved,result,sizeof(result))) return 8;
    int32_t count=-99;
    const struct { uint64_t length; } huge={(uint64_t)INT32_MAX+1};
    if (neoclr_string_byte_count_v1((const neoclr_aot_text *)&huge,&count)!=3 || count!=-99) return 9;
    if (neoclr_string_byte_count_v1(text,&count) || count!=4) return 10;
    memset(storage,0xa5,sizeof(storage));arena.data=(unsigned char *)storage;arena.capacity=20;arena.used=0;
    const void *bytes=0;
    if (neoclr_utf8_encode_v1(text,&arena,&bytes) || arena.used!=20) return 11;
    uint64_t kind,length;memcpy(&kind,bytes,8);memcpy(&length,(const unsigned char *)bytes+8,8);
    if (kind!=UINT64_C(0x80000003) || length!=4 || memcmp((const unsigned char *)bytes+16,input.bytes,4)) return 12;
    for (unsigned i=20;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 13;
    const void *saved_bytes=bytes;arena.used=0;arena.capacity=19;
    if (neoclr_utf8_encode_v1(text,&arena,&bytes)!=5 || arena.used || bytes!=saved_bytes) return 14;
    const struct { uint64_t length; } excessive={65537};
    if (neoclr_utf8_encode_v1((const neoclr_aot_text *)&excessive,&arena,&bytes)!=7 || arena.used || bytes!=saved_bytes) return 15;
    if (neoclr_utf8_encode_v1(0,&arena,&bytes)!=3 || arena.used || bytes!=saved_bytes) return 16;
    static const struct { uint64_t length; unsigned char bytes[65536]; } largest={65536,{0}};
    static uint64_t large_storage[8195];
    arena.data=(unsigned char *)large_storage;arena.capacity=65552;
    large_storage[8194]=UINT64_C(0xa5a5a5a5a5a5a5a5);
    if (neoclr_utf8_encode_v1((const neoclr_aot_text *)&largest,&arena,&bytes) || arena.used!=65552 ||
        memcmp((const unsigned char *)bytes+16,largest.bytes,65536) || large_storage[8194]!=UINT64_C(0xa5a5a5a5a5a5a5a5)) return 17;

    struct { uint64_t kind, length; unsigned char bytes[8]; } encoded={UINT64_C(0x80000001),4,{0xf0,0x9f,0x98,0x80}};
    memset(storage,0xa5,sizeof(storage));arena.data=(unsigned char*)storage;arena.used=0;arena.capacity=12;
    if (neoclr_utf8_decode_v1(&encoded,&arena,result) || arena.used!=12 || (uint32_t)result[0]!=4) return 18;
    slice=(const neoclr_aot_text *)(uintptr_t)result[1];
    if (slice->length!=4 || memcmp(slice->bytes,input.bytes,4)) return 19;
    for (unsigned i=12;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 20;
    memcpy(saved,result,sizeof(result));arena.used=0;arena.capacity=11;
    if (neoclr_utf8_decode_v1(&encoded,&arena,result)!=5 || arena.used || memcmp(saved,result,sizeof(result))) return 21;
    if (neoclr_utf8_decode_v1(0,&arena,result)!=3 || arena.used || memcmp(saved,result,sizeof(result))) return 22;
    encoded.bytes[0]=0xff;arena.capacity=0;
    if (neoclr_utf8_decode_v1(&encoded,&arena,result) || arena.used || (uint32_t)result[0]!=2 || result[1]!=1) return 23;
    encoded.kind=UINT64_C(0x80000002);memcpy(saved,result,sizeof(result));
    if (neoclr_utf8_decode_v1(&encoded,&arena,result)!=3 || arena.used || memcmp(saved,result,sizeof(result))) return 24;
    memset(encoded.bytes+4,1,4);
    if (neoclr_utf8_decode_v1(&encoded,&arena,result) || arena.used || (uint32_t)result[0]!=2) return 25;
    encoded.length=0;arena.capacity=8;
    if (neoclr_utf8_decode_v1(&encoded,&arena,result) || arena.used!=8 || (uint32_t)result[0]!=4) return 26;
    slice=(const neoclr_aot_text *)(uintptr_t)result[1];if (slice->length) return 27;
    return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(&base).arg(dir.0.join("host.c")).arg(base.join("text-arena.c"))
        .arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    assert!(Command::new(dir.0.join("app")).env_clear().status().unwrap().success());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn utf8_encoding_snapshots_and_managed_copy_match_interpreter() {
    let dir=Temp::new();
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/utf8-encode.neoil");
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[32];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in -6..11 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}; interpreter: {}",fault.diagnostic());
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
fn utf8_value_snapshots_reject_mutation_and_default_initialization() {
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let original=include_str!("../../../docs/experiments/aot-console/utf8-encode.neoil");
    for operations in ["ldloca original\nldc.i4 0\nldc.i4 42\nstelem Byte",
        "ldloca original\nldc.i4 0\nldelema Byte\npop", "ldloca original\ninitobj Byte[]"] {
        let source=original.replacen("stloc original",&format!("stloc original\n{operations}"),1);
        let dir=Temp::new();
        let r=compile_source(&dir,&seed,&source,&["--compile-system","--reference-arena","--bind-utf8-text"],false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(),"{operations}: {r:?}");
        if !operations.contains("initobj") { assert!(!String::from_utf8_lossy(&r.stderr).contains("Fault:"),"source must verify before native rejection: {r:?}"); }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_instance_projection_preserves_receiver_arguments_and_null_faults() {
    let dir=Temp::new();
    let mut seed=neoclr::library::system().unwrap().clone();
    let template=neoclr::assemble(".module Template\n.type class Owner\n.method instance Answer(Int32 answer) -> Int32\nldarg answer\nret\n.end\n.end").unwrap();
    let mut method=template.functions[0].clone();
    method.name="System.String.AotAnswer".into();
    method.owner=Some(neoclr::metadata::Type::String);
    method.definition=None;
    seed.functions.push(method.clone());
    method.name="System.String.AotCount".into();method.parameters.clear();method.parameter_names.clear();
    method.body=vec![neoclr::metadata::Instruction::Arg(0),
        neoclr::metadata::Instruction::Call(neoclr::assembler::parse_function_ref("neoCLR.Runtime.StringByteCount(String)").unwrap()),neoclr::metadata::Instruction::Return];
    seed.functions.push(method);
    let source=r#"
.module StringMember
.function Calculate(Int32 mode) -> Int32
.local String text
ldarg mode
ldc.i4 0
ceq
brfalse Null
ldstr "receiver"
stloc text
br Invoke
Null:
ldloca text
initobj String
Invoke:
ldloc text
ldarg mode
ldc.i4 2
ceq
brtrue Direct
call instance System.String::AotCount()
ret
Direct:
ldc.i4 42
call instance System.String::AotAnswer(Int32)
ret
.end
"#;
    let modules=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap();
    let app=&modules[0];
    fs::write(dir.0.join("app.neox"),neoclr::metadata_container::write_module(app).unwrap()).unwrap();
    fs::write(dir.0.join("seed.neox"),neoclr::metadata_container::write_module(&seed).unwrap()).unwrap();
    fs::write(dir.0.join("empty.neoil"),".module Empty\n.references ()\n").unwrap();
    let r=Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc")).arg("--closed-world")
        .arg(dir.0.join("app.neox")).arg("Calculate").arg(dir.0.join("app.o"))
        .arg("--system").arg(dir.0.join("seed.neox")).arg("--module").arg(dir.0.join("empty.neoil"))
        .args(["--compile-system","--reference-arena","--bind-utf8-text"]).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["stringInstanceProjections"].as_array().unwrap().len(),2);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[16];neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;int status=neoclr_entry_v4(argc>1?atoi(argv[1]):0,&result,&ctx);
    if (status) { if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1; }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c")).arg(dir.0.join("app.o"))
        .arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let program=neoclr::LoadedProgram::with_library(app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..3 {
        let expected=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match expected {
            Ok(_)=>{assert_eq!(r.status.code(),Some(0));assert_eq!(r.stdout,if mode==0 {b"8\n".as_slice()} else {b"42\n".as_slice()});assert!(r.stderr.is_empty());}
            Err(fault)=>{assert_eq!(r.status.code(),Some(1));assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string());}
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn boxed_empty_records_keep_identity_root_views_and_fault_sites() {
    let dir=Temp::new();
    let seed=neoclr::assemble(".module System\n.references ()\n.type class abstract System.Object\n.end\n").unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/boxed-empty.neoil");
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap().remove(0);
    let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena"]);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["emptyRecordBoxes"].as_array().unwrap().len(),1);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[3]={0,0,UINT64_C(0xa5a5a5a5a5a5a5a5)};
    int mode=argc>1?atoi(argv[1]):0;
    neoclr_aot_context ctx={.text={(unsigned char*)storage,mode==9?7:16,0}};
    int32_t result=-99;int status=neoclr_entry_v4(mode,&result,&ctx);
    if(storage[2]!=UINT64_C(0xa5a5a5a5a5a5a5a5))return 93;
    if(status){if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1;}
    if(ctx.text.used!=(mode==0?16:8))return 94;
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..4 {
        let expected=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match expected {
            Ok(_)=>{assert_eq!(r.status.code(),Some(0),"{r:?}");assert_eq!(r.stdout,b"1\n");assert!(r.stderr.is_empty());}
            Err(fault)=>{assert_eq!(r.status.code(),Some(1));assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string());}
        }
    }
    let r=Command::new(dir.0.join("app")).arg("9").env_clear().output().unwrap();
    assert_eq!(r.status.code(),Some(1));
    assert_eq!(String::from_utf8_lossy(&r.stderr),"NativeMemoryLimitExceeded: Native memory limit exceeded\n   at Box [instruction 3]\n   at Calculate [instruction 0]\n");
}

fn compile_linked_module(dir:&Temp, seed:&neoclr::Module, app:&neoclr::Module, flags:&[&str]) -> std::process::Output {
    fs::write(dir.0.join("app.neox"),neoclr::metadata_container::write_module(app).unwrap()).unwrap();
    fs::write(dir.0.join("seed.neox"),neoclr::metadata_container::write_module(seed).unwrap()).unwrap();
    fs::write(dir.0.join("empty.neoil"),".module Empty\n.references ()\n").unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc")).arg("--closed-world")
        .arg(dir.0.join("app.neox")).arg("Calculate").arg(dir.0.join("app.o"))
        .arg("--system").arg(dir.0.join("seed.neox")).arg("--module").arg(dir.0.join("empty.neoil"))
        .args(flags).output().unwrap()
}

#[test]
fn boxed_empty_profile_rejects_other_shapes_and_counts_generated_helpers() {
    let seed=neoclr::assemble(".module System\n.references ()\n.type class abstract System.Object\n.end\n").unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/boxed-empty.neoil");
    for (source,flags,message) in [
        (source.to_string(),vec!["--compile-system"],"requires --reference-arena"),
        (source.replace(".type Marker<T>",".type Marker<T>\n.field Value T"),vec!["--compile-system","--reference-arena"],"requires empty value records"),
    ] {
        let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&source)],&seed).unwrap().remove(0);
        let dir=Temp::new();let r=compile_linked_module(&dir,&seed,&app,&flags);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains(message),"{r:?}");
    }
    let calls=(0..1022).map(|n| format!("call F{n}()\npop\n")).collect::<String>();
    let mut large=source.replacen("call Box()",&format!("{calls}call Box()"),1);
    for n in 0..1022 {large.push_str(&format!(".function F{n}() -> Int32\nldc.i4 0\nret\n.end\n"));}
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&large)],&seed).unwrap().remove(0);
    let dir=Temp::new();let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena"]);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("boxing helpers exceed"),"{r:?}");
}

fn array_views_module() -> neoclr::Module {
    array_views_source(include_str!("../../../docs/experiments/aot-console/array-views.neoil"))
}

fn array_views_source(source: &str) -> neoclr::Module {
    let mut app=neoclr::assemble(source).unwrap();
    let index=app.types.iter().position(|t|t.name=="Storage").unwrap();
    let id=neoclr::metadata::TypeDefId{module:app.name.clone(),revision:app.revision.clone(),index:index as u32};
    app.types[index].definition=Some(id.clone());
    app.types[index].origin=Some(serde_json::from_value(serde_json::json!({
        "assembly":"ArrayViews","module":"ArrayViews.neox","name":"Storage`1","token":33554433,
        "publicly_visible":true,"field_tokens":[67108865],"field_access":["Private"],"field_readonly":[false]
    })).unwrap());
    app.assemblies=vec![serde_json::from_value(serde_json::json!({
        "name":"ArrayViews","full_name":"ArrayViews","modules":["ArrayViews.neox"],"references":[],"array_backing":id
    })).unwrap()];
    app
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn nominal_byte_array_views_preserve_aliases_dispatch_and_initialization() {
    check_array_views(array_views_module(), false);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn nominal_byte_array_views_keep_exact_element_identity_across_specialization_order() {
    let source = include_str!("../../../docs/experiments/aot-console/array-views.neoil");
    for first in [true, false] {
        let locals = if first {
            ".local Storage<Int32> other\n.local arrayref<Byte> bytes"
        } else {
            ".local arrayref<Byte> bytes\n.local Storage<Int32> other"
        };
        let source = source.replace(".local arrayref<Byte> bytes", locals)
            .replace("isinst Other", "isinst Read<Int32>")
            .replace("castclass Other", "castclass Read<Int32>");
        check_array_views(array_views_source(&source), false);
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn nominal_reference_array_views_preserve_exact_dispatch_aliases_and_faults() {
    let source = include_str!("../../../docs/experiments/aot-console/reference-array-views.neoil");
    check_array_views(array_views_source(source), true);
}

#[test]
fn nominal_reference_array_views_reject_storage_borrows_and_replacement() {
    let source = include_str!("../../../docs/experiments/aot-console/reference-array-views.neoil");
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    for (body, message) in [
        ("ldarg 0\nldflda 0\npop\nldc.i4 0\nret", "storage field cannot be borrowed"),
        ("ldarg 0\nldc.i4 0\nnewarr T\nstfld 0\nldc.i4 0\nret", "storage field cannot be replaced"),
    ] {
        let source = source.replace("ldarg 0\nldfld 0\nldlen\nconv.i4\nret", body);
        let app = array_views_source(&source);
        let dir = Temp::new();
        let r = compile_linked_module(&dir, &seed, &app, &["--compile-system", "--reference-arena"]);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(), "{r:?}");
        assert!(String::from_utf8_lossy(&r.stderr).contains(message), "{r:?}");
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn check_array_views(app: neoclr::Module, gc: bool) {
    let dir=Temp::new();
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();program.verify().unwrap();
    let flags = if gc { vec!["--compile-system", "--reference-arena", "--native-gc"] } else { vec!["--compile-system", "--reference-arena"] };
    let r=compile_linked_module(&dir,&seed,&app,&flags);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    if gc { assert_eq!(report["referenceArrayBackingProjections"].as_array().unwrap().len(), 2); }
    else { assert!(report["arrayBackingProjection"]["compiledIndex"].is_number()); }
    assert_eq!(report["interfaceDispatch"].as_array().unwrap().len(), if gc {4} else {3});
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "native-gc.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[512];neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;int status=neoclr_entry_v4(argc>1?atoi(argv[1]):0,&result,&ctx);
#ifdef NEOCLR_NATIVE_GC
    if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 93;
    if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 94;
#endif
    if(status){if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1;}
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let mut command = Command::new("clang");
    if gc { command.args(["-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds"]).arg(base.join("aot-console/native-gc.c")).arg(base.join("aot-console/root-probe.c")); }
    let r=command.args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..if gc {15} else {10} {
        let expected=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match expected {
            Ok(value)=>{let neoclr::Value::Int32(value)=value.value else{panic!("expected Int32")};
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");assert_eq!(r.stdout,format!("{value}\n").as_bytes());assert!(r.stderr.is_empty());}
            Err(fault)=>{assert_eq!(r.status.code(),Some(1), "mode {mode}: native {r:?}; interpreter {fault:?}");assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");}
        }
    }
}

#[test]
fn nominal_byte_array_views_require_verified_backing_and_reject_class_allocation() {
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    for mode in 0..4 {
        let mut app=array_views_module();
        match mode {
            0=>app.assemblies[0].array_backing=None,
            1=>app.types.iter_mut().find(|t|t.name=="Storage").unwrap().origin.as_mut().unwrap().field_access=vec![neoclr::metadata_origin::SourceAccess::Public],
            _=>{
                let root=app.functions.iter_mut().find(|f|f.name=="Calculate").unwrap();
                root.body=vec![neoclr::metadata::Instruction::Construct(neoclr::assembler::parse_function_ref(if mode == 2 { "instance Storage<Byte>::.ctor()" } else { "instance Storage<Int32>::.ctor()" }).unwrap()),
                    neoclr::metadata::Instruction::Pop,neoclr::metadata::Instruction::Int(0),neoclr::metadata::Instruction::Return];
            }
        }
        let dir=Temp::new();let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena"]);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(),"mode {mode}: {r:?}");
        let message=String::from_utf8_lossy(&r.stderr);
        assert!(message.contains(match mode {0=>"verified backing",1=>"invalid nominal array backing",_=>"not class construction"}),"{r:?}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn utf8_decode_matches_strict_interpreter_validation() {
    let dir=Temp::new();
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let mut cases: Vec<Vec<u8>> = (0..=255).map(|b| vec![b]).collect();
    cases.extend([vec![], "aé😀\0z".as_bytes().to_vec(), vec![0xc2,0x80], vec![0xdf,0xbf],
        vec![0xe0,0xa0,0x80], vec![0xed,0x9f,0xbf], vec![0xee,0x80,0x80],
        vec![0xf0,0x90,0x80,0x80], vec![0xf4,0x8f,0xbf,0xbf],
        vec![0xc0,0x80], vec![0xe0,0x9f,0xbf], vec![0xed,0xa0,0x80],
        vec![0xf0,0x8f,0xbf,0xbf], vec![0xf4,0x90,0x80,0x80],
        vec![0xc2], vec![0xe1,0x80], vec![0xf1,0x80,0x80], vec![0xe1,0x41,0x80],
        vec![0xe1,0x80,0x41], vec![0xf1,0x80,0x80,0x41]]);
    let mut source=String::from(".module Decode\n.function Calculate(Int32) -> Int32\n.local arrayref<Byte> bytes\n");
    for n in 0..cases.len() { source+=&format!("ldarg 0\nldc.i4 {n}\nceq\nbrtrue Case{n}\n"); }
    source+="ldarg 0\nldc.i4 -1\nceq\nbrtrue Null\nldc.i4 2\narray.reserve Byte\nstloc bytes\nldloc bytes\nldc.i4 0\nldc.i4 255\nconv.u1\nstelem Byte\nbr Decode\nNull:\nldloca bytes\ninitobj arrayref<Byte>\nbr Decode\n";
    for (n,bytes) in cases.iter().enumerate() {
        source+=&format!("Case{n}:\nldc.i4 {}\nnewarr Byte\nstloc bytes\n",bytes.len());
        for (i,b) in bytes.iter().enumerate() { source+=&format!("ldloc bytes\nldc.i4 {i}\nldc.i4 {b}\nconv.u1\nstelem Byte\n"); }
        source+="br Decode\n";
    }
    source+="Decode:\nldloc bytes\ncall neoCLR.Runtime.Utf8Decode(arrayref<Byte>)\ndup\nvalue.is Byte\nbrtrue Error\nvalue.unpack String\ncall neoCLR.Runtime.Fault(String)\npop\nldc.i4 0\nret\nError:\nvalue.unpack Byte\nconv.i4\nldc.i4 100\nadd\nret\n.end\n.function neoCLR.Runtime.Utf8Decode(arrayref<Byte>) -> Value\n.methodimpl InternalCall\n.end\n";
    let r=compile_source(&dir,&seed,&source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[64];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(&source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in -2..cases.len() as i32 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
fn utf8_decoder_rejects_unbound_or_noncontract_services() {
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let original=".module DecoderContract\n.function Calculate() -> Int32\nldc.i4 0\nnewarr Byte\ncall neoCLR.Runtime.Utf8Decode(arrayref<Byte>)\npop\nldc.i4 0\nret\n.end\n.function neoCLR.Runtime.Utf8Decode(arrayref<Byte>) -> Value\n.methodimpl InternalCall\n.end\n";
    for mode in 0..3 {
        let source=match mode {
            1=>original.replace(".methodimpl InternalCall", "ldstr \"impostor\"\nvalue.pack String\nret"),
            2=>original.replace("-> Value", "-> Int32"),
            _=>original.into(),
        };
        let mut flags=vec!["--compile-system","--reference-arena"];
        if mode!=0 {flags.push("--bind-utf8-text");}
        let dir=Temp::new();let r=compile_source(&dir,&seed,&source,&flags,false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(),"{mode}: {r:?}");
        if mode!=0 {assert!(String::from_utf8_lossy(&r.stderr).contains(if mode==1 {"exact StringByteCount"} else {"runtime binding return type mismatch"}),"{mode}: {r:?}");}
    }
}

const OBJECT_DISPLAY_SEED: &str = r#"
.module System
.references ()
.type class abstract System.Object
.method instance .ctor() -> noresult
ret
.end
.method instance virtual ToString() -> String
ldstr "default"
ret
.end
.method instance virtual Equals(System.Object other) -> Boolean
ldc.bool false
ret
.end
.method instance virtual GetHashCode() -> Int32
ldc.i4 -1
ret
.end
.end
.function neoCLR.Runtime.Fault(String) -> Void
.methodimpl InternalCall
.end
"#;

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn object_display_dispatch_preserves_overrides_null_and_fault_frames() {
    let seed=neoclr::assemble(OBJECT_DISPLAY_SEED).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/object-display.neoil");
    for declared_base in [false,true] {
    let mut app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap().remove(0);
    app.assemblies=vec![serde_json::from_value(serde_json::json!({
        "name":"ObjectDisplay","full_name":"ObjectDisplay","modules":["ObjectDisplay.neox"],"references":[]
    })).unwrap()];
    app.functions.iter_mut().find(|f|f.name=="A.ToString").unwrap().origin=Some(serde_json::from_value(serde_json::json!({
        "assembly":"ObjectDisplay","module":"ObjectDisplay.neox","name":"SourceDisplayName","token":100663297,"member_access":"Public"
    })).unwrap());
    if declared_base {
        for ty in &mut app.types {ty.base=Some(neoclr::metadata::Type::Named("System.Object".into()));}
        for f in &mut app.functions {
            if f.name.ends_with("..ctor") {
                f.body.splice(0..0,[neoclr::metadata::Instruction::Arg(0),neoclr::metadata::Instruction::Call(neoclr::assembler::parse_function_ref("instance System.Object::.ctor()").unwrap())]);
            }
        }
    }
    let dir=Temp::new();
    let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena","--bind-user-fault"]);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["objectDisplayDispatch"][0]["targets"].as_array().unwrap().len(),3);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[32];neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;int status=neoclr_entry_v4(argc>1?atoi(argv[1]):0,&result,&ctx);
    if (!status || result!=-99) return 92;
    neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..3 {
        let fault=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default()).unwrap_err();
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(1));assert!(r.stdout.is_empty());
        assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
    }
    }
}

#[test]
fn object_display_rejects_fallback_base_calls_and_unverified_overrides() {
    let seed=neoclr::assemble(OBJECT_DISPLAY_SEED).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/object-display.neoil");
    let original=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap().remove(0);
    for mode in 0..9 {
        let mut app=original.clone();
        match mode {
            0=>app.functions.retain(|f| f.name!="B.ToString"),
            1=>{
                let root=app.functions.iter_mut().find(|f|f.name=="Calculate").unwrap();
                for op in &mut root.body {
                    if let neoclr::metadata::Instruction::CallVirtual(target)=op {
                        *op=neoclr::metadata::Instruction::Call(target.clone());
                    }
                }
            }
            2=>app.functions.iter_mut().find(|f|f.name=="B.ToString").unwrap().is_override=false,
            3=>app.functions.iter_mut().find(|f|f.name=="B.ToString").unwrap().visibility=neoclr::metadata::Visibility::Private,
            5=>{
                let root=app.functions.iter_mut().find(|f|f.name=="Calculate").unwrap();
                root.body.splice(0..0,[neoclr::metadata::Instruction::Int(0),neoclr::metadata::Instruction::NewArray(neoclr::metadata::Type::Byte),neoclr::metadata::Instruction::Pop]);
            }
            6=>{
                let mut marker=neoclr::assemble(".module Marker\n.type Marker\n.end").unwrap().types[0].clone();
                marker.definition=None;
                app.types.push(marker);
                let root=app.functions.iter_mut().find(|f|f.name=="Calculate").unwrap();
                let ty=neoclr::metadata::Type::Named("Marker".into());
                root.body.splice(0..0,[neoclr::metadata::Instruction::New(ty.clone()),neoclr::metadata::Instruction::BoxValue(ty),neoclr::metadata::Instruction::Pop]);
            }
            7=>{
                for ty in &mut app.types {ty.base=Some(neoclr::metadata::Type::Named("System.Object".into()));}
            }
            8=>{
                for ty in &mut app.types {ty.base=Some(neoclr::metadata::Type::Named("System.Object".into()));}
                for f in &mut app.functions {
                    if f.name.ends_with("..ctor") {
                        let call=neoclr::metadata::Instruction::Call(neoclr::assembler::parse_function_ref("instance System.Object::.ctor()").unwrap());
                        f.body.splice(0..0,[neoclr::metadata::Instruction::Arg(0),call.clone(),neoclr::metadata::Instruction::Arg(0),call]);
                    }
                }
            }
            _=>(),
        }
        if matches!(mode,5|6) {
            for op in &mut app.functions.iter_mut().find(|f|f.name=="Calculate").unwrap().body {
                if let neoclr::metadata::Instruction::Branch(pc) | neoclr::metadata::Instruction::BranchTrue(pc)=op {*pc+=3;}
            }
        }
        // Removing a function requires fresh local row identities before encoding.
        for f in &mut app.functions {f.definition=None;}
        let dir=Temp::new();let flags=if mode==4 {vec!["--compile-system","--bind-user-fault"]}
            else {vec!["--compile-system","--reference-arena","--bind-user-fault"]};
        let r=compile_linked_module(&dir,&seed,&app,&flags);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(),"{mode}: {r:?}");
        let error=String::from_utf8_lossy(&r.stderr);
        assert!(!error.contains("panicked"),"{mode}: {r:?}");
        if mode==0 {assert!(error.contains("default display"),"{r:?}");}
        if mode==1 {assert!(error.contains("direct Object.ToString"),"{r:?}");}
        if mode==2 {assert!(error.contains("verified concrete override"),"{r:?}");}
        if mode==3 {assert!(error.contains("Fault:"),"source validation must reject before projection: {r:?}");}
        if matches!(mode,5|6) {assert!(error.contains("boxing or arrays"),"{r:?}");}
        if mode==7 {assert!(error.contains("leading base constructor"),"{r:?}");}
        if mode==8 {assert!(error.contains("single leading base initializer"),"{r:?}");}
        if mode==4 {assert!(error.contains("requires --reference-arena"),"{r:?}");}
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_concatenation_preserves_bytes_snapshots_and_faults() {
    let dir=Temp::new();
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/concat.neoil");
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[128];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in -3..8 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn concatenation_arena_bounds_aliases_and_failure_publication() {
    let dir=Temp::new();
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <string.h>
int main(void) {
    const struct {uint64_t length;unsigned char bytes[4];} value={4,{0xc3,0xa9,0,0x7a}};
    const neoclr_aot_text *input=(const neoclr_aot_text*)&value;
    const struct {uint64_t length;} empty={0},huge={UINT64_MAX},one={1};
    uint64_t storage[8];memset(storage,0xa5,sizeof(storage));
    neoclr_aot_text_arena arena={(unsigned char*)storage,16,0};
    const neoclr_aot_text *output=0;
    if(neoclr_string_concat_v1(input,input,&arena,&output) || arena.used!=16 || output->length!=8 ||
       memcmp(output->bytes,value.bytes,4) || memcmp(output->bytes+4,value.bytes,4))return 1;
    for(unsigned i=16;i<sizeof(storage);i++)if(((unsigned char*)storage)[i]!=0xa5)return 2;
    const neoclr_aot_text *saved=output;
    if(neoclr_string_concat_v1(input,input,&arena,&output)!=5 || arena.used!=16 || output!=saved)return 3;
    arena.capacity=48;
    if(neoclr_string_concat_v1(saved,saved,&arena,&output) || arena.used!=40 || output->length!=16 ||
       memcmp(output->bytes,saved->bytes,8) || memcmp(output->bytes+8,saved->bytes,8) || saved->length!=8)return 4;
    saved=output;arena.used=1;arena.capacity=23;
    if(neoclr_string_concat_v1(input,input,&arena,&output)!=5 || arena.used!=1 || output!=saved)return 5;
    arena.capacity=24;
    if(neoclr_string_concat_v1(input,input,&arena,&output) || arena.used!=24 || ((uintptr_t)output&7))return 6;
    saved=output;
    if(neoclr_string_concat_v1(0,input,&arena,&output)!=3 || arena.used!=24 || output!=saved)return 7;
    if(neoclr_string_concat_v1(input,0,&arena,&output)!=3 || arena.used!=24 || output!=saved)return 8;
    if(neoclr_string_concat_v1((const neoclr_aot_text*)&huge,(const neoclr_aot_text*)&one,&arena,&output)!=3 || arena.used!=24 || output!=saved)return 9;
    if(neoclr_string_concat_v1((const neoclr_aot_text*)&huge,(const neoclr_aot_text*)&empty,&arena,&output)!=5 || arena.used!=24 || output!=saved)return 10;
    arena.used=0;arena.capacity=8;
    if(neoclr_string_concat_v1((const neoclr_aot_text*)&empty,(const neoclr_aot_text*)&empty,&arena,&output) || arena.used!=8 || output->length)return 11;
    saved=output;arena.used=9;
    if(neoclr_string_concat_v1(input,input,&arena,&output)!=3 || arena.used!=9 || output!=saved)return 12;
    return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(&base).arg(dir.0.join("host.c")).arg(base.join("text-arena.c"))
        .arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    assert!(Command::new(dir.0.join("app")).env_clear().status().unwrap().success());
}

#[test]
fn concatenation_requires_exact_opt_in_contract() {
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let original=include_str!("../../../docs/experiments/aot-console/concat.neoil");
    for mode in 0..3 {
        let source=if mode==2 {original.replace(".methodimpl InternalCall", "ldstr \"impostor\"\nret")}else{original.into()};
        let mut flags=vec!["--compile-system","--bind-user-fault"];
        if mode!=1 {flags.push("--reference-arena");}
        if mode!=0 {flags.push("--bind-utf8-text");}
        let dir=Temp::new();let r=compile_source(&dir,&seed,&source,&flags,false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(),"{mode}: {r:?}");
        if mode==2 {assert!(String::from_utf8_lossy(&r.stderr).contains("conflicting function signature"),"{r:?}");}
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn boxed_int32_display_preserves_snapshot_identity_and_limits() {
    let seed=neoclr::assemble(&format!("{OBJECT_DISPLAY_SEED}\n.function neoCLR.Runtime.WriteLine(String) -> Void\n.methodimpl InternalCall\n.end\n")).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/boxed-int32.neoil");
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap().remove(0);
    let dir=Temp::new();
    let flags=["--compile-system","--reference-arena","--bind-int32-to-string","--bind-console-write-line"];
    let r=compile_linked_module(&dir,&seed,&app,&flags);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["int32Boxes"].as_array().unwrap().len(),1);
    assert_eq!(report["boxedInt32Display"],true);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[33]={0};storage[32]=UINT64_C(0xa5a5a5a5a5a5a5a5);
    int32_t value=argc>1?(int32_t)strtol(argv[1],NULL,10):0;
    unsigned capacity=argc>2?(unsigned)strtoul(argv[2],NULL,10):256;
    neoclr_aot_context ctx={.text={(unsigned char*)storage,capacity,0}};
    int32_t result=-99;int status=neoclr_entry_v4(value,&result,&ctx);
    if(storage[32]!=UINT64_C(0xa5a5a5a5a5a5a5a5) || ctx.text.used>capacity)return 93;
    if(status){if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1;}
    return result;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-console/console.c")).arg(base.join("aot-scalar/console.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for value in [i32::MIN,-1,0,1,42,i32::MAX] {
        let expected=method.invoke(vec![neoclr::Value::Int32(value)],neoclr::Limits::default()).unwrap();
        assert_eq!(expected.value,neoclr::Value::Int32(0));
        let r=Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(0),"{r:?}");
        assert_eq!(r.stdout,format!("{value}\n{value}\n").as_bytes());
        assert_eq!(r.stdout,expected.stdout);assert!(r.stderr.is_empty());
    }
    for capacity in [15,31,32] {
        let r=Command::new(dir.0.join("app")).args(["42",&capacity.to_string()]).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(1),"{r:?}");assert!(r.stdout.is_empty());
        let diagnostic=String::from_utf8_lossy(&r.stderr);
        assert!(diagnostic.starts_with("NativeMemoryLimitExceeded: Native memory limit exceeded\n"),"{r:?}");
        let frames=match capacity {
            15=>"   at Box [instruction 1]\n   at Calculate [instruction 3]\n",
            31=>"   at Box [instruction 1]\n   at Calculate [instruction 8]\n",
            _=>"   at Calculate [instruction 19]\n",
        };
        assert_eq!(diagnostic,format!("NativeMemoryLimitExceeded: Native memory limit exceeded\n{frames}"));
    }
    let reject=Temp::new();
    let r=compile_linked_module(&reject,&seed,&app,&["--compile-system","--reference-arena","--bind-console-write-line"]);
    assert!(!r.status.success() && !reject.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("boxed Int32 display requires --bind-int32-to-string"),"{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_object_views_preserve_identity_casts_nulls_and_faults() {
    let seed=neoclr::assemble(&format!("{OBJECT_DISPLAY_SEED}\n.function neoCLR.Runtime.WriteLine(String) -> Void\n.methodimpl InternalCall\n.end\n")).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/string-object-views.neoil");
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap().remove(0);
    let dir=Temp::new();
    let flags=["--compile-system","--reference-arena","--bind-int32-to-string","--bind-console-write-line"];
    let r=compile_linked_module(&dir,&seed,&app,&flags);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["int32Boxes"].as_array().unwrap().len(),1);
    assert_eq!(report["boxedInt32Display"],true);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[33]={0};storage[32]=UINT64_C(0xa5a5a5a5a5a5a5a5);
    int32_t value=argc>1?(int32_t)strtol(argv[1],NULL,10):0;
    unsigned capacity=argc>2?(unsigned)strtoul(argv[2],NULL,10):256;
    neoclr_aot_context ctx={.text={(unsigned char*)storage,capacity,0}};
    int32_t result=-99;int status=neoclr_entry_v4(value,&result,&ctx);
    if(storage[32]!=UINT64_C(0xa5a5a5a5a5a5a5a5) || ctx.text.used>capacity)return 93;
    if(status){if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1;}
    return result;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-console/console.c")).arg(base.join("aot-scalar/console.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for value in [0] {
        let expected=method.invoke(vec![neoclr::Value::Int32(value)],neoclr::Limits::default()).unwrap();
        assert_eq!(expected.value,neoclr::Value::Int32(0));
        let r=Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(0),"{r:?}");
        assert_eq!(r.stdout,"hé😀\0z\n".as_bytes());
        assert_eq!(r.stdout,expected.stdout);assert!(r.stderr.is_empty());
    }
    for mode in [1,2,3] {
        let expected=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default()).unwrap_err();
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(1),"{r:?}");assert!(r.stdout.is_empty());
        assert_eq!(String::from_utf8_lossy(&r.stderr),expected.diagnostic().to_string());
    }
    let r=Command::new(dir.0.join("app")).args(["0","7"]).env_clear().output().unwrap();
    assert_eq!(r.status.code(),Some(1),"{r:?}");assert!(r.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&r.stderr),"NativeMemoryLimitExceeded: Native memory limit exceeded\n   at Fresh [instruction 0]\n   at Calculate [instruction 0]\n");
    let reject=Temp::new();
    let r=compile_linked_module(&reject,&seed,&app,&["--compile-system","--reference-arena","--bind-console-write-line"]);
    assert!(!r.status.success() && !reject.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("boxed Int32 display requires --bind-int32-to-string"),"{r:?}");
}


#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn character_text_preserves_fresh_string_identity_and_faults() {
    let seed=neoclr::assemble(&format!("{OBJECT_DISPLAY_SEED}\n{CHARACTER_SEED}\n.function neoCLR.Runtime.WriteLine(String) -> Void\n.methodimpl InternalCall\n.end\n")).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/character-text-identity.neoil");
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap().remove(0);
    let dir=Temp::new();
    let flags=["--compile-system","--reference-arena","--bind-int32-to-string","--bind-console-write-line","--bind-character-text"];
    let r=compile_linked_module(&dir,&seed,&app,&flags);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["int32Boxes"].as_array().unwrap().len(),1);
    assert_eq!(report["boxedInt32Display"],true);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[33]={0};storage[32]=UINT64_C(0xa5a5a5a5a5a5a5a5);
    int32_t value=argc>1?(int32_t)strtol(argv[1],NULL,10):0;
    unsigned capacity=argc>2?(unsigned)strtoul(argv[2],NULL,10):256;
    neoclr_aot_context ctx={.text={(unsigned char*)storage,capacity,0}};
    int32_t result=-99;int status=neoclr_entry_v4(value,&result,&ctx);
    if(storage[32]!=UINT64_C(0xa5a5a5a5a5a5a5a5) || ctx.text.used>capacity)return 93;
    if(status){if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1;}
    return result;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-console/console.c")).arg(base.join("aot-scalar/console.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for value in [0] {
        let expected=method.invoke(vec![neoclr::Value::Int32(value)],neoclr::Limits::default()).unwrap();
        assert_eq!(expected.value,neoclr::Value::Int32(0));
        let r=Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(0),"{r:?}");
        assert_eq!(r.stdout,"\0\n".as_bytes());
        assert_eq!(r.stdout,expected.stdout);assert!(r.stderr.is_empty());
    }
    for mode in [1,2,3] {
        let expected=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default()).unwrap_err();
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(1),"{r:?}");assert!(r.stdout.is_empty());
        assert_eq!(String::from_utf8_lossy(&r.stderr),expected.diagnostic().to_string());
    }
    let r=Command::new(dir.0.join("app")).args(["0","7"]).env_clear().output().unwrap();
    assert_eq!(r.status.code(),Some(1),"{r:?}");assert!(r.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&r.stderr),"NativeMemoryLimitExceeded: Native memory limit exceeded\n   at Fresh [instruction 3]\n   at Calculate [instruction 0]\n");
    let reject=Temp::new();
    let r=compile_linked_module(&reject,&seed,&app,&["--compile-system","--reference-arena","--bind-console-write-line"]);
    assert!(!r.status.success() && !reject.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("boxed Int32 display requires --bind-int32-to-string"),"{r:?}");
}

const STRING_VIEW_SEED: &str = r#"
.interface Root<T>
.end
.interface View<T>
.implements Root<T>
.end
.type System.String
.implements View<String>
.end
"#;

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_interface_views_preserve_closed_conformance_identity_and_faults() {
    let seed=neoclr::assemble(&format!("{OBJECT_DISPLAY_SEED}\n{STRING_VIEW_SEED}\n.function neoCLR.Runtime.WriteLine(String) -> Void\n.methodimpl InternalCall\n.end\n")).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/string-interface-views.neoil");
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)],&seed).unwrap().remove(0);
    let dir=Temp::new();
    let flags=["--compile-system","--reference-arena","--bind-int32-to-string","--bind-console-write-line"];
    let r=compile_linked_module(&dir,&seed,&app,&flags);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["int32Boxes"].as_array().unwrap().len(),1);
    assert_eq!(report["boxedInt32Display"],true);
    assert_eq!(report["stringInterfaceViews"].as_array().unwrap().len(),2);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[33]={0};storage[32]=UINT64_C(0xa5a5a5a5a5a5a5a5);
    int32_t value=argc>1?(int32_t)strtol(argv[1],NULL,10):0;
    unsigned capacity=argc>2?(unsigned)strtoul(argv[2],NULL,10):256;
    neoclr_aot_context ctx={.text={(unsigned char*)storage,capacity,0}};
    int32_t result=-99;int status=neoclr_entry_v4(value,&result,&ctx);
    if(storage[32]!=UINT64_C(0xa5a5a5a5a5a5a5a5) || ctx.text.used>capacity)return 93;
    if(status){if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1;}
    return result;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-console/console.c")).arg(base.join("aot-scalar/console.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for value in [0] {
        let expected=method.invoke(vec![neoclr::Value::Int32(value)],neoclr::Limits::default()).unwrap();
        assert_eq!(expected.value,neoclr::Value::Int32(0));
        let r=Command::new(dir.0.join("app")).arg(value.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(0),"{r:?}");
        assert_eq!(r.stdout,"hé😀\0z\n".as_bytes());
        assert_eq!(r.stdout,expected.stdout);assert!(r.stderr.is_empty());
    }
    for mode in [1,2,3] {
        let expected=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default()).unwrap_err();
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(),Some(1),"{r:?}");assert!(r.stdout.is_empty());
        assert_eq!(String::from_utf8_lossy(&r.stderr),expected.diagnostic().to_string());
    }
    let r=Command::new(dir.0.join("app")).args(["0","7"]).env_clear().output().unwrap();
    assert_eq!(r.status.code(),Some(1),"{r:?}");assert!(r.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&r.stderr),"NativeMemoryLimitExceeded: Native memory limit exceeded\n   at Fresh [instruction 0]\n   at Calculate [instruction 0]\n");
    let reject=Temp::new();
    let r=compile_linked_module(&reject,&seed,&app,&["--compile-system","--reference-arena","--bind-console-write-line"]);
    assert!(!r.status.success() && !reject.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("boxed Int32 display requires --bind-int32-to-string"),"{r:?}");
}


#[test]
fn string_interface_byref_receivers_reject_before_native_publication() {
    let views=STRING_VIEW_SEED.replace(".interface Root<T>\n.end", ".interface Root<T>\n.method instance Get() -> T\n.end\n.end")
        .replace(".implements View<String>\n.end", ".implements View<String>\n.method instance readonly byref Get() -> String\nldarg 0\nldobj String\nret\n.end\n.end");
    let seed=neoclr::assemble(&format!("{OBJECT_DISPLAY_SEED}\n{views}\n.function neoCLR.Runtime.WriteLine(String) -> Void\n.methodimpl InternalCall\n.end\n")).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/string-interface-views.neoil")
        .replace("ldstr \"hé😀\\u0000z\"\nret", "ldstr \"hé😀\\u0000z\"\ncastclass Root<String>\ncallvirt instance Root<String>::Get()\nret");
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&source)],&seed).unwrap().remove(0);
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    program.verify().unwrap();
    let dir=Temp::new();
    let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena","--bind-int32-to-string","--bind-console-write-line"]);
    assert!(!r.status.success() && !dir.0.join("app.o").exists(),"{r:?}");
    assert!(String::from_utf8_lossy(&r.stderr).contains("String instance projection requires public or explicit-interface nonvirtual wrappers"),"{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_equality_matches_contents_nulls_and_dynamic_text() {
    let dir=Temp::new();
    let seed=neoclr::assemble(UTF8_SEED).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/string-equality.neoil");
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[128];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    if(ctx.text.used != ((argc>1 && atoi(argv[1])==9)?15:0))return 93;
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..12 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_interface_dispatch_calls_verified_bodies_and_preserves_faults() {
    for native_gc in [false, true] {
        check_string_interface_dispatch(native_gc);
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn check_string_interface_dispatch(native_gc: bool) {
    let views = STRING_VIEW_SEED
        .replace(".interface Root<T>\n.end", ".interface Root<T>\n.method instance Equals(T other) -> Boolean\n.end\n.end")
        .replace(".implements View<String>\n.end", r#".implements View<String>
.method instance Equals(String other) -> Boolean
ldarg 0
ldstr "fault"
ceq
brfalse Compare
ldstr "from String.Equals"
call neoCLR.Runtime.Fault(String)
pop
Compare:
ldarg 0
ldarg other
ceq
ret
.end
.end"#);
    let seed = neoclr::assemble(&format!("{OBJECT_DISPLAY_SEED}\n{views}")).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/string-interface-dispatch.neoil");
    let app = neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], &seed).unwrap().remove(0);
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    program.verify().unwrap();
    let dir = Temp::new();
    let r = compile_linked_module(&dir, &seed, &app, &["--compile-system", "--reference-arena", "--bind-user-fault", if native_gc { "--native-gc" } else { "--probe-stack-roots" }]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["stringInterfaceDispatch"].as_array().unwrap().len(), 1);
    assert_eq!(report["interfaceDispatch"][0]["targets"].as_array().unwrap().len(), 1);
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"), r#"
#include "text-arena.h"
#include "root-probe.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[128];
    neoclr_aot_context ctx = {.text = {(unsigned char*)storage, sizeof(storage), 0}};
    int32_t result = -99;
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 93;
    if (status) {
        if (result != -99 || ctx.fault.code != (uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr, &ctx.fault);
        return 1;
    }
    return result;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(if native_gc { vec!["-DNEOCLR_NATIVE_GC"] } else { vec![] })
        .args(if native_gc { vec![base.join("aot-console/native-gc.c")] } else { vec![] })
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/root-probe.c")).arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..6 {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(value) => {
                assert_eq!(value.value, neoclr::Value::Int32(0));
                assert_eq!(r.status.code(), Some(0), "{mode}: {r:?}");
                assert!(r.stdout.is_empty() && r.stderr.is_empty());
            }
            Err(fault) => {
                assert!(mode == 3 || mode == 5);
                assert_eq!(r.status.code(), Some(1), "{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr), fault.diagnostic().to_string(), "{mode}");
            }
        }
    }
    // The implicit target contributes a real call-graph edge: it cannot hide
    // recursion from the bounded profile's stack-budget admission check.
    let mut recursive = seed.clone();
    recursive.functions.iter_mut().find(|f| f.name == "System.String.Equals").unwrap().body = vec![
        neoclr::metadata::Instruction::Arg(0),
        neoclr::metadata::Instruction::CastClass(neoclr::metadata::Type::Constructed {
            definition: "Root".into(), arguments: vec![neoclr::metadata::Type::String],
        }),
        neoclr::metadata::Instruction::Arg(1),
        neoclr::metadata::Instruction::CallVirtual(neoclr::assembler::parse_function_ref("instance Root<String>::Equals(String)").unwrap()),
        neoclr::metadata::Instruction::Return,
    ];
    let reject = Temp::new();
    let r = compile_linked_module(&reject, &recursive, &app, &["--compile-system", "--reference-arena", "--bind-user-fault", if native_gc { "--native-gc" } else { "--probe-stack-roots" }]);
    assert!(!r.status.success() && !reject.0.join("app.o").exists(), "{r:?}");
    assert!(String::from_utf8_lossy(&r.stderr).contains("recursive calls require a native stack-budget contract"), "{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn int32_parsing_preserves_grammar_range_precedence_and_null_faults() {
    let seed = neoclr::assemble(".module System\n.references ()\n.function neoCLR.Runtime.ParseInt32(String) -> Value\n.methodimpl InternalCall\n.end").unwrap();
    let mut cases: Vec<String> = ["", "+", "-", "0", "+0", "-0", "42", "-42", "2147483647", "-2147483648",
        "2147483648", "-2147483649", "999999999999999999999", "999999999999999999999x",
        "0000000000000000000000000000001", " 1", "1 ", "1\n", "1\0", "é", "１２", "++1", "--1", "+-1", "0x10"]
        .into_iter().map(str::to_owned).collect();
    for byte in 0..128u8 { cases.push(format!("999999999999999{}", char::from(byte))); }
    let mut source = String::from(".module Parse\n.function Calculate(Int32 mode) -> Int32\n.local String absent\n");
    for (i, _) in cases.iter().enumerate() {
        source.push_str(&format!("ldarg mode\nldc.i4 {i}\nceq\nbrtrue Case{i}\n"));
    }
    source.push_str("ldloca absent\ninitobj String\nldloc absent\nbr Parse\n");
    for (i, text) in cases.iter().enumerate() {
        let literal = serde_json::to_string(text).unwrap();
        source.push_str(&format!("Case{i}:\nldstr {literal}\nbr Parse\n"));
    }
    source.push_str("Parse:\ncall neoCLR.Runtime.ParseInt32(String)\ndup\nvalue.is Int32\nbrtrue Good\nvalue.unpack Byte\nconv.i4\nldc.i4 100\nadd\nret\nGood:\nvalue.unpack Int32\nret\n.end\n");
    let dir = Temp::new();
    let denied = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena"], false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--bind-integer-text"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert!(report["nativeBindings"].as_array().unwrap().iter().any(|row| row["implementation"] == "parse-int32-v1"));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"), r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    neoclr_aot_context ctx = {0};
    int32_t result = -99;
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (ctx.text.used) return 93;
    if (status) {
        if (result != -99 || ctx.fault.code != (uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr, &ctx.fault);
        return 1;
    }
    printf("%d\n", result);
    return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..=cases.len() {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode as i32)], neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        if mode == cases.len() {
            let fault = reference.unwrap_err();
            assert_eq!(r.status.code(), Some(1), "{r:?}");
            assert_eq!(String::from_utf8_lossy(&r.stderr), fault.diagnostic().to_string());
        } else {
            let neoclr::Value::Int32(value) = reference.unwrap().value else { panic!("unexpected result") };
            assert_eq!(r.status.code(), Some(0), "{mode}: {r:?}");
            assert_eq!(String::from_utf8_lossy(&r.stdout), format!("{value}\n"), "input {:?}", cases[mode]);
            assert!(r.stderr.is_empty());
        }
    }
}

#[test]
fn int32_parsing_rejects_ordinary_same_named_methods() {
    let seed = neoclr::assemble(".module System\n.references ()\n.function neoCLR.Runtime.ParseInt32(String) -> Value\nldc.i4 42\nvalue.pack Int32\nret\n.end").unwrap();
    let dir = Temp::new();
    let source = ".module Parse\n.function Calculate() -> Int32\nldstr \"1\"\ncall neoCLR.Runtime.ParseInt32(String)\nvalue.unpack Int32\nret\n.end";
    let r = compile_source(&dir, &seed, source, &["--compile-system", "--bind-integer-text"], false);
    assert!(!r.status.success() && !dir.0.join("app.o").exists(), "{r:?}");
    assert!(String::from_utf8_lossy(&r.stderr).contains("exact reserved Int32 parsing"), "{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn ordinal_string_predicates_preserve_utf8_nulls_and_allocation_independence() {
    let names = ["StringContainsOrdinal", "StringStartsWithOrdinal", "StringEndsWithOrdinal", "StringCompareOrdinal"];
    let mut seed_source = String::from(".module System\n.references ()\n");
    for name in names {
        let returns = if name == "StringCompareOrdinal" { "Int32" } else { "Boolean" };
        seed_source.push_str(&format!(".function neoCLR.Runtime.{name}(String, String) -> {returns}\n.methodimpl InternalCall\n.end\n"));
    }
    let seed = neoclr::assemble(&seed_source).unwrap();
    let cases = [("", ""), ("abc", ""), ("", "a"), ("abc", "a"), ("abc", "b"), ("abc", "c"),
        ("abc", "abc"), ("abc", "abcd"), ("abc", "A"), ("aaaaab", "aaab"), ("hé😀z", "é😀"),
        ("hé😀z", "😀z"), ("é😀", "é"), ("é", "é"), ("a\0b", "\0"), ("a\0b", "a\0"),
        ("a\0b", "\0b"), ("a\0b", "a\0c"), ("\u{10000}", "\u{e000}"), ("b", "a")];
    let mut source = String::from(".module Predicates\n.function Calculate(Int32 mode) -> Int32\n.local String absent\n");
    for i in 0..cases.len()*4+8 {
        source.push_str(&format!("ldarg mode\nldc.i4 {i}\nceq\nbrtrue Case{i}\n"));
    }
    source.push_str("ldc.i4 -1\nret\n");
    for (i, (text, pattern)) in cases.iter().enumerate() {
        for (j, name) in names.iter().enumerate() {
            let end = if j == 3 { "ret\n" } else { "brtrue Yes\nldc.i4 0\nret\n" };
            source.push_str(&format!("Case{}:\nldstr {}\nldstr {}\ncall neoCLR.Runtime.{name}(String, String)\n{end}", i*4+j, serde_json::to_string(text).unwrap(), serde_json::to_string(pattern).unwrap()));
        }
    }
    for j in 0..8 {
        source.push_str(&format!("Case{}:\nldloca absent\ninitobj String\n", cases.len()*4+j));
        source.push_str(if j<4 { "ldloc absent\nldstr \"\"\n" } else { "ldstr \"\"\nldloc absent\n" });
        let end = if j%4 == 3 { "ret\n" } else { "brtrue Yes\nldc.i4 0\nret\n" };
        source.push_str(&format!("call neoCLR.Runtime.{}(String, String)\n{end}", names[j%4]));
    }
    source.push_str("Yes:\nldc.i4 1\nret\n.end\n");
    let dir = Temp::new();
    let denied = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena"], false);
    assert!(!denied.status.success() && !dir.0.join("app.o").exists());
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--bind-utf8-text"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"), r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    neoclr_aot_context ctx = {0};
    int32_t result = -99;
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (ctx.text.used) return 93;
    if (status) {
        if (result != -99 || ctx.fault.code != (uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr, &ctx.fault);
        return 1;
    }
    printf("%d\n", result);
    return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds", "-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..cases.len()*4+8 {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode as i32)], neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        if mode >= cases.len()*4 {
            let fault = reference.unwrap_err();
            assert_eq!(r.status.code(), Some(1), "{r:?}");
            assert_eq!(String::from_utf8_lossy(&r.stderr), fault.diagnostic().to_string());
        } else {
            let neoclr::Value::Int32(value) = reference.unwrap().value else { panic!("unexpected result") };
            assert_eq!(r.status.code(), Some(0), "{mode}: {r:?}");
            assert_eq!(String::from_utf8_lossy(&r.stdout), format!("{value}\n"), "mode {mode}");
            assert!(r.stderr.is_empty());
        }
    }
    for name in names {
        let (returns, body) = if name == "StringCompareOrdinal" { ("Int32", "ldc.i4 0") } else { ("Boolean", "ldc.bool false") };
        let ordinary = neoclr::assemble(&format!(".module System\n.references ()\n.function neoCLR.Runtime.{name}(String, String) -> {returns}\n{body}\nret\n.end\n")).unwrap();
        let source = format!(".module Bad\n.function Calculate() -> Int32\nldstr \"a\"\nldstr \"a\"\ncall neoCLR.Runtime.{name}(String, String)\npop\nldc.i4 0\nret\n.end");
        let reject = Temp::new();
        let r = compile_source(&reject, &ordinary, &source, &["--compile-system", "--reference-arena", "--bind-utf8-text"], false);
        assert!(!r.status.success() && !reject.0.join("app.o").exists(), "{r:?}");
        assert!(String::from_utf8_lossy(&r.stderr).contains("native UTF-8 binding requires exact"), "{r:?}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_array_slots_preserve_owners_initialization_aliases_and_faults() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/string-arrays.neoil");
    let dir = Temp::new();
    let r = compile_source(&dir, &seed, source, &["--compile-system", "--reference-arena"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"), r#"
#include "text-arena.h"
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
    uint64_t storage[129]; memset(storage, 0xa5, sizeof(storage));
    unsigned capacity = argc > 2 ? (unsigned)atoi(argv[2]) : 1024;
    neoclr_aot_context ctx = {.text = {(unsigned char*)storage, capacity, 0}};
    int32_t result = -99;
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (ctx.text.used > capacity || storage[128] != UINT64_C(0xa5a5a5a5a5a5a5a5)) return 93;
    if (status) {
        if (result != -99 || ctx.fault.code != (uint32_t)status) return 92;
        if (!capacity && (ctx.text.used || storage[0] != UINT64_C(0xa5a5a5a5a5a5a5a5))) return 94;
        neoclr_aot_render_fault(stderr, &ctx.fault);
        return 1;
    }
    return result;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..9 {
        let reference = method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default());
        let r = Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(value) => {
                assert_eq!(value.value, neoclr::Value::Int32(0), "{mode}");
                assert_eq!(r.status.code(), Some(0), "{mode}: {r:?}");
                assert!(r.stdout.is_empty() && r.stderr.is_empty());
            }
            Err(fault) => {
                assert_eq!(r.status.code(), Some(1), "{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr), fault.diagnostic().to_string(), "{mode}");
            }
        }
    }
    let r = Command::new(dir.0.join("app")).args(["0", "0"]).output().unwrap();
    assert_eq!(r.status.code(), Some(1), "{r:?}");
    assert!(String::from_utf8_lossy(&r.stderr).starts_with("NativeMemoryLimitExceeded:"), "{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_wrapper_probes_keep_arguments_live_and_unlink_on_early_faults() {
    let dir = Temp::new();
    let source = r#".module WrapperRoots
.function Calculate(Int32 mode) -> Int32
.local String absent
ldarg mode
ldc.i4 2
ceq
brtrue Null
ldstr "native boundary"
br Write
Null:
ldloca absent
initobj String
ldloc absent
Write:
call neoCLR.Runtime.WriteLine(String)
pop
ldc.i4 42
ret
.end
"#;
    let seed = neoclr::assemble(OUTPUT_SEED).unwrap();
    let r = compile_source(&dir, &seed, source,
        &["--compile-system", "--bind-console-write-line", "--probe-stack-roots"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"), r#"
#include "root-probe.h"
#include "fault-details.h"
#include <stdlib.h>
#include <string.h>
static unsigned calls;
static int failure;
int32_t neoclr_console_write_line_utf8_v1(const uint8_t *bytes, size_t length) {
    const neoclr_probe_frame *f = neoclr_root_probe_head_v1();
    if (!f || !f->previous || f->previous->previous || neoclr_root_probe_depth_v1() != 2 ||
        f->context != f->previous->context || f->storage_count != 1 || f->lane_count ||
        f->storage[0].read_bytes != 8 || f->storage[0].flags ||
        !strstr(f->storage_plan, "\"argument\":true")) abort();
    const neoclr_aot_text *text = *(const neoclr_aot_text *const *)f->storage[0].address;
    if (!text || text->bytes != bytes || text->length != length || length != 15 ||
        memcmp(bytes, "native boundary", length)) abort();
    calls++;
    return failure;
}
int main(void) {
    neoclr_aot_fault fault = {0};
    /* Success, service failure, null before service invocation, then reentry. */
    for (int mode = 0; mode < 4; mode++) {
        int32_t result = -99;
        failure = mode == 1;
        int status = neoclr_entry_v3(mode, &result, &fault);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 1;
        if (mode == 1 || mode == 2) {
            if (status != 3 || result != -99 || fault.code != 3 || fault.frame_count != 1) return 2;
        } else if (status || result != 42 || fault.code || fault.frame_count) return 3;
    }
    return calls == 3 ? 0 : 4;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(base.join("aot-console")).arg("-I").arg(base.join("aot-fault-details"))
        .arg(dir.0.join("host.c")).arg(base.join("aot-console/root-probe.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn fault_root_slots_survive_unwind_and_reset_with_the_owning_context() {
    let dir = Temp::new();
    let source = r#".module FaultRoots
.function Calculate(Int32 mode) -> Int32
ldarg mode
brfalse Success
ldarg mode
call neoCLR.Runtime.Int32ToString(Int32)
call System.Fail(String)
Success:
ldc.i4 42
ret
.end
"#;
    let seed = neoclr::assemble(TEXT_SEED).unwrap();
    let r = compile_source(&dir, &seed, source,
        &["--compile-system", "--bind-int32-to-string", "--bind-user-fault", "--probe-stack-roots"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "root-probe.h"
#include "text-arena.h"
#include <string.h>
static int verify(neoclr_aot_context *ctx, int32_t value, const char *expected,
                  neoclr_probe_storage *slots) {
    int32_t result = -99;
    if (neoclr_entry_v4(value, &result, ctx) != 4 || result != -99 || ctx->fault.code != 4 ||
        ctx->fault.frame_count != 2 || neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 1;
    if (neoclr_probe_fault_roots_v1(ctx, slots, 65) != 3 || slots[0].address != &ctx->fault.message ||
        slots[1].address != &ctx->fault.frames[0].function ||
        slots[2].address != &ctx->fault.frames[1].function) return 2;
    const neoclr_aot_text *text = *(const neoclr_aot_text *const *)slots[0].address;
    if ((uintptr_t)text < (uintptr_t)ctx->text.data ||
        (uintptr_t)text >= (uintptr_t)ctx->text.data + ctx->text.used ||
        text->length != strlen(expected) || memcmp(text->bytes, expected, text->length)) return 3;
    return neoclr_aot_render_fault(stderr, &ctx->fault);
}
int main(void) {
    uint64_t a_buffer[32], b_buffer[32];
    neoclr_aot_context a = {.text = {(unsigned char *)a_buffer, sizeof(a_buffer), 0}};
    neoclr_aot_context b = {.text = {(unsigned char *)b_buffer, sizeof(b_buffer), 0}};
    neoclr_probe_storage a_slots[65], b_slots[65];
    if (verify(&a, 12345, "12345", a_slots) || verify(&b, -77, "-77", b_slots)) return 1;
    /* A second independent context does not replace the first host's fault roots. */
    const neoclr_aot_text *a_text = *(const neoclr_aot_text *const *)a_slots[0].address;
    if (a_text->length != 5 || memcmp(a_text->bytes, "12345", 5)) return 2;
    int32_t result = -99;
    if (neoclr_entry_v4(0, &result, &a) || result != 42 || a.text.used ||
        neoclr_probe_fault_roots_v1(&a, NULL, 0) != 0 ||
        *(const neoclr_aot_text *const *)a_slots[0].address ||
        neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 3;
    /* The old frame-name storage can remain populated; code/count retire that view. */
    if (neoclr_probe_fault_roots_v1(&b, b_slots, 65) != 3 || b.fault.message->length != 3) return 4;
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    let expected: String = [12345, -77].into_iter().map(|mode| {
        method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default())
            .unwrap_err().diagnostic().to_string()
    }).collect();
    assert_eq!(String::from_utf8_lossy(&r.stderr), expected);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_gc_preserves_borrowed_owners_erased_results_and_fault_messages() {
    let dir = Temp::new();
    let source = include_str!("../../../docs/experiments/aot-console/native-gc.neoil");
    let seed = neoclr::assemble(TEXT_SEED).unwrap();
    let flags = ["--compile-system", "--reference-arena", "--bind-int32-to-string", "--bind-user-fault", "--native-gc"];
    let inspection = compile_source(&dir, &seed, source, &flags, true);
    assert!(inspection.status.success(), "{}", String::from_utf8_lossy(&inspection.stderr));
    let report: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(report["admission"]["accepted"], true, "{report}");
    assert_eq!(report["capabilities"]["nativeGC"], true);
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let imports = Command::new("nm").arg("-u").arg(dir.0.join("app.o")).output().unwrap();
    let imports = String::from_utf8_lossy(&imports.stdout);
    assert!(imports.contains("neoclr_gc_stack_roots_v1") && !imports.contains("neoclr_probe_stack_roots_v2"));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#include <string.h>
int main(void) {
    uint64_t buffer[257];
    buffer[256] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 2048, 0}};
    for (int mode = 0; mode < 3; mode++) {
        int32_t result = -99;
        int status = neoclr_entry_v4(mode == 1, &result, &ctx);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 1;
        if (mode == 1) {
            if (status != 4 || result != -99 || ctx.fault.code != 4 || ctx.fault.frame_count != 3) return 2;
            if (neoclr_gc_collect_v1(&ctx, NULL)) return 3;
            const neoclr_aot_text *message = ctx.fault.message;
            if (message->length != 5 || memcmp(message->bytes, "12345", 5)) return 4;
            if (neoclr_aot_render_fault(stderr, &ctx.fault)) return 5;
        } else {
            if (status || result != 42 || ctx.fault.code) return 6;
            if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 7;
        }
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.collections < 1500 || stats.reclaimed_allocations < 1500 || stats.allocations < 1500) return 8;
    ctx.text.capacity = 64;
    int32_t result = -99;
    if (neoclr_entry_v4(0, &result, &ctx) != 5 || result != -99 || ctx.fault.code != 5 ||
        neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 9;
    if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 10; /* runtime message is image data */
    return buffer[256] == UINT64_C(0x1122334455667788) ? 0 : 11;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    let fault = method.invoke(vec![neoclr::Value::Int32(1)], neoclr::Limits::default()).unwrap_err();
    assert_eq!(String::from_utf8_lossy(&r.stderr), fault.diagnostic().to_string());
}

const SOCKET_SEED: &str = ".module System\n.references ()\n";

#[test]
fn socket_listener_binding_requires_explicit_exact_contracts() {
    let source = include_str!("../../../docs/experiments/aot-console/socket-listener.neoil");
    let seed = neoclr::assemble(SOCKET_SEED).unwrap();
    for flags in [vec![], vec!["--bind-socket-listener"], vec!["--compile-system", "--bind-socket-listener"], vec!["--compile-system", "--reference-arena"], vec!["--compile-system", "--reference-arena", "--bind-socket-listener", "--bind-socket-listener"]] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    for index in 0..3 {
        let mut changed = source.to_owned();
        let at = changed.match_indices(".methodimpl InternalCall").nth(index).unwrap().0;
        changed.replace_range(at..at + ".methodimpl InternalCall".len(), "ldvoid\nvalue.pack Void\nret");
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &changed, &["--compile-system", "--reference-arena", "--bind-socket-listener"], false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("exact reserved Socket"), "{}", String::from_utf8_lossy(&r.stderr));
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_socket_listener_lifecycle_cleans_faults_and_unclosed_handles() {
    let dir = Temp::new();
    let seed = neoclr::assemble(SOCKET_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/socket-listener.neoil");
    let flags = ["--compile-system", "--reference-arena", "--bind-socket-listener", "--native-gc"];
    let r = compile_source(&dir, &seed, source, &flags, true);
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["admission"]["accepted"], true, "{report}");
    assert_eq!(report["selection"]["nativeBindings"].as_array().unwrap().len(), 3);
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&r.stdout).unwrap(), report["selection"]);
    fs::write(dir.0.join("host.c"), r#"
#include "socket-listener.h"
#include "root-probe.h"
#include <errno.h>
#include <fcntl.h>
int main(void) {
    uint64_t storage[8192];
    neoclr_aot_context c = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    int32_t result = -99;
    if (neoclr_entry_v4(0, &result, &c) != 3 || result != -99 || c.fault.code != 3 || neoclr_root_probe_head_v1()) return 1;
    for (int i = 0; i < 3; i++) {
        neoclr_socket_scope scope;
        if (neoclr_socket_scope_enter_v1(&scope, &c)) return 2;
        result = -99;
        int status = neoclr_entry_v4(i, &result, &c);
        if (status != (i == 1 ? 4 : 0) || result != (i == 1 ? -99 : 42) || neoclr_root_probe_head_v1()) return 3;
        int count = 0, descriptor = -1;
        for (int j = 0; j < 64; j++) if (scope.slots[j].id) { count++; descriptor = scope.slots[j].descriptor; }
        if (count != (i == 0 ? 0 : 1) || neoclr_socket_scope_leave_v1(&scope)) return 4;
        if (count && (fcntl(descriptor, F_GETFD) != -1 || errno != EBADF)) return 5;
    }
    return 0;
}
"#).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for (name, inputs) in [
        ("kernel", vec![base.join("socket-listener-test.c"), base.join("socket-listener.c"), base.join("root-probe.c"), base.join("native-gc.c"), base.join("text-arena.c")]),
        ("compiled", vec![dir.0.join("host.c"), dir.0.join("app.o"), base.join("socket-listener.c"), base.join("root-probe.c"), base.join("native-gc.c"), base.join("text-arena.c")]),
    ] {
        let binary = dir.0.join(name);
        let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds", "-DNEOCLR_NATIVE_GC", "-I"]).arg(&base).args(inputs).arg("-o").arg(&binary).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(binary).output().unwrap();
        assert!(r.status.success(), "{r:?}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_callbacks_retain_receivers_and_propagate_faults() {
    check_native_callbacks(include_str!("../../../docs/experiments/aot-console/callbacks.neoil"));
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_interface_callbacks_retain_receivers_and_propagate_faults() {
    let source = include_str!("../../../docs/experiments/aot-console/callbacks.neoil")
        .replace(".type class Counter\n", ".interface Adder\n.method instance Add(Int32 delta) -> Int32\n.end\n.end\n.type class Counter\n.implements Adder\n")
        .replace("function.bind fn<Int32,Int32> = instance Counter::Add(Int32)", "castclass Adder\nfunction.bind fn<Int32,Int32> = instance Adder::Add(Int32)")
        .replace("Fault:\nfunction.bind fn<Int32,Int32> = Fail(Int32)", "Fault:\nnewobj.ctor instance Broken::.ctor()\ncastclass Adder\nfunction.bind fn<Int32,Int32> = instance Adder::Add(Int32)");
    let source = source + "\n.type class Broken\n.implements Adder\n.method instance .ctor() -> noresult\nret\n.end\n.method instance Add(Int32 number) -> Int32\nldarg number\nldc.i4 0\ndiv\nret\n.end\n.end\n";
    check_native_callbacks(&source);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn check_native_callbacks(source: &str) {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
int main(void) {
    uint64_t buffer[257];
    buffer[256] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 2048, 0}};
    for (int mode = 0; mode < 4; mode++) {
        int32_t result = -99;
        int status = neoclr_entry_v4(mode, &result, &ctx);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 1;
        if (!mode) {
            if (status || result != 42 || ctx.fault.code) return 2;
        } else {
            if (status != (mode == 1 ? 1 : mode == 2 ? 3 : 6) || result != -99 || ctx.fault.code != (uint32_t)status) return 3;
            if (neoclr_aot_render_fault(stderr, &ctx.fault)) return 4;
        }
        if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 5;
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.collections < 1000 || stats.reclaimed_allocations < 1000) return 6;
    return buffer[256] == UINT64_C(0x1122334455667788) ? 0 : 7;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    assert_eq!(method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let mut faults = String::new();
    for mode in 1..4 {
        let fault = method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default()).unwrap_err();
        faults.push_str(&fault.diagnostic().to_string());
    }
    assert_eq!(String::from_utf8_lossy(&r.stderr), faults);
}

#[test]
fn native_callbacks_reject_recursion_borrows_and_excess_dispatch_targets() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    let recursive = ".module Recursive\n.function Again() -> Int32\nfunction.bind fn<Int32> = Again()\ncall instance fn<Int32>::Invoke()\nret\n.end\n.function Calculate() -> Int32\ncall Again()\nret\n.end";
    let borrowed = ".module Borrowed\n.function Read(Int32& value) -> Int32\nldarg value\nldobj Int32\nret\n.end\n.function Calculate() -> Int32\nfunction.bind fn<Int32&,Int32> = Read(Int32&)\npop\nldc.i4 0\nret\n.end";
    let mut many = String::from(".module Many\n");
    for i in 0..33 { many.push_str(&format!(".function Target{i}() -> Int32\nldc.i4 {i}\nret\n.end\n")); }
    many.push_str(".function Calculate() -> Int32\n");
    for i in 0..33 { many.push_str(&format!("function.bind fn<Int32> = Target{i}()\npop\n")); }
    many.push_str("function.bind fn<Int32> = Target0()\ncallvirt instance fn<Int32>::Invoke()\nret\n.end\n");
    for (source, message) in [(recursive, "recursive calls"), (borrowed, "borrowed"), (many.as_str(), "32 possible targets")] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success());
        if source == recursive {
            let diagnostic = String::from_utf8_lossy(&r.stderr);
            assert_eq!(diagnostic.matches("Again [#").count(), 2, "{diagnostic}");
            assert!(diagnostic.contains(" -> "), "{diagnostic}");
        }
        assert!(String::from_utf8_lossy(&r.stderr).contains(message), "{}", String::from_utf8_lossy(&r.stderr));
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_callback_arrays_trace_initialized_slots_and_release_replacements() {
    let dir = Temp::new();
    let source = include_str!("../../../docs/experiments/aot-console/callback-arrays.neoil");
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
int main(void) {
    uint64_t buffer[257];
    buffer[256] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 2048, 0}};
    for (int mode = 0; mode < 4; mode++) {
        int32_t result = -99;
        int status = neoclr_entry_v4(mode, &result, &ctx);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 1;
        if (!mode) {
            if (status || result != 42 || ctx.fault.code) return 2;
        } else {
            if (status != (mode == 1 ? 3 : mode == 2 ? 8 : 6) || result != -99 || ctx.fault.code != (uint32_t)status) return 3;
            if (neoclr_aot_render_fault(stderr, &ctx.fault)) return 4;
        }
        if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 5;
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.collections < 1000 || stats.reclaimed_allocations < 1000) return 6;
    return buffer[256] == UINT64_C(0x1122334455667788) ? 0 : 7;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    assert_eq!(method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let mut faults = String::new();
    for mode in 1..4 {
        let fault = method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default()).unwrap_err();
        faults.push_str(&fault.diagnostic().to_string());
    }
    assert_eq!(String::from_utf8_lossy(&r.stderr), faults);
}

#[test]
fn native_callback_arrays_reject_element_borrows_and_shape_substitution() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    for body in [
        "ldc.i4 1\nnewarr fn<Int32>\nldc.i4 0\nldelema fn<Int32>\npop",
        "ldc.i4 1\nnewarr fn<Int32>\nldc.i4 0\nfunction.bind fn<Boolean> = Truth()\nstelem fn<Int32>",
    ] {
        let source = format!(".module InvalidArray\n.function Truth() -> Boolean\nldc.bool true\nret\n.end\n.function Calculate() -> Int32\n{body}\nldc.i4 0\nret\n.end");
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &source, &flags, false);
        assert!(!r.status.success(), "unsupported callback array operation admitted");
        assert!(String::from_utf8_lossy(&r.stderr).contains("unsupported value instruction") ||
            String::from_utf8_lossy(&r.stderr).contains("verification:"), "{}", String::from_utf8_lossy(&r.stderr));
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_record_arrays_preserve_snapshots_nested_references_and_faults() {
    let dir = Temp::new();
    let source = include_str!("../../../docs/experiments/aot-console/record-arrays.neoil");
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
int main(void) {
    uint64_t buffer[257];
    buffer[256] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 2048, 0}};
    for (int mode = 0; mode < 3; mode++) {
        int32_t result = -99;
        int status = neoclr_entry_v4(mode, &result, &ctx);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 1;
        if (!mode) {
            if (status || result != 42 || ctx.fault.code) return 2;
        } else {
            if (status != (mode == 1 ? 3 : mode == 2 ? 8 : 6) || result != -99 || ctx.fault.code != (uint32_t)status) return 3;
            if (neoclr_aot_render_fault(stderr, &ctx.fault)) return 4;
        }
        if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 5;
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.collections < 1000 || stats.reclaimed_allocations < 1000) return 6;
    return buffer[256] == UINT64_C(0x1122334455667788) ? 0 : 7;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    assert_eq!(method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let mut faults = String::new();
    for mode in 1..3 {
        let fault = method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default()).unwrap_err();
        faults.push_str(&fault.diagnostic().to_string());
    }
    assert_eq!(String::from_utf8_lossy(&r.stderr), faults);
}

#[test]
fn native_record_arrays_require_reservation_and_indexed_value_access() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    for body in [
        "ldc.i4 1\nnewarr Cell\npop",
        "ldc.i4 1\narray.reserve Cell\nldc.i4 0\nldelema Cell\npop",
    ] {
        let source = format!(".module UnsupportedRecordArray\n.type Cell\n.field Number Int32\n.end\n.function Calculate() -> Int32\n{body}\nldc.i4 0\nret\n.end");
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &source, &flags, false);
        assert!(!r.status.success(), "unsupported record-array operation admitted");
        let message = String::from_utf8_lossy(&r.stderr);
        assert!(message.contains("unsupported value instruction") || message.contains("element borrows"), "{message}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_join_parts_preserves_bytes_snapshots_and_faults() {
    let dir=Temp::new();
    let seed=neoclr::assemble(concat!(".module System\n.references ()\n", include_str!("../../../runtime/neoCLR/Runtime/StringJoinParts.neoil"))).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/join-parts.neoil");
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[128];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in -1..8 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn sealed_virtual_calls_preserve_results_and_null_faults() {
    let dir=Temp::new();
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/sealed-member.neoil");
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[128];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..2 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
fn unsealed_virtual_calls_still_require_dispatch() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/sealed-member.neoil").replace(".sealed\n", "");
    let dir = Temp::new();
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena"], false);
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("virtual calls requiring dispatch"), "{r:?}");
    assert!(!dir.0.join("app.o").exists());
}

#[test]
fn descriptive_sealing_does_not_hide_an_unselected_derived_type() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = format!("{}\n.type class Derived\n.extends Answer\n.end\n",
        include_str!("../../../docs/experiments/aot-console/sealed-member.neoil"));
    let dir = Temp::new();
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena"], false);
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("virtual calls requiring dispatch"), "{r:?}");
    assert!(!dir.0.join("app.o").exists());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn enum_values_preserve_signed_domain_nominal_calls_and_array_snapshots() {
    let dir=Temp::new();
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/enum-values.neoil");
    let r=compile_source(&dir,&seed,source,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[128];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assemble(source).unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in [i32::MIN, -1, 0, 200, 599, i32::MAX] {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
fn enum_layout_and_nominal_identity_are_verified_before_native_projection() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let original = include_str!("../../../docs/experiments/aot-console/enum-values.neoil");
    for (source, diagnostic) in [
        (original.replace(".field private Bits Int32", ".field Bits Int32"), "enum requires"),
        (original.replace(".field private Bits Int32", ".field private Bits Int64"), "enum requires"),
        (original.replace("call Echo<Status>(Status)", "call Echo<Other>(Other)")
            + "\n.type Other\n.enum Int32\n.field private Bits Int32\n.end\n", "Fault:"),
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena"], false);
        assert!(!r.status.success());
        assert!(String::from_utf8_lossy(&r.stderr).contains(diagnostic), "{r:?}");
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_reference_arrays_preserve_identity_owners_and_faults() {
    check_reference_arrays(false);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_default_reference_arrays_preserve_nulls_identity_and_faults() {
    check_reference_arrays(true);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn check_reference_arrays(default_initialized: bool) {
    let dir = Temp::new();
    let mut source = include_str!("../../../docs/experiments/aot-console/reference-arrays.neoil").to_owned();
    if default_initialized {
        source = source.replace("ldc.i4 2\narray.reserve Counter", r#"ldarg mode
ldc.i4 4
beq Negative
ldarg mode
ldc.i4 5
beq Limit
ldc.i4 2
br Allocate
Negative:
ldc.i4 -1
br Allocate
Limit:
ldc.i4 65537
Allocate:
newarr Counter"#);
    }
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    let r = compile_source(&dir, &seed, &source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
int main(void) {
    uint64_t buffer[257];
    buffer[256] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 2048, 0}};
    const int expected[] = {0, UNWRITTEN_FAULT, 8, 6, 3, 7};
    for (int mode = 0; mode < MODE_COUNT; mode++) {
        int32_t result = -99;
        int status = neoclr_entry_v4(mode, &result, &ctx);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 1;
        if (!mode) {
            if (status || result != 42 || ctx.fault.code) return 2;
        } else {
            if (status != expected[mode] || result != -99 || ctx.fault.code != (uint32_t)status) return 3;
            if (neoclr_aot_render_fault(stderr, &ctx.fault)) return 4;
        }
        if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 5;
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.collections < 1000 || stats.reclaimed_allocations < 1000) return 6;
    return buffer[256] == UINT64_C(0x1122334455667788) ? 0 : 7;
}
"#.replace("MODE_COUNT", if default_initialized { "6" } else { "4" })
        .replace("UNWRITTEN_FAULT", if default_initialized { "6" } else { "3" })).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    let app = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    assert_eq!(method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let mut faults = String::new();
    for mode in 1..if default_initialized { 6 } else { 4 } {
        let fault = method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default()).unwrap_err();
        faults.push_str(&fault.diagnostic().to_string());
    }
    assert_eq!(String::from_utf8_lossy(&r.stderr), faults);
}

#[test]
fn native_reference_arrays_reject_borrows_and_wrong_elements() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let flags = ["--compile-system", "--reference-arena", "--native-gc"];
    for body in [
        "ldc.i4 1\nnewarr Cell\nldc.i4 0\nldelema Cell\npop",
        "ldc.i4 1\nnewarr Cell\nldc.i4 0\nldloca other\ninitobj Other\nldloc other\nstelem Cell",
        "ldc.i4 1\narray.reserve Cell\nldc.i4 0\nldelema Cell\npop",
        "ldc.i4 1\narray.reserve Cell\nldc.i4 0\nldloca other\ninitobj Other\nldloc other\nstelem Cell",
    ] {
        let source = format!(".module InvalidReferenceArray\n.type class Cell\n.end\n.type class Other\n.end\n.function Calculate() -> Int32\n.local Other other\n{body}\nldc.i4 0\nret\n.end");
        let dir = Temp::new();
        let result = compile_source(&dir, &seed, &source, &flags, false);
        assert!(!result.status.success());
        assert!(!dir.0.join("app.o").exists());
        let message = String::from_utf8_lossy(&result.stderr);
        assert!(message.contains("unsupported value instruction") || message.contains("element borrows") || message.contains("Fault:"), "{message}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_gc_entry_rejects_live_host_handles_before_heap_reset() {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = ".module App\n.function Calculate() -> Int32\nldc.i4 42\nret\n.end";
    let r = compile_source(&dir, &seed, source,
        &["--compile-system", "--reference-arena", "--native-gc"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[129] = {0}; buffer[128] = 1234567;
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 1024, 0}};
    void *object = NULL, *read = NULL;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text, 16, NEOCLR_GC_OBJECT, &object));
    ((uint64_t *)object)[1] = 77;
    uint64_t handle = 0, used = ctx.text.used;
    CHECK(!neoclr_gc_host_root_create_v1(&ctx, object, &handle));
    int32_t result = -99;
    CHECK(neoclr_entry_v4(0, &result, &ctx) == 3);
    CHECK(result == -99 && ctx.fault.code == 3 && ctx.text.used == used);
    CHECK(!neoclr_gc_host_root_read_v1(&ctx, handle, &read) && read == object);
    CHECK(!neoclr_gc_collect_v1(&ctx, NULL) && ((uint64_t *)object)[1] == 77);
    CHECK(!neoclr_gc_host_root_release_v1(&ctx, handle));
    CHECK(!neoclr_entry_v4(0, &result, &ctx) && result == 42 && !ctx.fault.code);
    CHECK(!ctx.text.used && !neoclr_root_probe_depth_v1());
    CHECK(!neoclr_gc_host_root_create_v1(&ctx, NULL, &handle));
    result = -99;
    CHECK(neoclr_entry_v4(0, &result, &ctx) == 3 && result == -99);
    CHECK(!neoclr_gc_host_root_release_v1(&ctx, handle));
    CHECK(!neoclr_entry_v4(0, &result, &ctx) && result == 42 && !ctx.fault.code);
    CHECK(buffer[128] == 1234567);
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_host_callbacks_preserve_roots_state_and_faults_without_entry_reset() {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/host-callbacks.neoil");
    let r = compile_source(&dir, &seed, source,
        &["--compile-system", "--reference-arena", "--native-gc"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
/* Fixture-only discovery of the single descriptor left in a guest local at return.
 * Real services receive and root callbacks during submission, never scan the heap. */
static uint64_t *descriptor(neoclr_aot_context *c) {
    for (uint64_t at = 0; at < c->text.used;) {
        uint64_t *header = (void *)(c->text.data + at);
        if (header[3] && header[1] == 24 && header[2] == NEOCLR_GC_OBJECT && header[4] == UINT32_MAX)
            return header + 4;
        at += header[0];
    }
    return NULL;
}
int main(void) {
    uint64_t buffer[257] = {0}; buffer[256] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 2048, 0}}, other = {0};
    for (int mode = 0; mode < 4; mode++) {
        int32_t result = -1;
        CHECK(!neoclr_entry_v4(mode, &result, &c) && result == 42);
        uint64_t *callback = descriptor(&c);
        CHECK(callback);
        uint64_t handle = 0;
        CHECK(!neoclr_gc_host_root_create_v1(&c, callback, &handle));
        CHECK(neoclr_invoke_void_callback_v1(handle, &other) == 3);
        neoclr_probe_frame frame;
        neoclr_probe_storage empty = {0};
        neoclr_probe_enter_v3(&frame, &c, 0, &empty, 0, "", 0);
        CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3 && !c.fault.code);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t used = c.text.used;
        if (mode == 3) {
            CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3 && !c.fault.code);
        } else if (mode == 2) {
            CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 1 && c.fault.code == 1);
            CHECK(c.fault.frame_count == 1);
            CHECK(!neoclr_aot_render_fault(stderr, &c.fault));
            CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3 && c.fault.code == 1);
        } else {
            CHECK(!neoclr_invoke_void_callback_v1(handle, &c));
            CHECK(!neoclr_gc_collect_v1(&c, NULL));
            CHECK(!neoclr_invoke_void_callback_v1(handle, &c));
            if (!mode) CHECK(*(int32_t *)(uintptr_t)(callback[2] + 8) == 2);
        }
        CHECK(c.text.used == used && !neoclr_root_probe_depth_v1());
        CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
        CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3);
        c.fault.code = 0;
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    }
    uint64_t handle;
    CHECK(!neoclr_gc_host_root_create_v1(&c, NULL, &handle));
    CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    void *ordinary = NULL, *output = (void *)(uintptr_t)123;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 24, NEOCLR_GC_OBJECT, &ordinary));
    CHECK(!neoclr_gc_host_root_create_v1(&c, ordinary, &handle));
    CHECK(neoclr_gc_callback_read_v1(&c, handle, &output) == 3 && output == (void *)(uintptr_t)123);
    CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    CHECK(buffer[256] == 1234567);
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Fail()").unwrap()).unwrap();
    let fault = method.invoke(vec![], neoclr::Limits::default()).unwrap_err();
    assert_eq!(String::from_utf8_lossy(&r.stderr), fault.diagnostic().to_string());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn compiled_socket_accept_consumes_results_from_retained_guest_callbacks() {
    let dir = Temp::new();
    let seed = neoclr::assemble(SOCKET_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/socket-accept.neoil");
    let flags = ["--compile-system", "--reference-arena", "--native-gc", "--bind-socket-listener", "--bind-socket-accept"];
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "socket-listener.h"
#include "native-gc.h"
#include <arpa/inet.h>
#include <sys/socket.h>
#include <unistd.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[513] = {0}; buffer[512] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 4096, 0}};
    for (int cancel = 0; cancel < 2; cancel++) {
        neoclr_socket_scope scope;
        CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
        int32_t port = -1;
        CHECK(!neoclr_entry_v4(cancel, &port, &c) && port > 0);
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t root = 999;
        int peer = -1;
        if (!cancel) {
            CHECK(!neoclr_socket_poll_v1(&c, &root) && root == 999);
            struct sockaddr_in endpoint = {0};
            endpoint.sin_family = AF_INET; endpoint.sin_port = htons((uint16_t)port);
            endpoint.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
            peer = socket(AF_INET, SOCK_STREAM, 0);
            CHECK(peer >= 0 && !connect(peer, (const void *)&endpoint, sizeof(endpoint)));
        }
        int ready = 0;
        for (unsigned retry = 0; retry < 1000 && !ready; retry++) ready = neoclr_socket_poll_v1(&c, &root);
        CHECK(ready == 1);
        CHECK(!neoclr_invoke_void_callback_v1(root, &c) && !c.fault.code);
        CHECK(!neoclr_root_probe_depth_v1() && !neoclr_socket_poll_v1(&c, &root));
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        unsigned sockets = 0;
        for (unsigned i = 0; i < 64; i++) {
            CHECK(!scope.operations[i].state);
            sockets += scope.slots[i].id != 0;
        }
        CHECK(sockets == 1); /* Guest callback closed accepted connection. */
        CHECK(!neoclr_socket_scope_leave_v1(&scope));
        if (peer >= 0) close(peer);
    }
    CHECK(buffer[512] == 1234567);
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("socket-listener.c")).arg(base.join("root-probe.c"))
        .arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
}

#[test]
fn socket_accept_binding_requires_gc_and_exact_internal_services() {
    let seed = neoclr::assemble(SOCKET_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/socket-accept.neoil");
    for flags in [
        vec!["--bind-socket-accept"],
        vec!["--compile-system", "--reference-arena", "--bind-socket-accept"],
        vec!["--compile-system", "--reference-arena", "--native-gc", "--bind-socket-listener"],
        vec!["--bind-socket-accept", "--bind-socket-accept"],
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    for service in ["SocketAccept", "SocketConnectResult", "SocketCancel"] {
        let mut changed = source.to_owned();
        let start = changed.find(&format!(".function neoCLR.Runtime.{service}")).unwrap();
        let at = start + changed[start..].find(".methodimpl InternalCall").unwrap();
        changed.replace_range(at..at + ".methodimpl InternalCall".len(),
            if service == "SocketCancel" { "ldc.bool false\nret" } else { "ldvoid\nvalue.pack Void\nret" });
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &changed,
            &["--compile-system", "--reference-arena", "--native-gc", "--bind-socket-listener", "--bind-socket-accept"], false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("exact reserved Accept/ConnectResult/Cancel"), "{}", String::from_utf8_lossy(&r.stderr));
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn compiled_socket_echo_chains_accept_receive_send_and_reclaims_receivers() {
    let dir = Temp::new();
    let seed = neoclr::assemble(SOCKET_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/socket-echo.neoil");
    let flags = ["--compile-system", "--reference-arena", "--native-gc", "--bind-socket-listener", "--bind-socket-accept", "--bind-socket-transfer"];
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "socket-listener.h"
#include "native-gc.h"
#include <arpa/inet.h>
#include <sys/socket.h>
#include <unistd.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[513] = {0}; buffer[512] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 4096, 0}};
    for (int cancel = 0; cancel < 1; cancel++) {
        neoclr_socket_scope scope;
        CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
        int32_t port = -1;
        CHECK(!neoclr_entry_v4(cancel, &port, &c) && port > 0);
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t root = 999;
        int peer = -1;
        if (!cancel) {
            CHECK(!neoclr_socket_poll_v1(&c, &root) && root == 999);
            struct sockaddr_in endpoint = {0};
            endpoint.sin_family = AF_INET; endpoint.sin_port = htons((uint16_t)port);
            endpoint.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
            peer = socket(AF_INET, SOCK_STREAM, 0);
            CHECK(peer >= 0 && !connect(peer, (const void *)&endpoint, sizeof(endpoint)));
        }
        int ready = 0;
        for (unsigned retry = 0; retry < 1000 && !ready; retry++) ready = neoclr_socket_poll_v1(&c, &root);
        CHECK(ready == 1);
        CHECK(!neoclr_invoke_void_callback_v1(root, &c) && !c.fault.code);
        CHECK(send(peer, "x", 1, 0) == 1);
        for (unsigned stage = 0; stage < 2; stage++) {
            CHECK(!neoclr_gc_collect_v1(&c, NULL));
            ready = 0;
            for (unsigned retry = 0; retry < 1000 && !ready; retry++) ready = neoclr_socket_poll_v1(&c, &root);
            CHECK(ready == 1 && !neoclr_invoke_void_callback_v1(root, &c) && !c.fault.code);
        }
        char echoed = 0;
        CHECK(recv(peer, &echoed, 1, 0) == 1 && echoed == 'x');
        CHECK(!neoclr_root_probe_depth_v1() && !neoclr_socket_poll_v1(&c, &root));
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        unsigned sockets = 0;
        for (unsigned i = 0; i < 64; i++) {
            CHECK(!scope.operations[i].state);
            sockets += scope.slots[i].id != 0;
        }
        CHECK(sockets == 1); /* Guest callback closed accepted connection. */
        CHECK(!neoclr_socket_scope_leave_v1(&scope));
        if (peer >= 0) close(peer);
    }
    CHECK(buffer[512] == 1234567);
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("socket-listener.c")).arg(base.join("root-probe.c"))
        .arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
}

#[test]
fn socket_transfer_binding_requires_gc_and_exact_internal_services() {
    let seed = neoclr::assemble(SOCKET_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/socket-echo.neoil");
    for flags in [
        vec!["--bind-socket-transfer"],
        vec!["--compile-system", "--reference-arena", "--bind-socket-transfer"],
        vec!["--compile-system", "--reference-arena", "--native-gc", "--bind-socket-listener"],
        vec!["--bind-socket-transfer", "--bind-socket-transfer"],
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    for service in ["SocketReceive", "SocketSend", "SocketTransferResult"] {
        let mut changed = source.to_owned();
        let start = changed.find(&format!(".function neoCLR.Runtime.{service}")).unwrap();
        let at = start + changed[start..].find(".methodimpl InternalCall").unwrap();
        changed.replace_range(at..at + ".methodimpl InternalCall".len(),
            "ldvoid\nvalue.pack Void\nret");
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &changed,
            &["--compile-system", "--reference-arena", "--native-gc", "--bind-socket-listener", "--bind-socket-accept", "--bind-socket-transfer"], false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("exact reserved Receive/Send/TransferResult"), "{}", String::from_utf8_lossy(&r.stderr));
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn compiled_socket_echo_uses_deadline_service_bindings() {
    let dir = Temp::new();
    let seed = neoclr::assemble(SOCKET_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/socket-echo.neoil");
    let flags = ["--compile-system", "--reference-arena", "--native-gc", "--bind-socket-listener", "--bind-socket-accept", "--bind-socket-transfer"];
    let source = source.replace("ldarg 0\nfunction.bind fn<Void> = instance Completion::Complete()\ncall neoCLR.Runtime.SocketReceive(Int64, arrayref<Byte>, Int32, Int32, fn<Void>)", "ldc.i4 60000\ncall neoCLR.Runtime.SocketDeadlineAfter(Int32)\nldarg 0\nfunction.bind fn<Void> = instance Completion::Complete()\ncall neoCLR.Runtime.SocketReceiveUntil(Int64, arrayref<Byte>, Int32, Int32, Int64, fn<Void>)")
        .replace("ldarg 0\nfunction.bind fn<Void> = instance Completion::Complete()\ncall neoCLR.Runtime.SocketSend(Int64, arrayref<Byte>, Int32, Int32, fn<Void>)", "ldc.i4 60000\ncall neoCLR.Runtime.SocketDeadlineAfter(Int32)\ndup\ncall neoCLR.Runtime.SocketDeadlineExpired(Int64)\npop\nldarg 0\nfunction.bind fn<Void> = instance Completion::Complete()\ncall neoCLR.Runtime.SocketSendUntil(Int64, arrayref<Byte>, Int32, Int32, Int64, fn<Void>)");
    let source = format!("{source}\n{DEADLINE_SERVICES}");
    let r = compile_source(&dir, &seed, &source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "socket-listener.h"
#include "native-gc.h"
#include <arpa/inet.h>
#include <sys/socket.h>
#include <unistd.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[513] = {0}; buffer[512] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 4096, 0}};
    for (int cancel = 0; cancel < 1; cancel++) {
        neoclr_socket_scope scope;
        CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
        int32_t port = -1;
        CHECK(!neoclr_entry_v4(cancel, &port, &c) && port > 0);
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t root = 999;
        int peer = -1;
        if (!cancel) {
            CHECK(!neoclr_socket_poll_v1(&c, &root) && root == 999);
            struct sockaddr_in endpoint = {0};
            endpoint.sin_family = AF_INET; endpoint.sin_port = htons((uint16_t)port);
            endpoint.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
            peer = socket(AF_INET, SOCK_STREAM, 0);
            CHECK(peer >= 0 && !connect(peer, (const void *)&endpoint, sizeof(endpoint)));
        }
        int ready = 0;
        for (unsigned retry = 0; retry < 1000 && !ready; retry++) ready = neoclr_socket_poll_v1(&c, &root);
        CHECK(ready == 1);
        CHECK(!neoclr_invoke_void_callback_v1(root, &c) && !c.fault.code);
        CHECK(send(peer, "x", 1, 0) == 1);
        for (unsigned stage = 0; stage < 2; stage++) {
            CHECK(!neoclr_gc_collect_v1(&c, NULL));
            ready = 0;
            for (unsigned retry = 0; retry < 1000 && !ready; retry++) ready = neoclr_socket_poll_v1(&c, &root);
            CHECK(ready == 1 && !neoclr_invoke_void_callback_v1(root, &c) && !c.fault.code);
        }
        char echoed = 0;
        CHECK(recv(peer, &echoed, 1, 0) == 1 && echoed == 'x');
        CHECK(!neoclr_root_probe_depth_v1() && !neoclr_socket_poll_v1(&c, &root));
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        unsigned sockets = 0;
        for (unsigned i = 0; i < 64; i++) {
            CHECK(!scope.operations[i].state);
            sockets += scope.slots[i].id != 0;
        }
        CHECK(sockets == 1); /* Guest callback closed accepted connection. */
        CHECK(!neoclr_socket_scope_leave_v1(&scope));
        if (peer >= 0) close(peer);
    }
    CHECK(buffer[512] == 1234567);
    return 0;
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("socket-listener.c")).arg(base.join("root-probe.c"))
        .arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host"))
        .output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
}

const DEADLINE_SERVICES: &str = r#"
.function neoCLR.Runtime.SocketDeadlineAfter(Int32 milliseconds) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.SocketDeadlineExpired(Int64 stamp) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.SocketReceiveUntil(Int64 socket, arrayref<Byte> buffer, Int32 offset, Int32 count, Int64 deadline, fn<Void> callback) -> Value
.methodimpl InternalCall
.end
.function neoCLR.Runtime.SocketSendUntil(Int64 socket, arrayref<Byte> buffer, Int32 offset, Int32 count, Int64 deadline, fn<Void> callback) -> Value
.methodimpl InternalCall
.end
"#;

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_stack_budget_faults_unwinds_and_preserves_host_callback_roots() {
    for indirect in [false, true] {
    let dir = Temp::new();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/native-stack.neoil");
    let source = if indirect {
        source.replace("ldarg depth\nldc.i4 1\nsub\ncall Recurse(Int32)",
            "function.bind fn<Int32,Int32> = Recurse(Int32)\nldarg depth\nldc.i4 1\nsub\ncall instance fn<Int32,Int32>::Invoke(Int32)")
    } else { source.to_owned() };
    let source = source.as_str();
    let flags = ["--compile-system", "--reference-arena", "--native-gc", "--native-stack-budget"];
    let r = compile_source(&dir, &seed, source, &flags, false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#include "native-stack.h"
#include <assert.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
extern int32_t neoclr_invoke_void_callback_v1(uint64_t, neoclr_aot_context *);
static void check_fault(neoclr_aot_context *ctx, int traced) {
    assert(ctx->fault.code == 9 && ctx->fault.message);
    const char *message = "Call stack limit exceeded";
    assert(ctx->fault.message->length == strlen(message));
    assert(!memcmp(ctx->fault.message->bytes, message, strlen(message)));
    assert(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
    assert(ctx->fault.frame_count == (traced ? 64u : 0u));
    assert(ctx->fault.truncated == (unsigned)traced);
    assert(!neoclr_aot_render_fault(stderr, &ctx->fault));
}
__attribute__((noinline)) static int32_t enter_callback_near_limit(uint64_t handle, neoclr_aot_context *ctx) {
    volatile unsigned char storage[16384]; storage[0] = 17;
    int32_t status = neoclr_native_stack_check_v1() == 0
        ? enter_callback_near_limit(handle, ctx)
        : neoclr_invoke_void_callback_v1(handle, ctx);
    assert(storage[0] == 17);
    return status;
}
static void *run(void *small) {
    uint64_t *heap = calloc(8193, sizeof(uint64_t)); assert(heap);
    heap[8192] = 1234567;
    neoclr_aot_context ctx = {.text = {(unsigned char *)heap, 65536, 0}};
    int32_t result = -99;
    if (small) {
        assert(neoclr_entry_v4(0, &result, &ctx) == 9 && result == -99);
        check_fault(&ctx, 0);
    } else {
        assert(!neoclr_entry_v4(5, &result, &ctx) && result == 42);
        result = -99;
        assert(neoclr_entry_v4(-1, &result, &ctx) == 9 && result == -99);
        check_fault(&ctx, 1);
        assert(!neoclr_gc_collect_v1(&ctx, NULL) && !ctx.text.used);
        assert(!neoclr_entry_v4(-2, &result, &ctx) && result == 0);
        void *callback = (unsigned char *)heap + 32;
        assert(*(uint64_t *)callback == UINT32_MAX);
        uint64_t handle = 0;
        assert(!neoclr_gc_host_root_create_v1(&ctx, callback, &handle));
        assert(neoclr_invoke_void_callback_v1(handle, &ctx) == 9);
        check_fault(&ctx, 1);
        assert(!neoclr_gc_collect_v1(&ctx, NULL) && ctx.text.used);
        void *retained = NULL;
        assert(!neoclr_gc_host_root_read_v1(&ctx, handle, &retained) && retained == callback);
        memset(&ctx.fault, 0, sizeof(ctx.fault));
        assert(enter_callback_near_limit(handle, &ctx) == 9);
        check_fault(&ctx, 0);
        assert(!neoclr_gc_host_root_read_v1(&ctx, handle, &retained) && retained == callback);
        assert(!neoclr_gc_host_root_release_v1(&ctx, handle));
        assert(!neoclr_gc_collect_v1(&ctx, NULL) && !ctx.text.used);
        assert(!neoclr_entry_v4(5, &result, &ctx) && result == 42 && !ctx.fault.code);
    }
    assert(heap[8192] == 1234567);
    free(heap);
    return NULL;
}
int main(void) {
    pthread_attr_t attr; pthread_t thread;
    assert(!pthread_attr_init(&attr));
    assert(!pthread_attr_setstacksize(&attr, 512 * 1024));
    assert(!pthread_create(&thread, &attr, run, NULL));
    assert(!pthread_join(thread, NULL));
    assert(!pthread_attr_setstacksize(&attr, 128 * 1024));
    assert(!pthread_create(&thread, &attr, run, (void *)1));
    assert(!pthread_join(thread, NULL));
    assert(!pthread_attr_destroy(&attr));
}
"#).unwrap();
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c"))
        .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
        .arg(base.join("native-stack.c")).arg(base.join("../aot-fault-details/render.c")).arg(dir.0.join("app.o"))
        .arg("-o").arg(dir.0.join("host")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
    assert!(r.status.success(), "{r:?}");
    assert!(String::from_utf8_lossy(&r.stderr).contains("StackOverflow: Call stack limit exceeded"));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    assert_eq!(method.invoke(vec![neoclr::Value::Int32(5)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    let fault = method.invoke(vec![neoclr::Value::Int32(-1)], neoclr::Limits {frames: 128, ..Default::default()}).unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::StackOverflow);
    assert!(fault.diagnostic().to_string().contains("Call stack limit exceeded"));
}
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_unit_entry_returns_zero_only_on_success() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for (result_type, result_instruction) in [("noresult", ""), ("Void", "ldvoid") ] {
        for arguments in [false, true] {
            let dir = Temp::new();
            let source = format!(".module Entry\n.function Calculate({}) -> {result_type}\n{}\n{result_instruction}\nret\n.end", if arguments { "Int32 mode" } else { "" },
                if arguments { "ldc.i4 42\nldarg mode\ndiv\npop" } else { "" });
            let r = compile_source(&dir, &seed, &source,
                &["--compile-system", "--reference-arena", "--native-gc", "--native-stack-budget"], false);
            assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
            fs::write(dir.0.join("host.c"), format!(r#"
#include "native-gc.h"
#include <assert.h>
int main(void) {{
    uint64_t heap[129]; heap[128] = 1234567;
    neoclr_aot_context ctx = {{.text = {{(unsigned char *)heap, 1024, 0}}}};
    int32_t result = -99;
    assert(!neoclr_entry_v4(1, &result, &ctx) && result == 0);
    result = -99;
    int status = neoclr_entry_v4(0, &result, &ctx);
    assert(status == {status} && result == {result});
    assert(ctx.fault.code == {status});
    assert(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
    assert(!neoclr_gc_collect_v1(&ctx, NULL) && !ctx.text.used);
    assert(heap[128] == 1234567);
}}
"#, status=if arguments {1} else {0}, result=if arguments {-99} else {0})).unwrap();
            let r = Command::new("clang")
                .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
                .arg(&base).arg(dir.0.join("host.c"))
                .arg(base.join("root-probe.c")).arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
                .arg(base.join("native-stack.c")).arg(dir.0.join("app.o"))
                .arg("-o").arg(dir.0.join("host")).output().unwrap();
            assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
            let r = Command::new(dir.0.join("host")).env_clear().output().unwrap();
            assert!(r.status.success(), "{r:?}");
        }
    }
}


#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_scalar_arrays_preserve_types_defaults_roots_and_faults() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for (ty, value, zero) in [
        ("Int32", "ldc.i4 -2147483648", "ldc.i4 0"),
        ("UInt32", "ldc.i4 -1\nconv.u4", "ldc.i4 0\nconv.u4"),
        ("Int64", "ldc.i8 -9223372036854775808", "ldc.i8 0"),
        ("UInt64", "ldc.i8 -1\nconv.u8", "ldc.i8 0\nconv.u8"),
        ("SByte", "ldc.i4 255\nconv.i1", "ldc.i4 0\nconv.i1"),
        ("Int16", "ldc.i4 65535\nconv.i2", "ldc.i4 0\nconv.i2"),
        ("UInt16", "ldc.i4 -1\nconv.u2", "ldc.i4 0\nconv.u2"),
        ("Boolean", "ldc.bool true", "ldc.bool false"),
        ("Void", "ldvoid", "ldvoid"),
        ("IntPtr", "ldc.i4 -42\nconv.i", "ldc.i4 0\nconv.i"),
        ("UIntPtr", "ldc.i4 42\nconv.u", "ldc.i4 0\nconv.u"),
    ] {
        let check = |expected: &str| match ty {
            "Void" => "pop".to_owned(),
            "IntPtr" | "UIntPtr" => format!("conv.i8\n{expected}\nconv.i8\nceq\nbrfalse Failed"),
            _ => format!("{expected}\nceq\nbrfalse Failed"),
        };
        let source = format!(r#"
.module ScalarArrays
.function Calculate(Int32 mode) -> Int32
.local arrayref<{ty}> values
.local Int32 count
ldc.i4 2
newarr {ty}
stloc values
ldloc values
ldc.i4 0
ldelem {ty}
{default_check}
ldarg mode
ldc.i4 1
ceq
brtrue Uninitialized
ldarg mode
ldc.i4 2
ceq
brtrue NegativeIndex
ldarg mode
ldc.i4 3
ceq
brtrue EndIndex
ldarg mode
ldc.i4 4
ceq
brtrue Null
ldarg mode
ldc.i4 5
ceq
brtrue NegativeLength
ldarg mode
ldc.i4 6
ceq
brtrue TooLarge
ldc.i4 2
array.reserve {ty}
stloc values
ldloc values
ldc.i4 1
{value}
stelem {ty}
ldc.i4 0
stloc count
Loop:
ldc.i4 8
newarr {ty}
pop
ldloc count
ldc.i4 1
add
stloc count
ldloc count
ldc.i4 100
clt
brtrue Loop
ldloc values
ldc.i4 1
ldelem {ty}
{written_check}
ldc.i4 42
ret
Uninitialized:
ldc.i4 1
array.reserve {ty}
ldc.i4 0
ldelem {ty}
pop
br Failed
NegativeIndex:
ldloc values
ldc.i4 -1
ldelem {ty}
pop
br Failed
EndIndex:
ldloc values
ldc.i4 2
{value}
stelem {ty}
br Failed
Null:
ldloca values
initobj arrayref<{ty}>
ldloc values
ldc.i4 0
ldelem {ty}
pop
br Failed
NegativeLength:
ldc.i4 -1
newarr {ty}
pop
br Failed
TooLarge:
ldc.i4 65537
array.reserve {ty}
pop
Failed:
ldc.i4 -1
ret
.end
"#, default_check=check(zero), written_check=check(value));
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--native-gc"], false);
        assert!(r.status.success(), "{ty}: {}", String::from_utf8_lossy(&r.stderr));
        fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
int main(void) {
    uint64_t buffer[257]; buffer[256] = 1234567;
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 2048, 0}};
    int statuses[] = {0, 3, 8, 8, 6, 3, 7};
    for (int mode = 0; mode < 7; mode++) {
        int32_t result = -99;
        int status = neoclr_entry_v4(mode, &result, &ctx);
        if (status != statuses[mode] || result != (mode ? -99 : 42)) return 1;
        if (status && neoclr_aot_render_fault(stderr, &ctx.fault)) return 2;
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 3;
        if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 4;
        if (buffer[256] != 1234567) return 5;
    }
    return 0;
}
"#).unwrap();
        let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
            .arg(&base).arg(dir.0.join("host.c")).arg(base.join("root-probe.c"))
            .arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
            .arg(base.join("../aot-fault-details/render.c"))
            .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("host")).output().unwrap();
        assert!(r.status.success(), "{ty}: {}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(dir.0.join("host")).output().unwrap();
        assert!(r.status.success(), "{ty}: {r:?}");
        let app = neoclr::assemble(&source).unwrap();
        let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
        let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
        assert_eq!(method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
        let mut faults = String::new();
        for mode in 1..7 {
            let fault = method.invoke(vec![neoclr::Value::Int32(mode)], neoclr::Limits::default()).unwrap_err();
            faults.push_str(&fault.diagnostic().to_string());
        }
        assert_eq!(String::from_utf8_lossy(&r.stderr), faults, "{ty}");
    }
}

#[test]
fn native_scalar_arrays_reject_element_borrows_and_incompatible_elements() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    for body in [
        "ldc.i4 1\nnewarr Int32\nldc.i4 0\nldelema Int32\npop",
        "ldc.i4 1\nnewarr Int32\nldc.i4 0\nldelem UInt32\npop",
        "ldc.i4 1\nnewarr Int64\nldc.i4 0\nldelem UInt64\npop",
    ] {
        let dir = Temp::new();
        let source = format!(".module BadArrays\n.function Calculate() -> Int32\n{body}\nldc.i4 0\nret\n.end");
        let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--native-gc"], false);
        assert!(!r.status.success(), "unsupported scalar array contract admitted");
    }
}


#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_wide_ordering_matches_signed_unsigned_and_comparison_branches() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let dir = Temp::new();
    let mut source = String::from(".module WideOrdering\n");
    let mut count = 0;
    for (left, right) in [(i64::MIN, i64::MAX), (-1, 1), (0, 0), (i64::MAX, i64::MIN)] {
        for (op, expected, branch) in [
            ("clt", left < right, false), ("cgt", left > right, false),
            ("clt.un", (left as u64) < right as u64, false), ("cgt.un", (left as u64) > right as u64, false),
            ("beq", left == right, true), ("bne.un", left != right, true),
            ("blt", left < right, true), ("bgt", left > right, true),
            ("ble", left <= right, true), ("bge", left >= right, true),
            ("blt.un", (left as u64) < right as u64, true), ("bgt.un", (left as u64) > right as u64, true),
            ("ble.un", (left as u64) <= right as u64, true), ("bge.un", (left as u64) >= right as u64, true),
        ] {
            let comparison = if branch { format!("{op} Yes") } else { format!("{op}\nbrtrue Yes") };
            source.push_str(&format!(".function Case{count}() -> Int32\nldc.i8 {left}\nldc.i8 {right}\n{comparison}\nldc.i4 {}\nret\nYes:\nldc.i4 {}\nret\n.end\n", i32::from(!expected), i32::from(expected)));
            count += 1;
        }
    }
    source.push_str(".function Calculate() -> Int32\nldc.i4 0\n");
    for i in 0..count { source.push_str(&format!("call Case{i}()\nadd\n")); }
    source.push_str("ret\n.end\n");
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(base.join("text-host.c")).arg(base.join("text-arena.c"))
        .arg(base.join("../aot-fault-details/render.c")).arg(dir.0.join("app.o"))
        .arg("-o").arg(dir.0.join("host")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(dir.0.join("host")).output().unwrap();
    assert_eq!(r.status.code(), Some(count), "{r:?}");
    assert!(r.stdout.is_empty() && r.stderr.is_empty());
    let app = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate()").unwrap()).unwrap();
    assert_eq!(method.invoke(vec![], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(count));
}


#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn primitive_instance_projection_preserves_borrowed_storage() {
    use neoclr::metadata::{Instruction as Op, Type};
    let template = neoclr::assemble(".module Template\n.type Owner\n.field Number Int32\n.method instance byref Set(Int32 value) -> noresult\nret\n.end\n.end").unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for (ty, name, value) in [
        (Type::Boolean, "Boolean", "ldc.bool true"),
        (Type::Int32, "Int32", "ldc.i4 -2147483648"),
        (Type::Int64, "Int64", "ldc.i8 -9223372036854775808"),
        (Type::UInt64, "UInt64", "ldc.i8 -1\nconv.u8"),
    ] {
        let mut seed = neoclr::library::system().unwrap().clone();
        let mut method = template.functions[0].clone();
        method.name = format!("System.{name}.AotSet");
        method.owner = Some(ty.clone());
        method.parameters = vec![ty.clone()];
        method.definition = None;
        method.body = vec![Op::Arg(0), Op::Arg(1), Op::StoreObject(ty), Op::Return];
        seed.functions.push(method);
        let source = format!(".module PrimitiveBorrow\n.function Calculate() -> Int32\n.local {name} value\nldloca value\ninitobj {name}\nldloca value\n{value}\ncall instance System.{name}::AotSet({name})\nldloc value\n{value}\nceq\nbrfalse Bad\nldc.i4 42\nret\nBad:\nldc.i4 -1\nret\n.end");
        let dir = Temp::new();
        let app = neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&source)], &seed).unwrap().remove(0);
        let r = compile_linked_module(&dir, &seed, &app, &["--compile-system", "--reference-arena"]);
        assert!(r.status.success(), "{name}: {}", String::from_utf8_lossy(&r.stderr));
        let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
        assert_eq!(report["primitiveInstanceProjections"].as_array().unwrap().len(), 1);
        assert_eq!(report["primitiveInstanceProjections"][0]["receiverType"], name);
        let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-fsanitize=undefined,bounds", "-I"])
            .arg(&base).arg(base.join("text-host.c")).arg(base.join("text-arena.c"))
            .arg(base.join("../aot-fault-details/render.c")).arg(dir.0.join("app.o"))
            .arg("-o").arg(dir.0.join("host")).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(dir.0.join("host")).output().unwrap();
        assert_eq!(r.status.code(), Some(42), "{name}: {r:?}");
        let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
        let entry = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate()").unwrap()).unwrap();
        assert_eq!(entry.invoke(vec![], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    }
}


#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_wide_division_preserves_results_and_faults_at_both_widths() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for wide in [false, true] {
        let (ty, load, min, max) = if wide { ("Int64", "ldc.i8", i64::MIN, i64::MAX) }
            else { ("Int32", "ldc.i4", i32::MIN as i64, i32::MAX as i64) };
        let mut source = String::from(".module Division\n");
        let mut modes = vec![];
        for (left, right) in [(min, -1), (min, 0), (-1, 2), (0, 0), (max, 1),
                              (i32::MIN as i64, -1), (-7, 3), (7, -3), (min, max)] {
            for op in ["div", "div.un", "rem", "rem.un"] {
                let expected = if wide {
                    match op {
                        "div" => left.checked_div(right), "rem" => left.checked_rem(right),
                        "div.un" => (left as u64).checked_div(right as u64).map(|v| v as i64),
                        _ => (left as u64).checked_rem(right as u64).map(|v| v as i64),
                    }
                } else {
                    match op {
                        "div" => (left as i32).checked_div(right as i32).map(i64::from),
                        "rem" => (left as i32).checked_rem(right as i32).map(i64::from),
                        "div.un" => (left as u32).checked_div(right as u32).map(|v| v as i32 as i64),
                        _ => (left as u32).checked_rem(right as u32).map(|v| v as i32 as i64),
                    }
                };
                let n = modes.len();
                source.push_str(&format!(".function Case{n}() -> {ty}\n{load} {left}\n{load} {right}\n{op}\nret\n.end\n"));
                modes.push(expected);
            }
        }
        source.push_str(".function Calculate(Int32 mode) -> Int32\n");
        for n in 0..modes.len() {
            source.push_str(&format!("ldarg mode\nldc.i4 {n}\nceq\nbrtrue Test{n}\n"));
        }
        source.push_str("ldc.i4 99\nret\n");
        for (n, expected) in modes.iter().enumerate() {
            source.push_str(&format!("Test{n}:\ncall Case{n}()\n{load} {}\nceq\nbrfalse Bad\nldc.i4 42\nret\n", expected.unwrap_or(0)));
        }
        source.push_str("Bad:\nldc.i4 99\nret\n.end\n");
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--native-gc"], false);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t buffer[128];
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, sizeof(buffer), 0}};
    int32_t result = -99;
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 90;
    if (status) {
        if (result != -99 || ctx.fault.code != (uint32_t)status) return 91;
        if (neoclr_aot_render_fault(stderr, &ctx.fault)) return 92;
    }
    if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 93;
    return status ? 1 : result;
}
"#).unwrap();
        let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-fsanitize=undefined,bounds", "-DNEOCLR_NATIVE_GC", "-I"])
            .arg(&base).arg(dir.0.join("host.c")).arg(base.join("root-probe.c"))
            .arg(base.join("native-gc.c")).arg(base.join("text-arena.c"))
            .arg(base.join("../aot-fault-details/render.c")).arg(dir.0.join("app.o"))
            .arg("-o").arg(dir.0.join("host")).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let app = neoclr::assemble(&source).unwrap();
        let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
        let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
        for (n, expected) in modes.iter().enumerate() {
            let vm = method.invoke(vec![neoclr::Value::Int32(n as i32)], neoclr::Limits::default());
            let native = Command::new(dir.0.join("host")).arg(n.to_string()).output().unwrap();
            assert!(native.stdout.is_empty());
            if expected.is_some() {
                assert_eq!(vm.unwrap().value, neoclr::Value::Int32(42));
                assert_eq!(native.status.code(), Some(42), "{ty} mode {n}: {native:?}");
                assert!(native.stderr.is_empty());
            } else {
                assert_eq!(native.status.code(), Some(1), "{ty} mode {n}: {native:?}");
                assert_eq!(String::from_utf8_lossy(&native.stderr), vm.unwrap_err().diagnostic().to_string());
            }
        }
    }
}

const PATH_SEED: &str = r#"
.module System
.references ()
.function neoCLR.Runtime.PathCombine(String, String) -> String
.methodimpl InternalCall
.end
.function neoCLR.Runtime.PathGetFileName(String) -> String
.methodimpl InternalCall
.end
.function neoCLR.Runtime.StringCompareOrdinal(String, String) -> Int32
.methodimpl InternalCall
.end
"#;

#[test]
fn lexical_paths_require_exact_opt_in_contracts() {
    let source = ".module Paths\n.function Calculate(Int32 mode) -> Int32\nldstr \"left\"\nldstr \"right\"\ncall neoCLR.Runtime.PathCombine(String, String)\ncall neoCLR.Runtime.PathGetFileName(String)\npop\nldc.i4 0\nret\n.end";
    let seed = neoclr::assemble(PATH_SEED).unwrap();
    for flags in [
        vec!["--compile-system", "--reference-arena"],
        vec!["--bind-paths"],
        vec!["--compile-system", "--bind-paths"],
        vec!["--compile-system", "--reference-arena", "--bind-paths", "--bind-paths"],
    ] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(), "{r:?}");
    }
    for index in 0..2 {
        let mut impostor = seed.clone();
        impostor.functions[index].impl_flags = 0;
        impostor.functions[index].body = vec![neoclr::metadata::Instruction::String("managed".into()), neoclr::metadata::Instruction::Return];
        let dir = Temp::new();
        let r = compile_source(&dir, &impostor, source, &["--compile-system", "--reference-arena", "--bind-paths"], false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("native paths require exact"), "{r:?}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn lexical_paths_match_interpreter_bytes_faults_and_gc_ownership() {
    let combined = [
        ("", "", ""), ("", "leaf", "leaf"), ("left", "", "left"),
        ("left", "leaf", "left/leaf"), ("left/", "leaf", "left/leaf"),
        ("left//", "leaf", "left//leaf"), ("left", "/leaf", "/leaf"),
        ("left", "//leaf", "//leaf"), ("/", "leaf", "/leaf"),
        ("reports/..", "leaf", "reports/../leaf"), (".", "..", "./.."),
        ("資料", "世界.txt", "資料/世界.txt"), ("a\0b", "c\0d", "a\0b/c\0d"),
        ("a\\b", "c\\d", "a\\b/c\\d"), ("left", "C:\\leaf", "left/C:\\leaf"),
    ];
    let names = [
        ("", ""), ("leaf", "leaf"), ("reports/leaf", "leaf"),
        ("reports/", ""), ("/", ""), ("//", ""), ("/leaf", "leaf"),
        ("a//leaf", "leaf"), ("a/..", ".."), ("a/.", "."),
        ("資料/世界.txt", "世界.txt"), ("a/\0b", "\0b"),
        ("a\\b", "a\\b"), ("C:leaf", "C:leaf"),
    ];
    let literal = |text: &str| format!("ldstr {}\n", serde_json::to_string(text).unwrap());
    let mut cases = vec![];
    for (left, right, expected) in combined {
        cases.push(format!("{}{}call neoCLR.Runtime.PathCombine(String, String)\n{}call neoCLR.Runtime.StringCompareOrdinal(String, String)\nret\n", literal(left), literal(right), literal(expected)));
    }
    for (path, expected) in names {
        cases.push(format!("{}call neoCLR.Runtime.PathGetFileName(String)\n{}call neoCLR.Runtime.StringCompareOrdinal(String, String)\nret\n", literal(path), literal(expected)));
    }
    cases.push("ldstr \"reports\"\nldstr \"nested\"\ncall neoCLR.Runtime.PathCombine(String, String)\nldstr \"leaf\"\ncall neoCLR.Runtime.PathCombine(String, String)\ncall neoCLR.Runtime.PathGetFileName(String)\nldstr \"leaf\"\ncall neoCLR.Runtime.StringCompareOrdinal(String, String)\nret\n".into());
    let successes = cases.len();
    cases.push("ldloca empty\ninitobj String\nldloc empty\ncall neoCLR.Runtime.PathGetFileName(String)\npop\nldc.i4 -1\nret\n".into());
    cases.push("ldloca empty\ninitobj String\nldloc empty\nldstr \"leaf\"\ncall neoCLR.Runtime.PathCombine(String, String)\npop\nldc.i4 -1\nret\n".into());
    cases.push("ldloca empty\ninitobj String\nldstr \"left\"\nldloc empty\ncall neoCLR.Runtime.PathCombine(String, String)\npop\nldc.i4 -1\nret\n".into());
    let mut source = String::from(".module Paths\n.function Calculate(Int32 mode) -> Int32\n.local String empty\n");
    for i in 0..cases.len() { source += &format!("ldarg mode\nldc.i4 {i}\nbeq Case{i}\n"); }
    source += "ldc.i4 -1\nret\n";
    for (i, body) in cases.iter().enumerate() { source += &format!("Case{i}:\n{body}"); }
    source += ".end\n";
    let dir = Temp::new();
    let seed = neoclr::assemble(PATH_SEED).unwrap();
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--native-gc", "--bind-paths", "--bind-utf8-text"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[257]; storage[256] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)storage, 2048, 0}};
    int32_t result = -99;
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 92;
    if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 93;
    if (storage[256] != UINT64_C(0x1122334455667788)) return 94;
    if (status) { if (result != -99) return 95; neoclr_aot_render_fault(stderr, &ctx.fault); return 1; }
    printf("%d\n", result); return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c")).arg(base.join("text-arena.c"))
        .arg(base.join("native-gc.c")).arg(base.join("root-probe.c"))
        .arg(base.join("../aot-fault-details/render.c")).arg(dir.0.join("app.o"))
        .arg("-o").arg(dir.0.join("host")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for i in 0..cases.len() {
        let expected = method.invoke(vec![neoclr::Value::Int32(i as i32)], neoclr::Limits::default());
        let r = Command::new(dir.0.join("host")).arg(i.to_string()).env_clear().output().unwrap();
        if i < successes {
            assert_eq!(expected.unwrap().value, neoclr::Value::Int32(0), "case {i}");
            assert_eq!(r.status.code(), Some(0), "case {i}: {r:?}");
            assert_eq!(r.stdout, b"0\n"); assert!(r.stderr.is_empty());
        } else {
            let fault = expected.unwrap_err();
            assert_eq!(r.status.code(), Some(1), "case {i}: {r:?}");
            assert_eq!(String::from_utf8_lossy(&r.stderr), fault.diagnostic().to_string());
        }
    }
}

const FILE_OUTPUT_SEED: &str = ".module System\n.references ()\n.function neoCLR.Runtime.WriteAllText(String, String, Int32) -> Int32\n.methodimpl InternalCall\n.end";

#[test]
fn file_output_requires_exact_opt_in_contract() {
    let source = ".module FileOutput\n.function Calculate(Int32 mode) -> Int32\nldstr \"unused\"\nldstr \"text\"\nldc.i4 4\ncall neoCLR.Runtime.WriteAllText(String, String, Int32)\nret\n.end";
    let seed = neoclr::assemble(FILE_OUTPUT_SEED).unwrap();
    for flags in [vec!["--compile-system", "--reference-arena"], vec!["--bind-file-output"],
        vec!["--compile-system", "--bind-file-output"],
        vec!["--compile-system", "--reference-arena", "--bind-file-output", "--bind-file-output"]] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(), "{r:?}");
    }
    let mut impostor = seed.clone();
    impostor.functions[0].impl_flags = 0;
    impostor.functions[0].body = vec![neoclr::metadata::Instruction::Int(0), neoclr::metadata::Instruction::Return];
    let dir = Temp::new();
    let r = compile_source(&dir, &impostor, source, &["--compile-system", "--reference-arena", "--bind-file-output"], false);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("native file output requires exact"));
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn file_output_matches_interpreter_and_preserves_preflight_files() {
    let dir = Temp::new();
    let destination = dir.0.join("世界.txt");
    let symlink = dir.0.join("link.txt");
    std::os::unix::fs::symlink(&destination, &symlink).unwrap();
    let regular = destination.to_str().unwrap();
    let missing = dir.0.join("missing/leaf.txt");
    let cases = [
        (regular, "Hello, värld!", 14, 0, "Hello, värld!"),
        (regular, "a\0b", 3, 0, "a\0b"),
        (regular, "", 0, 0, ""),
        (regular, "replacement", 2, 7, "original"),
        (regular, "", -1, 1, "original"),
        ("", "", -1, 1, "original"),
        ("", "", 0, 2, "original"),
        ("bad\0path", "", 0, 2, "original"),
        (missing.to_str().unwrap(), "x", 1, 3, "original"),
        (dir.0.to_str().unwrap(), "x", 1, 5, "original"),
        (symlink.to_str().unwrap(), "followed", 8, 0, "followed"),
    ];
    let mut source = String::from(".module FileOutput\n.function Calculate(Int32 mode) -> Int32\n");
    for i in 0..cases.len() { source += &format!("ldarg mode\nldc.i4 {i}\nbeq Case{i}\n"); }
    source += "ldc.i4 -1\nret\n";
    for (i, (path, text, limit, _, _)) in cases.iter().enumerate() {
        source += &format!("Case{i}:\nldstr {}\nldstr {}\nldc.i4 {limit}\ncall neoCLR.Runtime.WriteAllText(String, String, Int32)\nret\n", serde_json::to_string(path).unwrap(), serde_json::to_string(text).unwrap());
    }
    source += ".end\n";
    let seed = neoclr::assemble(FILE_OUTPUT_SEED).unwrap();
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--native-gc", "--bind-file-output"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#include "file-output.h"
#include <stdlib.h>
#include <fcntl.h>
static int open_count(void) {
    int count = 0;
    for (int fd = 0; fd < 256; fd++) if (fcntl(fd, F_GETFD) >= 0) count++;
    return count;
}
int main(int argc, char **argv) {
    uint64_t storage[257]; storage[256] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)storage, 2048, 0}};
    int32_t result = -99;
    if (neoclr_file_write_utf8_v1(NULL, NULL, 0, &result) != 3 || result != -99) return 91;
    int descriptors = open_count();
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (open_count() != descriptors) return 96;
    if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 92;
    if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 93;
    if (storage[256] != UINT64_C(0x1122334455667788)) return 94;
    if (status) { neoclr_aot_render_fault(stderr, &ctx.fault); return 1; }
    printf("%d\n", result); return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c")).arg(base.join("file-output.c")).arg(base.join("text-arena.c"))
        .arg(base.join("native-gc.c")).arg(base.join("root-probe.c"))
        .arg(base.join("../aot-fault-details/render.c")).arg(dir.0.join("app.o"))
        .arg("-o").arg(dir.0.join("host")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for (i, (_, _, _, status, expected)) in cases.iter().enumerate() {
        fs::write(&destination, "original").unwrap();
        assert_eq!(method.invoke(vec![neoclr::Value::Int32(i as i32)], neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(*status), "case {i}");
        assert_eq!(fs::read(&destination).unwrap(), expected.as_bytes());
        fs::write(&destination, "original").unwrap();
        let r = Command::new(dir.0.join("host")).arg(i.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(), Some(0), "case {i}: {r:?}");
        assert_eq!(r.stdout, format!("{status}\n").as_bytes()); assert!(r.stderr.is_empty());
        assert_eq!(fs::read(&destination).unwrap(), expected.as_bytes());
    }
}

const FILE_INPUT_SEED: &str = ".module System\n.references ()\n.function neoCLR.Runtime.ReadAllText(String, Int32) -> Value\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.StringCompareOrdinal(String, String) -> Int32\n.methodimpl InternalCall\n.end";

#[test]
fn file_input_requires_exact_opt_in_contract() {
    let source = ".module FileInput\n.function Calculate(Int32 mode) -> Int32\nldstr \"unused\"\nldc.i4 4\ncall neoCLR.Runtime.ReadAllText(String, Int32)\npop\nldc.i4 0\nret\n.end";
    let seed = neoclr::assemble(FILE_INPUT_SEED).unwrap();
    for flags in [vec!["--compile-system", "--reference-arena"], vec!["--bind-file-input"],
        vec!["--compile-system", "--bind-file-input"],
        vec!["--compile-system", "--reference-arena", "--bind-file-input", "--bind-file-input"]] {
        let dir = Temp::new();
        let r = compile_source(&dir, &seed, source, &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(), "{r:?}");
    }
    let mut impostor = seed.clone();
    impostor.functions[0].impl_flags = 0;
    impostor.functions[0].body = vec![neoclr::metadata::Instruction::Void, neoclr::metadata::Instruction::PackValue(neoclr::metadata::Type::Void), neoclr::metadata::Instruction::Return];
    let dir = Temp::new();
    let r = compile_source(&dir, &impostor, source, &["--compile-system", "--reference-arena", "--bind-file-input"], false);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("native file input requires exact"));
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn file_input_matches_interpreter_limits_utf8_and_cleanup() {
    let dir = Temp::new();
    let path = |name: &str| dir.0.join(name).to_str().unwrap().to_owned();
    fs::write(path("text"), "Hello, värld!").unwrap();
    fs::write(path("empty"), "").unwrap();
    fs::write(path("nul"), b"a\0b").unwrap();
    fs::write(path("bom"), "\u{feff}text\r\n").unwrap();
    fs::write(path("invalid"), [0xff, 0x61]).unwrap();
    fs::write(path("surrogate"), [0xed, 0xa0, 0x80]).unwrap();
    fs::write(path("large"), "x".repeat(9000)).unwrap();
    std::os::unix::fs::symlink(path("text"), path("link")).unwrap();
    let cases = vec![
        (path("text"), 14, "Hello, värld!".to_owned(), 0),
        (path("link"), 14, "Hello, värld!".to_owned(), 0),
        (path("empty"), 0, "".to_owned(), 0),
        (path("nul"), 3, "a\0b".to_owned(), 0),
        (path("bom"), 9, "\u{feff}text\r\n".to_owned(), 0),
        (path("invalid"), 2, "".to_owned(), 8),
        (path("surrogate"), 3, "".to_owned(), 8),
        (path("invalid"), 1, "".to_owned(), 7),
        (path("text"), 0, "".to_owned(), 7),
        (path("text"), 13, "".to_owned(), 7),
        (path("text"), -1, "".to_owned(), 1),
        ("".to_owned(), -1, "".to_owned(), 1),
        ("".to_owned(), 0, "".to_owned(), 2),
        ("bad\0path".to_owned(), 0, "".to_owned(), 2),
        (path("missing"), 10, "".to_owned(), 3),
        (dir.0.to_str().unwrap().to_owned(), 10, "".to_owned(), 5),
        (path("large"), 9000, "x".repeat(9000), 0),
        (path("large"), 8999, "".to_owned(), 7),
    ];
    let mut source = String::from(".module FileInput\n.function Calculate(Int32 mode) -> Int32\n");
    for i in 0..cases.len() { source += &format!("ldarg mode\nldc.i4 {i}\nbeq Case{i}\n"); }
    source += "ldc.i4 -1\nret\n";
    for (i, (path, limit, text, _)) in cases.iter().enumerate() {
        source += &format!("Case{i}:\nldstr {}\nldc.i4 {limit}\ncall neoCLR.Runtime.ReadAllText(String, Int32)\ndup\nvalue.is Byte\nbrtrue Error{i}\nvalue.unpack String\nldstr {}\ncall neoCLR.Runtime.StringCompareOrdinal(String, String)\nret\nError{i}:\nvalue.unpack Byte\nconv.i4\nret\n", serde_json::to_string(path).unwrap(), serde_json::to_string(text).unwrap());
    }
    source += ".end\n";
    let seed = neoclr::assemble(FILE_INPUT_SEED).unwrap();
    let r = compile_source(&dir, &seed, &source, &["--compile-system", "--reference-arena", "--native-gc", "--bind-file-input", "--bind-utf8-text"], false);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"), r#"
#include "native-gc.h"
#include "file-input.h"
#include <stdlib.h>
#include <fcntl.h>
#include <string.h>
static int open_count(void) {
    int count = 0;
    for (int fd = 0; fd < 256; fd++) if (fcntl(fd, F_GETFD) >= 0) count++;
    return count;
}
int main(int argc, char **argv) {
    uint64_t storage[8193]; storage[8192] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)storage, 65536, 0}};
    if (argc > 2) {
        size_t length = strlen(argv[2]);
        neoclr_aot_text *path = malloc(8 + length);
        if (!path) return 97;
        path->length = length;
        memcpy(path->bytes, argv[2], length);
        neoclr_aot_context tiny = {0};
        uint64_t result[2] = {77, 88};
        int descriptors = open_count();
        int32_t status = neoclr_file_read_utf8_v1(path, 64, &tiny.text, result);
        free(path);
        if (status != 5 || result[0] != 77 || result[1] != 88 || open_count() != descriptors) return 98;
        return 0;
    }
    int32_t result = -99;
    int descriptors = open_count();
    int status = neoclr_entry_v4(argc > 1 ? atoi(argv[1]) : 0, &result, &ctx);
    if (open_count() != descriptors) return 96;
    if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1()) return 92;
    if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used) return 93;
    if (storage[8192] != UINT64_C(0x1122334455667788)) return 94;
    if (status) { neoclr_aot_render_fault(stderr, &ctx.fault); return 1; }
    printf("%d\n", result); return 0;
}
"#).unwrap();
    let r = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-DNEOCLR_NATIVE_GC", "-fsanitize=undefined,bounds", "-I"])
        .arg(&base).arg(dir.0.join("host.c")).arg(base.join("file-input.c")).arg(base.join("text-arena.c"))
        .arg(base.join("native-gc.c")).arg(base.join("root-probe.c"))
        .arg(base.join("../aot-fault-details/render.c")).arg(dir.0.join("app.o"))
        .arg("-o").arg(dir.0.join("host")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let app = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for (i, (_, _, _, status)) in cases.iter().enumerate() {
        assert_eq!(method.invoke(vec![neoclr::Value::Int32(i as i32)], neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(*status), "case {i}");
        let r = Command::new(dir.0.join("host")).arg(i.to_string()).env_clear().output().unwrap();
        assert_eq!(r.status.code(), Some(0), "case {i}: {r:?}");
        assert_eq!(r.stdout, format!("{status}\n").as_bytes()); assert!(r.stderr.is_empty());
    }
    let exhausted = Command::new(dir.0.join("host")).arg("0").arg(path("text")).env_clear().output().unwrap();
    assert!(exhausted.status.success(), "native arena exhaustion must preserve output/close file: {exhausted:?}");

}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn object_display_accepts_string_boxing_with_unboxed_value_construction() {
    let seed = neoclr::assemble(OBJECT_DISPLAY_SEED).unwrap();
    let source = r#"
.module StringBox
.type Marker<T>
.method instance byref .ctor() -> noresult
ret
.end
.end
.function Box<T>(T value) -> System.Object
ldarg value
box T
ret
.end
.function Calculate(Int32 mode) -> Int32
newobj.ctor instance Marker<Int32>::.ctor()
pop
ldstr "hé🙂\u0000z"
call Box<String>(String)
callvirt instance System.Object::ToString()
call neoCLR.Runtime.Fault(String)
pop
ldc.i4 0
ret
.end
"#;
    let app = neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], &seed).unwrap().remove(0);
    let dir = Temp::new();
    let result = compile_linked_module(&dir, &seed, &app, &["--compile-system", "--reference-arena", "--bind-user-fault"]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["stringBoxSites"].as_array().unwrap().len(), 1);
    assert!(report["objectDisplayDispatch"][0]["targets"].as_array().unwrap().is_empty());
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    let result = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror"])
        .arg(base.join("aot-console/text-host.c"))
        .arg(base.join("aot-console/text-arena.c"))
        .arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    let fault = method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap_err();
    let result = Command::new(dir.0.join("app")).env_clear().output().unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&result.stderr), fault.diagnostic().to_string());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn char_box_tests_preserve_null_results_and_reject_matching_producers() {
    let seed = neoclr::assemble(OBJECT_DISPLAY_SEED).unwrap();
    let cases = [
        ("ldloca value\ninitobj System.Object\nldloc value", false),
        ("ldstr \"hé🙂\"\nbox String", false),
        ("newobj.ctor instance A::.ctor()\ncastclass System.Object", false),
        ("ldc.i4 7\nbox Int32", false),
        ("ldloca character\ninitobj Char\nldloc character\nbox Char", true),
    ];
    for (producer, matching) in cases {
        let source = format!(r#"
.module CharBoxTest
.type class A
.method instance .ctor() -> noresult
ret
.end
.end
.function Calculate(Int32 unused) -> Int32
.local System.Object value
.local Char character
{producer}
isinst Char
ref.isnull
brtrue Missing
ldc.i4 1
ret
Missing:
ldc.i4 42
ret
.end
"#);
        let app = neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&source)], &seed).unwrap().remove(0);
        let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
        let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
        assert_eq!(method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(if matching { 1 } else { 42 }));
        let dir = Temp::new();
        let result = compile_linked_module(&dir, &seed, &app, &["--compile-system", "--reference-arena"]);
        if matching {
            assert!(!result.status.success() && !dir.0.join("app.o").exists());
            assert!(String::from_utf8_lossy(&result.stderr).contains("record boxing requires Int32 or a local closed empty value record"));
            continue;
        }
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
        let result = Command::new("clang").args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(base.join("aot-console/text-host.c")).arg(base.join("aot-console/text-arena.c"))
            .arg(base.join("aot-fault-details/render.c")).arg(dir.0.join("app.o"))
            .arg("-o").arg(dir.0.join("app")).output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let result = Command::new(dir.0.join("app")).env_clear().output().unwrap();
        assert_eq!(result.status.code(), Some(42));
        assert!(result.stdout.is_empty() && result.stderr.is_empty());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_replace_ordinal_matches_interpreter_and_checks_admission() {
    let seed = neoclr::assemble(&format!(
        ".module System\n.references ()\n{}\n{}",
        include_str!("../../../runtime/neoCLR/Runtime/StringReplaceOrdinal.neoil"),
        include_str!("../../../runtime/neoCLR/Runtime/StringCompareOrdinal.neoil")
    ))
    .unwrap();
    let cases = [
        ("", "a", "x", ""),
        ("abc", "z", "x", "abc"),
        ("abc", "a", "a", "abc"),
        ("aaaaa", "aa", "X", "XXa"),
        ("a", "a", "aa", "aa"),
        ("banana", "na", "", "ba"),
        ("hé🙂\0hé", "hé", "é", "é🙂\0é"),
        ("a\0b\0", "\0", "🙂", "a🙂b🙂"),
        ("a\"b\\c", "\\", "\\\\", "a\"b\\\\c"),
        ("a\"b", "\"", "\\\"", "a\\\"b"),
        ("abc", "abcd", "", "abc"),
        ("abc", "", "x", ""),
    ];
    for (text, old, new, expected) in cases {
        let literal = |s: &str| serde_json::to_string(s).unwrap();
        let source = format!(
            ".module Replace\n.function Calculate(Int32 unused) -> Int32\nldstr {}\nldstr {}\nldstr {}\ncall neoCLR.Runtime.StringReplaceOrdinal(String,String,String)\nldstr {}\ncall neoCLR.Runtime.StringCompareOrdinal(String,String)\nret\n.end\n",
            literal(text),
            literal(old),
            literal(new),
            literal(expected)
        );
        let app = neoclr::assembler::read_modules(
            &[neoclr::assembler::ModuleInput::Source(&source)],
            &seed,
        )
        .unwrap()
        .remove(0);
        let program = neoclr::LoadedProgram::with_library(&app, &seed).unwrap();
        let method = program
            .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
            .unwrap();
        let interpreted = method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default());
        if old.is_empty() {
            assert_eq!(
                interpreted.unwrap_err().code,
                neoclr::FaultCode::RuntimeError
            );
        } else {
            assert_eq!(interpreted.unwrap().value, neoclr::Value::Int32(0));
        }
        let dir = Temp::new();
        let result = compile_linked_module(
            &dir,
            &seed,
            &app,
            &["--compile-system", "--reference-arena", "--bind-utf8-text"],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
        let result = Command::new("clang")
            .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(base.join("aot-console/text-host.c"))
            .arg(base.join("aot-console/text-arena.c"))
            .arg(base.join("aot-fault-details/render.c"))
            .arg(dir.0.join("app.o"))
            .arg("-o")
            .arg(dir.0.join("app"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let result = Command::new(dir.0.join("app"))
            .env_clear()
            .output()
            .unwrap();
        assert_eq!(
            result.status.code(),
            Some(if old.is_empty() { 1 } else { 0 })
        );
        if old.is_empty() {
            assert!(
                String::from_utf8_lossy(&result.stderr).starts_with("RuntimeError: Runtime error")
            );
        } else {
            assert!(result.stdout.is_empty() && result.stderr.is_empty());
        }
        let rejected = Temp::new();
        let result = compile_linked_module(
            &rejected,
            &seed,
            &app,
            &["--compile-system", "--reference-arena"],
        );
        assert!(!result.status.success() && !rejected.0.join("app.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn inherited_display_preserves_base_and_derived_fields_casts_and_null_faults() {
    let dir=Temp::new();
    let seed=neoclr::assemble(OBJECT_DISPLAY_SEED).unwrap();
    let source=include_str!("../../../docs/experiments/aot-console/inherited-display.neoil");
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], &seed).unwrap().remove(0);
    let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena","--bind-user-fault","--bind-utf8-text"]);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc, char **argv) {
    uint64_t storage[128];
    neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;
    int status=neoclr_entry_v4(argc>1 ? atoi(argv[1]) : 0,&result,&ctx);
    if (status) {
        if (result!=-99 || ctx.fault.code!=(uint32_t)status) return 92;
        neoclr_aot_render_fault(stderr,&ctx.fault);return 1;
    }
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-console/text-arena.c")).arg(base.join("aot-fault-details/render.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], &seed).unwrap().remove(0);
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..2 {
        let reference=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match reference {
            Ok(result)=>{
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");
                let neoclr::Value::Int32(value)=result.value else {panic!("expected Int32")};
                assert_eq!(r.stdout,format!("{value}\n").as_bytes());
                assert!(r.stderr.is_empty());
            }
            Err(fault)=>{
                assert_eq!(r.status.code(),Some(1),"{mode}: {r:?}");
                assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");
            }
        }
    }
}

#[test]
fn inherited_display_rejects_unselected_overrides() {
    let seed = neoclr::assemble(OBJECT_DISPLAY_SEED).unwrap();
    let source = include_str!("../../../docs/experiments/aot-console/inherited-display.neoil");
    let source = format!("{source}\n.type class Other\n.extends Base\n.method instance override ToString() -> String\nldstr \"other\"\nret\n.end\n.end\n");
    let dir = Temp::new();
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&source)], &seed).unwrap().remove(0);
    let r = compile_linked_module(&dir, &seed, &app, &["--compile-system", "--reference-arena", "--bind-user-fault"]);
    assert!(String::from_utf8_lossy(&r.stderr).contains("virtual calls requiring dispatch"), "{r:?}");
    assert!(!r.status.success());
    assert!(!dir.0.join("app.o").exists());
}
