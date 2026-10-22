use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "neoclr-aot-import-{}-{}",
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
fn source(library: &str, symbol: &str) -> String {
    format!(
        ".module Imports\n.function Imported(Int32 value) -> Int32\n.foreign {library:?} {symbol:?} cdecl\n.end\n.function Calculate(Int32 value) -> Int32\nldarg value\ncall Imported(Int32)\nret\n.end\n"
    )
}
fn compile(dir: &Temp, source: &str, flags: &[&str]) -> std::process::Output {
    let input = dir.0.join("app.neoil");
    fs::write(&input, source).unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
        .arg(&input)
        .arg("Calculate")
        .arg(dir.0.join("app.o"))
        .args(flags)
        .output()
        .unwrap()
}
#[test]
fn import_capability_and_abi_are_validated_before_output() {
    for (text, flags) in [
        (source("missing", "increment"), vec![]),
        (
            source("missing", "increment").replace("Int32", "Int64"),
            vec!["--native-imports"],
        ),
        (
            source("missing", "increment"),
            vec!["--native-imports", "--native-imports"],
        ),
        (
            source("missing", "increment"),
            vec!["--native-imports", "--target", "x86_64-pc-windows-msvc"],
        ),
        (
            source("missing", "increment"),
            vec!["--native-imports", "--jit-run"],
        ),
        (
            source("missing", "increment").replace("cdecl", "stdcall"),
            vec!["--native-imports"],
        ),
    ] {
        let dir = Temp::new();
        let result = compile(&dir, &text, &flags);
        assert!(!result.status.success(), "unexpected admission: {text}");
        assert!(!dir.0.join("app.o").exists());
    }
    // Emission does not load the library or resolve its symbol.
    let dir = Temp::new();
    let result = compile(
        &dir,
        &source("/missing/neoclr/library", "missing"),
        &["--native-imports"],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(dir.0.join("app.o").is_file());
}
fn cc(args: &[&Path], extra: &[&str], output: &Path) {
    let result = Command::new(std::env::var("CC").unwrap_or_else(|_| "clang".into()))
        .args(extra)
        .args(args)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn imported_calls_resolve_lazily_cache_and_preserve_fault_outputs() {
    let dir = Temp::new();
    let fixture = dir.0.join("fixture.c");
    let library = dir.0.join("fixture.dylib");
    fs::write(
        &fixture,
        "#include <stdint.h>\nint32_t increment(int32_t value) { return value + 1; }\n",
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    let host = dir.0.join("host.c");
    fs::write(
        &host,
        r#"
#include <stdint.h>
#include <stdio.h>
#include <string.h>
extern int32_t neoclr_entry_v2(int32_t, int32_t*);
int main(int argc, char **argv) {
    int32_t result = 12345;
    int32_t status = neoclr_entry_v2(41, &result);
    if (argc == 2) return status == 3 && result == 12345 ? 0 : 1;
    if (status || result != 42) return 2;
    /* Cached owner/address must survive removal of the original library path. */
    if (rename(argv[1], argv[2])) return 3;
    for (int i = 0; i < 32; ++i) {
        if (neoclr_entry_v2(i, &result) || result != i + 1) return 4;
    }
    return 0;
}
"#,
    )
    .unwrap();
    let resolver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../native/interop/imports.c");
    for (symbol, missing_library) in [
        ("increment", false),
        ("missing_symbol", false),
        ("increment", true),
    ] {
        let output = dir.0.join("app.o");
        let _ = fs::remove_file(&output);
        let path = if missing_library {
            dir.0.join("missing.dylib")
        } else {
            library.clone()
        };
        let result = compile(
            &dir,
            &source(path.to_str().unwrap(), symbol),
            &["--native-imports"],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let executable = dir.0.join("app");
        cc(
            &[&host, &resolver, &output],
            &["-pthread", "-Wall", "-Wextra", "-Werror"],
            &executable,
        );
        let mut run = Command::new(&executable);
        if symbol == "increment" && !missing_library {
            let moved = dir.0.join("moved.dylib");
            let result = run.arg(&library).arg(&moved).output().unwrap();
            assert!(result.status.success(), "{result:?}");
            fs::rename(moved, &library).unwrap();
        } else {
            let result = run.arg("failure").output().unwrap();
            assert!(result.status.success(), "{result:?}");
        }
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_int32_register_and_stack_arguments_keep_order_and_signedness() {
    let dir = Temp::new();
    let fixture = dir.0.join("wide.c");
    let library = dir.0.join("wide.dylib");
    let parameters = (0..16)
        .map(|i| format!("int32_t p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let sum = (0..16)
        .map(|i| format!("p{i} * {}", i + 1))
        .collect::<Vec<_>>()
        .join("+");
    fs::write(
        &fixture,
        format!("#include <stdint.h>\nint32_t sum16({parameters}) {{ return {sum}; }}"),
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    let signature = (0..16)
        .map(|i| format!("Int32 p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let types = vec!["Int32"; 16].join(",");
    let arguments = (1..16).map(|i| format!("ldc.i4 {i}\n")).collect::<String>();
    let text = format!(
        ".module Wide\n.function Imported({signature}) -> Int32\n.foreign {:?} \"sum16\" cdecl\n.end\n.function Calculate(Int32 value) -> Int32\nldarg value\n{arguments}call Imported({types})\nret\n.end\n",
        library.to_str().unwrap()
    );
    let result = compile(&dir, &text, &["--native-imports"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let host = dir.0.join("host.c");
    fs::write(&host, "#include <stdint.h>\nextern int32_t neoclr_entry_v2(int32_t,int32_t*);\nint main(void) { int32_t out=0; return neoclr_entry_v2(-100,&out) || out!=1260; }\n").unwrap();
    let resolver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../native/interop/imports.c");
    let executable = dir.0.join("app");
    cc(
        &[&host, &resolver, &dir.0.join("app.o")],
        &["-pthread", "-Wall", "-Wextra", "-Werror"],
        &executable,
    );
    let result = Command::new(executable).output().unwrap();
    assert!(result.status.success(), "{result:?}");
}

#[test]
fn pointer_imports_reject_unsupported_layouts_and_scalar_misuse() {
    for body in [
        ".function Imported() -> Ptr<String>\n.foreign \"missing\" \"get\" cdecl\n.end\n.function Calculate(Int32 value) -> Int32\nldarg value\nret\n.end",
        ".function Imported() -> Ptr<Int32>\n.foreign \"missing\" \"get\" cdecl\n.end\n.function Calculate(Int32 value) -> Int32\ncall Imported()\nldarg value\nadd\nret\n.end",
        ".function Imported() -> Ptr<Int32>\n.foreign \"missing\" \"get\" cdecl\n.end\n.function Calculate(Int32 value) -> Ptr<Int32>\ncall Imported()\nret\n.end",
        ".function Imported(Ptr<Byte> p) -> Int32\n.foreign \"missing\" \"use\" cdecl\n.end\n.function Get() -> Ptr<Int32>\n.foreign \"missing\" \"get\" cdecl\n.end\n.function Calculate(Int32 value) -> Int32\ncall Get()\ncall Imported(Ptr<Byte>)\nret\n.end",
    ] {
        let dir = Temp::new();
        let result = compile(
            &dir,
            &format!(".module Rejected\n{body}\n"),
            &["--native-imports"],
        );
        assert!(!result.status.success(), "unexpected admission: {body}");
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_owned_pointers_survive_locals_helpers_and_null_roundtrips() {
    let dir = Temp::new();
    let fixture = dir.0.join("pointer.c");
    let library = dir.0.join("pointer.dylib");
    fs::write(
        &fixture,
        r#"
#include <stdint.h>
#include <stdlib.h>
void *create(int32_t value) {
    if (value == 0) return NULL;
    int32_t *p = malloc(sizeof(*p));
    if (p) *p = value;
    return p;
}
int32_t consume(void *address) {
    if (!address) return 0;
    int32_t *p = address;
    int32_t value = *p;
    free(p);
    return value;
}
"#,
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    for pointee in ["Void", "Byte", "Int32"] {
        let _ = fs::remove_file(dir.0.join("app.o"));
        let pointer = format!("Ptr<{pointee}>");
        let source = format!(
            r#".module Pointers
.function Create(Int32 value) -> {pointer}
.foreign {library:?} "create" cdecl
.end
.function Consume({pointer} address) -> Int32
.foreign {library:?} "consume" cdecl
.end
.function Forward({pointer} address) -> {pointer}
ldarg address
ret
.end
.function Calculate(Int32 value) -> Int32
.local {pointer} address
ldarg value
call Create(Int32)
stloc address
ldloc address
call Forward({pointer})
call Consume({pointer})
ret
.end
"#,
            library = library.to_str().unwrap()
        );
        let result = compile(&dir, &source, &["--native-imports"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let host = dir.0.join("pointer-host.c");
        fs::write(&host, "#include <stdint.h>\nextern int32_t neoclr_entry_v2(int32_t,int32_t*);\nint main(void) { int32_t result=123; for(int32_t i=-4;i<5;++i) { if(neoclr_entry_v2(i,&result)||result!=i) return 1; } return 0; }\n").unwrap();
        let resolver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../native/interop/imports.c");
        let executable = dir.0.join("pointer-app");
        cc(
            &[&host, &resolver, &dir.0.join("app.o")],
            &["-pthread", "-Wall", "-Wextra", "-Werror"],
            &executable,
        );
        assert!(Command::new(&executable).status().unwrap().success());
    }
}

#[test]
fn no_result_calls_do_not_supply_a_stack_value() {
    for body in ["call Release()\npop\nldc.i4 1\nret", "call Release()\nret"] {
        let dir = Temp::new();
        let source = format!(
            ".module InvalidVoid\n.function Release() -> void\n.foreign \"missing\" \"release\" cdecl\n.end\n.function Calculate(Int32 value) -> Int32\n{body}\n.end\n"
        );
        let result = compile(&dir, &source, &["--native-imports"]);
        assert!(!result.status.success());
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_void_release_preserves_caller_stack_and_fault_output() {
    let dir = Temp::new();
    let fixture = dir.0.join("release.c");
    let library = dir.0.join("release.dylib");
    fs::write(
        &fixture,
        r#"
#include <stdint.h>
#include <stdlib.h>
static int32_t observed;
void *create(int32_t value) { int32_t *p = malloc(sizeof(*p)); if (p) *p = value; return p; }
void release(void *address) { if (address) { observed = *(int32_t *)address; free(address); } }
int32_t read_value(void) { return observed; }
"#,
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    let source = format!(
        r#".module Release
.function Create(Int32 value) -> Ptr<Int32>
.foreign {library:?} "create" cdecl
.end
.function Release(Ptr<Int32> address) -> void
.foreign {library:?} "release" cdecl
.end
.function Read() -> Int32
.foreign {library:?} "read_value" cdecl
.end
.function ReleaseHelper(Ptr<Int32> address) -> void
ldarg address
call Release(Ptr<Int32>)
ret
.end
.function Calculate(Int32 value) -> Int32
ldc.i4 99
ldarg value
call Create(Int32)
call ReleaseHelper(Ptr<Int32>)
call Read()
add
ret
.end
"#,
        library = library.to_str().unwrap()
    );
    let resolver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../native/interop/imports.c");
    let host = dir.0.join("release-host.c");
    for missing in [false, true] {
        let _ = fs::remove_file(dir.0.join("app.o"));
        let source = if missing {
            format!(
                ".module Missing\n.function Release() -> void\n.foreign {:?} \"missing_release\" cdecl\n.end\n.function Calculate(Int32 value) -> Int32\nldarg value\ncall Release()\nret\n.end\n",
                library.to_str().unwrap()
            )
        } else {
            source.clone()
        };
        let result = compile(&dir, &source, &["--native-imports"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let check = if missing {
            "status == 3 && value == 12345"
        } else {
            "status == 0 && value == 140"
        };
        fs::write(&host, format!("#include <stdint.h>\nextern int32_t neoclr_entry_v2(int32_t,int32_t*);\nint main(void) {{ int32_t value=12345; int32_t status=neoclr_entry_v2(41,&value); return ({check}) ? 0 : 1; }}\n")).unwrap();
        let executable = dir.0.join("release-app");
        cc(
            &[&host, &resolver, &dir.0.join("app.o")],
            &["-pthread", "-Wall", "-Wextra", "-Werror"],
            &executable,
        );
        assert!(Command::new(&executable).status().unwrap().success());
    }
}
