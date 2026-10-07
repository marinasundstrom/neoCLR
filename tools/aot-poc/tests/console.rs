use std::{fs, path::PathBuf, process::Command, sync::atomic::{AtomicUsize, Ordering}};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("aot-console-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); Self(path)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
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
fn compile(dir: &Temp, seed: &neoclr::Module, flags: &[&str], inspect: bool) -> std::process::Output {
    fs::write(dir.0.join("seed.neox"), neoclr::metadata_container::write_module(seed).unwrap()).unwrap();
    fs::write(dir.0.join("app.neoil"), APP).unwrap();
    fs::write(dir.0.join("lib.neoil"), ".module Helpers\n.references ()\n").unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    c.arg(if inspect { "--inspect" } else { "--closed-world" }).arg(dir.0.join("app.neoil")).arg("Calculate");
    if inspect { c.arg("--closed-world"); } else { c.arg(dir.0.join("app.o")); }
    c.arg("--system").arg(dir.0.join("seed.neox")).arg("--module").arg(dir.0.join("lib.neoil")).args(flags).output().unwrap()
}
#[test]
fn input_binding_is_explicit_and_exact() {
    for flags in [vec![], vec!["--compile-system"], vec!["--bind-console-read-byte"], vec!["--compile-system", "--bind-console-read-byte", "--bind-console-read-byte"]] {
        let dir = Temp::new();
        let r = compile(&dir, &neoclr::assemble(SEED).unwrap(), &flags, false);
        assert!(!r.status.success() && !dir.0.join("app.o").exists());
    }
    let mut seed = neoclr::assemble(SEED).unwrap();
    seed.functions[0].impl_flags = 0;
    seed.functions[0].body = vec![neoclr::metadata::Instruction::Void, neoclr::metadata::Instruction::PackValue(neoclr::metadata::Type::Void), neoclr::metadata::Instruction::Return];
    let dir = Temp::new();
    let r = compile(&dir, &seed, &["--compile-system", "--bind-console-read-byte"], false);
    assert!(!r.status.success() && !dir.0.join("app.o").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("exact neoCLR.Runtime.ConsoleReadByte"));
}
#[test]
#[cfg(all(target_os="macos", target_arch="aarch64"))]
fn input_transport_preserves_all_bytes_and_distinct_outcomes() {
    let dir = Temp::new(); let seed = neoclr::assemble(SEED).unwrap();
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
    let r = Command::new("clang").args(["-arch","arm64","-std=c11","-Wall","-Wextra","-Werror","-I"]).arg(base.join("aot-fault-details")).arg("-I").arg(base.join("aot-console")).arg(dir.0.join("host.c")).arg(dir.0.join("app.o")).arg("-o").arg(dir.0.join("app")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(Command::new(dir.0.join("app")).env_clear().status().unwrap().success());
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
    } else { panic!("expected call"); }
    for (name, module) in [("app.neox", &modules[0]), ("lib.neox", &modules[1]), ("seed.neox", &neoclr::assemble(SEED).unwrap())] {
        fs::write(dir.0.join(name), neoclr::metadata_container::write_module(module).unwrap()).unwrap();
    }
    let r = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg("--inspect").arg(dir.0.join("app.neox")).args(["Calculate", "--closed-world", "--compile-system", "--bind-console-read-byte"])
        .arg("--system").arg(dir.0.join("seed.neox")).arg("--module").arg(dir.0.join("lib.neox")).output().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(report["admission"]["accepted"], true, "{report}");
    assert_eq!(report["selection"]["nativeBindings"][0]["definition"]["module"], "ConsoleLibrary");
    assert_eq!(report["selection"]["nativeBindings"].as_array().unwrap().len(), 1);
}
