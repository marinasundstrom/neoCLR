use object::{Object, ObjectSymbol};
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
        let p = std::env::temp_dir().join(format!(
            "neoclr-entry-args-{}-{}",
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
const SOURCE: &str = ".module EntryArgs\n.function Calculate(arrayref<String> args) -> Int32\nldarg args\nldlen\nconv.i4\nret\n.end\n";
fn compile(dir: &Temp, source: &str, flags: &[&str]) -> std::process::Output {
    fs::write(dir.0.join("input.neoil"), source).unwrap();
    fs::write(
        dir.0.join("seed.neox"),
        neoclr::metadata_container::write_module(
            &neoclr::assemble(".module System\n.references ()\n").unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    fs::write(
        dir.0.join("library.neoil"),
        ".module Helpers\n.references ()\n",
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg("--closed-world")
        .arg(dir.0.join("input.neoil"))
        .arg("Calculate")
        .arg(dir.0.join("app.o"))
        .arg("--compile-system")
        .arg("--system")
        .arg(dir.0.join("seed.neox"))
        .arg("--module")
        .arg(dir.0.join("library.neoil"))
        .args(flags)
        .output()
        .unwrap()
}
#[test]
fn string_array_entry_emits_windows_abi_and_requires_gc() {
    let dir = Temp::new();
    let r = compile(&dir, SOURCE, &["--reference-arena"]);
    assert!(!r.status.success());
    assert!(
        String::from_utf8_lossy(&r.stderr).contains("String[] entry requires native GC"),
        "{:?}",
        r
    );
    let r = compile(
        &dir,
        SOURCE,
        &[
            "--reference-arena",
            "--native-gc",
            "--native-stack-budget",
            "--target",
            "x86_64-pc-windows-msvc",
            "--windows-console-experiment",
        ],
    );
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let bytes = fs::read(dir.0.join("app.o")).unwrap();
    let obj = object::File::parse(bytes.as_slice()).unwrap();
    assert!(
        obj.symbols()
            .any(|s| s.name().ok() == Some("neoclr_entry_args_v1") && s.is_definition())
    );
    assert!(
        obj.symbols()
            .any(|s| s.name().ok() == Some("neoclr_process_arguments_v1") && s.is_undefined())
    );
    assert!(
        !obj.symbols()
            .any(|s| s.name().ok() == Some("neoclr_entry_v4"))
    );
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn entry_copies_arguments_rejects_invalid_text_and_preserves_output_on_failure() {
    for (source, expected) in [
        (SOURCE, 3),
        (
            ".module EntryArgs\n.function Calculate(arrayref<String>) -> Void\nldvoid\nret\n.end\n",
            0,
        ),
        (
            ".module EntryArgs\n.function Calculate(arrayref<String>) -> noresult\nret\n.end\n",
            0,
        ),
    ] {
        let dir = Temp::new();
        let r = compile(&dir, source, &["--reference-arena", "--native-gc"]);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        fs::write(dir.0.join("host.c"), r#"
#include "entry-arguments.h"
#include <string.h>
int main(void) {
    uint64_t heap[4096] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    char *args[] = {"program", "", "two words", "R\xc3\xa4ven"};
    int32_t result = -99;
    if (neoclr_entry_args_v1(4, &result, &context, args) || result != 3) return 1;
    if (neoclr_root_probe_head_v1() || neoclr_gc_collect_v1(&context, NULL) || context.text.used) return 2;
    if (neoclr_entry_args_v1(1, &result, &context, args) || result) return 3;
    result = -99;
    char *invalid[] = {"program", "\xff"};
    if (neoclr_entry_args_v1(2, &result, &context, invalid) != 3 || result != -99 || context.fault.code != 3) return 4;
    if (neoclr_entry_args_v1(0, &result, &context, args) != 3 || result != -99) return 5;
    if (neoclr_entry_args_v1(1, &result, &context, NULL) != 3 || result != -99) return 6;
    if (neoclr_entry_args_v1(65538, &result, &context, args) != 7 || result != -99) return 7;
    context.text.capacity = 8;
    if (neoclr_entry_args_v1(1, &result, &context, args) != 5 || result != -99) return 8;
    context.text.capacity = sizeof(heap);
    if (neoclr_entry_args_v1(4, &result, &context, args) || result != 3 || context.fault.code) return 9;
    return neoclr_gc_collect_v1(&context, NULL) || context.text.used != 0;
}
"#.replace("result != 3", &format!("result != {expected}"))).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let base = root.join("docs/experiments/aot-console");
        let r = Command::new("clang")
            .args([
                "-arch",
                "arm64",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-DNEOCLR_NATIVE_GC",
                "-fsanitize=undefined,bounds",
                "-I",
            ])
            .arg(&base)
            .arg(dir.0.join("host.c"))
            .arg(dir.0.join("app.o"))
            .arg(root.join("tools/native/entry-arguments.c"))
            .arg(base.join("native-gc.c"))
            .arg(base.join("root-probe.c"))
            .arg(base.join("text-arena.c"))
            .arg("-o")
            .arg(dir.0.join("host"))
            .output()
            .unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(dir.0.join("host")).output().unwrap();
        assert!(r.status.success(), "{:?}", r);
        assert!(r.stderr.is_empty());
    }
}
