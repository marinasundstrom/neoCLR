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
fn sixteen_field_reference_allocation_respects_arena_canaries() {
    let dir=Temp::new();
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let fields=(0..16).map(|i| format!(".field F{i} Int32\n")).collect::<String>();
    let source=format!(".module WideObject\n.type class Wide\n{fields}.method instance .ctor() -> noresult\nldarg 0\nldc.i4 42\nstfld 15\nret\n.end\n.end\n.function Calculate() -> Int32\n.local Wide value\nnewobj.ctor instance Wide::.ctor()\nstloc value\nldloc value\nldfld 0\nldloc value\nldfld 15\nadd\nret\n.end");
    let r=compile_source(&dir,&seed,&source,&["--compile-system","--reference-arena"],false);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <string.h>
int main(void) {
    uint64_t storage[18];memset(storage,0xa5,sizeof(storage));
    neoclr_aot_context ctx={.text={(unsigned char*)storage,136,0}};
    int32_t result=-99;
    if (neoclr_entry_v4(0,&result,&ctx) || result!=42 || ctx.text.used!=136) return 1;
    for (unsigned i=136;i<sizeof(storage);i++) if (((unsigned char*)storage)[i]!=0xa5) return 2;
    ctx.text.capacity=135;result=-99;
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
    let calls=(0..254).map(|n| format!("call F{n}()\npop\n")).collect::<String>();
    let mut large=source.replacen("call Box()",&format!("{calls}call Box()"),1);
    for n in 0..254 {large.push_str(&format!(".function F{n}() -> Int32\nldc.i4 0\nret\n.end\n"));}
    let app=neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&large)],&seed).unwrap().remove(0);
    let dir=Temp::new();let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena"]);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("boxing helpers exceed"),"{r:?}");
}

fn array_views_module() -> neoclr::Module {
    let mut app=neoclr::assemble(include_str!("../../../docs/experiments/aot-console/array-views.neoil")).unwrap();
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
    let dir=Temp::new();let app=array_views_module();
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    let program=neoclr::LoadedProgram::with_library(&app,&seed).unwrap();program.verify().unwrap();
    let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena"]);
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let report:serde_json::Value=serde_json::from_slice(&r.stdout).unwrap();
    assert!(report["arrayBackingProjection"]["compiledIndex"].is_number());
    assert_eq!(report["interfaceDispatch"].as_array().unwrap().len(),3);
    let base=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    fs::write(dir.0.join("host.c"),r#"
#include "text-arena.h"
#include <stdlib.h>
int main(int argc,char **argv) {
    uint64_t storage[16];neoclr_aot_context ctx={.text={(unsigned char*)storage,sizeof(storage),0}};
    int32_t result=-99;int status=neoclr_entry_v4(argc>1?atoi(argv[1]):0,&result,&ctx);
    if(status){if(result!=-99)return 92;neoclr_aot_render_fault(stderr,&ctx.fault);return 1;}
    printf("%d\n",result);return 0;
}
"#).unwrap();
    let r=Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"])
        .arg(base.join("aot-console")).arg(dir.0.join("host.c"))
        .arg(base.join("aot-fault-details/render.c")).arg(base.join("aot-console/text-arena.c"))
        .arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
    let method=program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    for mode in 0..10 {
        let expected=method.invoke(vec![neoclr::Value::Int32(mode)],neoclr::Limits::default());
        let r=Command::new(dir.0.join("app")).arg(mode.to_string()).env_clear().output().unwrap();
        match expected {
            Ok(value)=>{let neoclr::Value::Int32(value)=value.value else{panic!("expected Int32")};
                assert_eq!(r.status.code(),Some(0),"{mode}: {r:?}");assert_eq!(r.stdout,format!("{value}\n").as_bytes());assert!(r.stderr.is_empty());}
            Err(fault)=>{assert_eq!(r.status.code(),Some(1));assert_eq!(String::from_utf8_lossy(&r.stderr),fault.diagnostic().to_string(),"{mode}");}
        }
    }
}

#[test]
fn nominal_byte_array_views_require_verified_backing_and_reject_class_allocation() {
    let seed=neoclr::assemble(".module System\n.references ()\n").unwrap();
    for mode in 0..3 {
        let mut app=array_views_module();
        match mode {
            0=>app.assemblies[0].array_backing=None,
            1=>app.types.iter_mut().find(|t|t.name=="Storage").unwrap().origin.as_mut().unwrap().field_access=vec![neoclr::metadata_origin::SourceAccess::Public],
            _=>{
                let root=app.functions.iter_mut().find(|f|f.name=="Calculate").unwrap();
                root.body=vec![neoclr::metadata::Instruction::Construct(neoclr::assembler::parse_function_ref("instance Storage<Byte>::.ctor()").unwrap()),
                    neoclr::metadata::Instruction::Pop,neoclr::metadata::Instruction::Int(0),neoclr::metadata::Instruction::Return];
            }
        }
        let dir=Temp::new();let r=compile_linked_module(&dir,&seed,&app,&["--compile-system","--reference-arena"]);
        assert!(!r.status.success() && !dir.0.join("app.o").exists(),"mode {mode}: {r:?}");
        let message=String::from_utf8_lossy(&r.stderr);
        assert!(message.contains(match mode {0=>"verified backing",1=>"invalid nominal array backing",_=>"not class construction"}),"{r:?}");
    }
}
