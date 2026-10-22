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
.function Release(Ptr<Int32> address) -> System.Void
.foreign {library:?} "release" cdecl
.end
.function Read() -> Int32
.foreign {library:?} "read_value" cdecl
.end
.function ReleaseHelper(Ptr<Int32> address) -> System.Void
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

// Raven is an external toolchain, so this integration is explicitly selected with
// a matching bundle. The Rust framework owns assertions and native fixture cleanup;
// the scalar profile cannot execute the managed Raven test runner yet.
#[test]
#[ignore = "requires NEOCLR_RAVEN_BUNDLE with the matching native compiler and libraries"]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_native_metadata_imports_execute_aot_with_explicit_load_context() {
    let bundle =
        PathBuf::from(std::env::var_os("NEOCLR_RAVEN_BUNDLE").expect("set NEOCLR_RAVEN_BUNDLE"))
            .canonicalize()
            .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let sample = root.join("docs/experiments/aot-native-imports/raven");
    let dir = Temp::new();
    let project = dir.0.join("raven");
    fs::create_dir(&project).unwrap();
    for name in ["App.rvnproj", "Main.rvn"] {
        fs::copy(sample.join(name), project.join(name)).unwrap();
    }
    let fixture = dir.0.join("fixture.c");
    let library = dir.0.join("fixture.dylib");
    fs::write(
        &fixture,
        r#"
#include <stdlib.h>
#include <stdint.h>
static int32_t live;
void *allocate_value(int32_t value, size_t size) {
    if (size != sizeof(int32_t)) return NULL;
    int32_t *p = malloc(sizeof(*p));
    if (p) { *p = value; ++live; }
    return p;
}
int32_t read_value(int32_t *p) { return p ? *p : -100; }
void release_value(void *p) { if (p) { free(p); --live; } }
int32_t live_count(void) { return live; }
size_t large_size(void) { return (size_t)UINT64_C(0xfedcba9876543210); }
int32_t check_size(size_t n) { return n == large_size() ? 0 : 100; }
intptr_t signed_offset(void) { return -INT64_C(0x123456789); }
int32_t check_offset(intptr_t n) { return n == signed_offset() ? 0 : 100; }
"#,
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    let imports = dir.0.join("Imports.rvn");
    fs::write(
        &imports,
        fs::read_to_string(sample.join("Imports.rvn"))
            .unwrap()
            .replace(
                "\"aot-raven-fixture\"",
                &serde_json::to_string(&library.to_str().unwrap()).unwrap(),
            ),
    )
    .unwrap();
    let result = Command::new("dotnet")
        .arg(bundle.join("sdk/tools/rvnc/rvnc.dll"))
        .args(["neoclr", "--project"])
        .arg(project.join("App.rvnproj"))
        .env("NeoClrBundleRoot", &bundle)
        .env("NeoClrAotImports", &imports)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let artifact = project.join("bin/neoclr/RavenAotImports.dll");
    let object = dir.0.join("app.o");
    let backend = |output: &Path, context: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        cmd.arg(&artifact)
            .arg("@entry")
            .arg(output)
            .arg("--native-imports");
        if context {
            cmd.arg("--system")
                .arg(bundle.join("lib/System.runtime.neox"))
                .arg("--module")
                .arg(bundle.join("lib/System.Runtime.dll"))
                .arg("--object-root")
                .arg(bundle.join("lib/System.Runtime.dll"));
        }
        cmd.output().unwrap()
    };
    let missing = dir.0.join("missing-context.o");
    assert!(!backend(&missing, false).status.success());
    assert!(
        !missing.exists(),
        "unresolved source references must not produce an object"
    );
    let emitted = backend(&object, true);
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let host = dir.0.join("host.c");
    fs::write(
        &host,
        r#"
#include <stdint.h>
extern int32_t neoclr_entry_v2(int32_t, int32_t *);
int main(void) {
    for (int i = 0; i < 32; ++i) {
        int32_t result = -1;
        if (neoclr_entry_v2(0, &result) || result != 42) return 1;
    }
    return 0;
}
"#,
    )
    .unwrap();
    let app = dir.0.join("app");
    cc(
        &[
            &host,
            &object,
            &root.join("runtime/native/interop/imports.c"),
        ],
        &["-pthread"],
        &app,
    );
    assert!(Command::new(&app).status().unwrap().success());
}

#[test]
fn pointer_casts_and_null_require_opt_in_and_preserve_type_validation() {
    let valid = ".module PointerCasts\n.function Calculate(Int32 value) -> Int32\nptr.null Byte\nptr.cast void\nptr.cast Int32\npop\nldarg value\nret\n.end\n";
    let dir = Temp::new();
    let admitted = compile(&dir, valid, &["--native-imports"]);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );
    for (text, flags) in [
        (valid.to_owned(), vec![]),
        (
            valid.replace("ptr.null Byte", "ldc.i4 0"),
            vec!["--native-imports"],
        ),
        (
            valid.replace("ptr.null Byte", "ptr.null Double"),
            vec!["--native-imports"],
        ),
        (
            valid.replace("ptr.cast void", "ptr.cast Double"),
            vec!["--native-imports"],
        ),
        (
            valid.replace(
                "ptr.null Byte\nptr.cast void\nptr.cast Int32\npop",
                "ptr.null Byte\ncall Consume(Ptr<Int32>)",
            ) + ".function Consume(Ptr<Int32> value) -> void\nret\n.end\n",
            vec!["--native-imports"],
        ),
    ] {
        let dir = Temp::new();
        let result = compile(&dir, &text, &flags);
        assert!(!result.status.success(), "unexpected admission: {text}");
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
fn scalar_local_defaults_do_not_admit_escaping_or_mismatched_addresses() {
    let valid = ".module PointerDefaults\n.function Calculate(Int32 value) -> Int32\n.local Ptr<Int32> address\nldloca address\ninitobj Ptr<Int32>\nldloc address\npop\nldarg value\nret\n.end\n";
    let dir = Temp::new();
    let admitted = compile(&dir, valid, &["--native-imports"]);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );
    for text in [
        valid.replace("initobj Ptr<Int32>", "initobj Ptr<Byte>"),
        valid.replace("ldloca address", "ptr.null Int32"),
        valid.replace("ldloca address\ninitobj Ptr<Int32>\n", ""),
        valid.replace("initobj Ptr<Int32>", "stloc address"),
        valid.replace("ldloc address\npop", "ldloca address\nptr.cast void\npop"),
    ] {
        let dir = Temp::new();
        let result = compile(&dir, &text, &["--native-imports"]);
        assert!(!result.status.success(), "unexpected admission: {text}");
        assert!(!dir.0.join("app.o").exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_sized_integer_conversions_and_stack_arguments_preserve_bits() {
    let dir = Temp::new();
    let fixture = dir.0.join("widths.c");
    let library = dir.0.join("widths.dylib");
    fs::write(
        &fixture,
        r#"
#include <stdint.h>
uintptr_t widths(intptr_t a, uintptr_t b, uintptr_t c, uintptr_t d,
                 uintptr_t e, uintptr_t f, uintptr_t g, uintptr_t h, uintptr_t i) {
    if (a != -1 || b != UINT32_MAX || c != UINTPTR_MAX || d != UINT64_C(0x123456789) ||
        e != 4 || f != 5 || g != 6 || h != 7 || i != UINT64_C(0xfedcba9876543210)) return 0;
    return UINT64_C(0xfedcba9876543210);
}
"#,
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    let text = format!(
        r#".module NativeWidths
.function Widths(IntPtr a, UIntPtr b, UIntPtr c, UIntPtr d, UIntPtr e, UIntPtr f, UIntPtr g, UIntPtr h, UIntPtr i) -> UIntPtr
.foreign {library:?} "widths" cdecl
.end
.function Forward(UIntPtr value) -> UIntPtr
.local UIntPtr copy
ldarg value
stloc copy
ldloc copy
ret
.end
.function Calculate(Int32 unused) -> Int32
ldc.i4 -1
conv.i
ldc.i4 -1
conv.u
ldc.i4 -1
conv.i8
conv.u
ldc.i8 4886718345
conv.u
ldc.i4 4
conv.u
ldc.i4 5
conv.u
ldc.i4 6
conv.u
ldc.i4 7
conv.u
ldc.i8 -81985529216486896
conv.u
call Widths(IntPtr, UIntPtr, UIntPtr, UIntPtr, UIntPtr, UIntPtr, UIntPtr, UIntPtr, UIntPtr)
call Forward(UIntPtr)
conv.i4
ret
.end
"#,
        library = library.to_str().unwrap()
    );
    let result = compile(&dir, &text, &["--native-imports"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let host = dir.0.join("host.c");
    fs::write(
        &host,
        r#"
#include <stdint.h>
extern int32_t neoclr_entry_v2(int32_t, int32_t *);
int main(void) {
    int32_t result = 0;
    return neoclr_entry_v2(0, &result) || result != INT32_C(0x76543210);
}
"#,
    )
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let app = dir.0.join("app");
    cc(
        &[
            &host,
            &dir.0.join("app.o"),
            &root.join("runtime/native/interop/imports.c"),
        ],
        &["-pthread"],
        &app,
    );
    assert!(Command::new(&app).status().unwrap().success());
}

#[test]
fn native_integer_transport_rejects_unimplemented_operations() {
    for body in [
        "ptr.null Int32\nconv.u\npop\nldc.i4 0",
        "ldc.i4 1\nconv.u\nldc.i4 2\nconv.u\nadd\nconv.i4",
        "ldc.i4 -1\nconv.ovf.u\nconv.i4",
        "ldc.i4 1\nconv.u\ncall Signed(IntPtr)",
    ] {
        let source = format!(
            ".module IntegerGaps\n.function Signed(IntPtr value) -> Int32\nldc.i4 0\nret\n.end\n.function Calculate(Int32 value) -> Int32\n{body}\nret\n.end\n"
        );
        let dir = Temp::new();
        assert!(
            !compile(&dir, &source, &["--native-imports"])
                .status
                .success(),
            "unexpected admission: {body}"
        );
        assert!(!dir.0.join("app.o").exists());
    }
}
