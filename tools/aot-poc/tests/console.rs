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
