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
fn raven_native_metadata_imports_execute_aot_and_jit_with_explicit_load_context() {
    qualify_raven_imports("Main.rvn", false, false);
}

#[test]
#[ignore = "requires NEOCLR_RAVEN_BUNDLE with the matching native compiler and libraries"]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_native_pointer_transport_executes_in_all_three_engines() {
    qualify_raven_imports("Transport.rvn", true, false);
}

#[test]
#[ignore = "requires matching NEOCLR_RAVEN_BUNDLE"]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_aot_library_exports_are_callable_from_c_and_all_three_engines() {
    qualify_raven_imports("Transport.rvn", true, true);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn qualify_raven_imports(source: &str, interpreter: bool, aot_provider: bool) {
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
    fs::copy(sample.join("App.rvnproj"), project.join("App.rvnproj")).unwrap();
    fs::copy(sample.join(source), project.join("Main.rvn")).unwrap();
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
int64_t signed_wide(void) { return INT64_MIN + INT64_C(123); }
uint64_t unsigned_wide(void) { return UINT64_MAX - UINT64_C(456); }
int32_t check_wide(int64_t s, uint64_t u) {
    return s == signed_wide() && u == unsigned_wide() ? 0 : 100;
}
uint32_t unsigned_id(void) { return UINT32_C(0xf1234567); }
int32_t check_id(uint32_t id, uint64_t wide, uintptr_t native, uint32_t narrowed) {
    return id == unsigned_id() && wide == unsigned_id() && native == unsigned_id() &&
        narrowed == (uint32_t)unsigned_wide() ? 0 : 100;
}
#include <string.h>
static const uint32_t single_bits[] = {0x80000000, 0x7f800000, 0x7fc12345, 0xc1540000};
static const uint64_t double_bits[] = {
    UINT64_C(0x8000000000000000), UINT64_C(0x7ff0000000000000),
    UINT64_C(0x7ff8123456789abc), UINT64_C(0xc02a800000000000)
};
float single_value(int32_t index) {
    float value; memcpy(&value, &single_bits[index], sizeof(value)); return value;
}
double double_value(int32_t index) {
    double value; memcpy(&value, &double_bits[index], sizeof(value)); return value;
}
int32_t check_floating(float a, double b, int32_t marker, float c, double d,
                      float e, double f, float g, double h, float i, double j,
                      int64_t end) {
    float singles[] = {a, c, e, g};
    double doubles[] = {b, d, f, h};
    uint32_t iz; uint64_t jz;
    memcpy(&iz, &i, sizeof(iz)); memcpy(&jz, &j, sizeof(jz));
    return marker == 123 && end == signed_wide() && iz == 0 && jz == 0 &&
        memcmp(singles, single_bits, sizeof(single_bits)) == 0 &&
        memcmp(doubles, double_bits, sizeof(double_bits)) == 0 ? 0 : 100;
}
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
    if aot_provider {
        qualify_aot_provider(
            &project, &sample, &bundle, &imports, &fixture, &library, &root, &dir,
        );
        fs::copy(sample.join(source), project.join("Main.rvn")).unwrap();
    }
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
    if interpreter {
        let decode = |path: &Path| {
            let bytes = fs::read(path).unwrap();
            if bytes.starts_with(b"NEOX") {
                neoclr::metadata_container::decode_envelope(&bytes).unwrap()
            } else {
                neoclr::metadata_container::decode(&bytes).unwrap()
            }
        };
        let app = decode(&artifact);
        let system = decode(&bundle.join("lib/System.runtime.neox"));
        let core = decode(&bundle.join("lib/System.Runtime.dll"));
        let index = core
            .types
            .iter()
            .position(|t| t.name == "System.Object")
            .unwrap();
        let object_root = neoclr::metadata::TypeDefId {
            module: core.name.clone(),
            revision: core.revision.clone(),
            index: index as u32,
        };
        let program = neoclr::LoadedProgram::with_modules_and_object_root(
            &app,
            &system,
            &[core],
            &object_root,
        )
        .unwrap();
        program.verify().unwrap();
        assert!(
            program
                .run(neoclr::Limits::default())
                .unwrap_err()
                .message
                .contains("run_with_native"),
            "native execution must require explicit trust"
        );
        for _ in 0..32 {
            // SAFETY: the test owns this C fixture and its exact scalar signatures;
            // the guest frees its allocation and C never retains a managed pointer.
            let result = unsafe { program.run_with_native(neoclr::Limits::default()) }.unwrap();
            assert_eq!(result.value, neoclr::Value::Int32(42));
        }
    }
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
    let jit = |context: bool, extra: &[&str]| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        cmd.arg("--jit-scalar").arg(&artifact).args([
            "@entry",
            "0",
            "--repetitions",
            "32",
            "--native-imports",
        ]);
        if context {
            cmd.arg("--system")
                .arg(bundle.join("lib/System.runtime.neox"))
                .arg("--module")
                .arg(bundle.join("lib/System.Runtime.dll"))
                .arg("--object-root")
                .arg(bundle.join("lib/System.Runtime.dll"));
        }
        cmd.args(extra).output().unwrap()
    };
    assert!(
        !jit(false, &[]).status.success(),
        "unresolved Raven dependencies must reject before JIT execution"
    );
    let executed = jit(true, &[]);
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&executed.stdout).unwrap();
    let invocations = result["invocations"].as_array().unwrap();
    assert_eq!(invocations.len(), 32);
    assert!(
        invocations
            .iter()
            .all(|i| i["status"] == 0 && i["value"] == 42)
    );
    for extra in [
        vec!["--system", "duplicate"],
        vec!["--object-root", "duplicate"],
        vec!["--module"],
    ] {
        let rejected = jit(true, &extra);
        assert!(!rejected.status.success());
        assert!(
            rejected.stdout.is_empty(),
            "invalid contexts must not execute"
        );
    }
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
        "ptr.null Int32\nldobj Byte",
        "ptr.null Byte\nldind.i4",
        "ldc.i4 0\nstloc scratch\nldloca scratch\nldobj Int32",
        "ptr.null Int32\nunaligned. 1\nldind.i4",
        "ptr.null Int32\nptr.null Int32\nclt\npop\nldc.i4 0",
        "ptr.null Int32\nldc.i4 0\nceq\npop\nldc.i4 0",
        "ptr.null Int32\nptr.null Byte\nceq\npop\nldc.i4 0",
        "ldc.i4 1\nconv.u\nldc.i4 2\nconv.u\nadd\nconv.i4",
        "ldc.i4 -1\nconv.ovf.u\nconv.i4",
        "ldc.i4 1\nconv.u\ncall Signed(IntPtr)",
    ] {
        let source = format!(
            ".module IntegerGaps\n.function Signed(IntPtr value) -> Int32\nldc.i4 0\nret\n.end\n.function Calculate(Int32 value) -> Int32\n.local Int32 scratch\n{body}\nret\n.end\n"
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

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_pointer_and_integer_conditions_use_full_address_width() {
    let dir = Temp::new();
    let fixture = dir.0.join("addresses.c");
    let library = dir.0.join("addresses.dylib");
    fs::write(
        &fixture,
        r#"
#include <stdint.h>
/* Opaque sentinels are compared only, never dereferenced. */
void *high(void) { return (void *)(uintptr_t)UINT64_C(0x100000000); }
void *low(void) { return (void *)(uintptr_t)UINT64_C(0x200000000); }
"#,
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    let text = format!(
        r#".module Conditions
.function High() -> Ptr<Int32>
.foreign {library:?} "high" cdecl
.end
.function Low() -> Ptr<Int32>
.foreign {library:?} "low" cdecl
.end
.function Calculate(Int32 unused) -> Int32
call High()
brfalse failure
ptr.null Int32
brtrue failure
call High()
call Low()
ceq
brtrue failure
call High()
call Low()
beq failure
call High()
call High()
bne.un failure
ldc.i8 4294967296
conv.u
brfalse failure
ldc.i8 4294967296
conv.u
ldc.i8 8589934592
conv.u
ceq
brtrue failure
ldc.i8 -4294967296
conv.i
brfalse failure
ldc.i4 0
conv.i
brtrue failure
ldc.i4 42
ret
failure:
ldc.i4 -1
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
    return neoclr_entry_v2(0, &result) || result != 42;
}
"#,
    )
    .unwrap();
    let resolver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../native/interop/imports.c");
    let app = dir.0.join("app");
    cc(
        &[&host, &dir.0.join("app.o"), &resolver],
        &["-pthread"],
        &app,
    );
    assert!(Command::new(&app).status().unwrap().success());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_memory_access_preserves_width_and_fault_outputs() {
    let dir = Temp::new();
    let fixture = dir.0.join("storage.c");
    let library = dir.0.join("storage.dylib");
    fs::write(
        &fixture,
        r#"
#include <stdint.h>
static int32_t number;
static uint8_t octet;
void *int_cell(int32_t mode) {
    number = 0; octet = 0;
    if (mode == 1 || mode == 3) return 0;
    if (mode == 2 || mode == 4) return (void *)((uintptr_t)&number + 1);
    return &number;
}
void *byte_cell(void) { return &octet; }
int32_t check(void) { return number == -42 && octet == 255 ? 0 : 1000; }
"#,
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    for (int_load, int_store, byte_load, byte_store) in [
        ("ldobj Int32", "stobj Int32", "ldobj Byte", "stobj Byte"),
        ("ldind.i4", "stind.i4", "ldind.u1", "stind.i1"),
    ] {
        let _ = fs::remove_file(dir.0.join("app.o"));
        let text = format!(
            r#".module NativeAccess
.function IntCell(Int32 mode) -> Ptr<Int32>
.foreign {library:?} "int_cell" cdecl
.end
.function ByteCell() -> Ptr<Byte>
.foreign {library:?} "byte_cell" cdecl
.end
.function Check() -> Int32
.foreign {library:?} "check" cdecl
.end
.function Calculate(Int32 mode) -> Int32
.local Ptr<Int32> cell
.local Ptr<Byte> octet
ldarg mode
call IntCell(Int32)
stloc cell
ldarg mode
ldc.i4 3
bge read_only
ldloc cell
ldc.i4 -42
{int_store}
call ByteCell()
stloc octet
ldloc octet
ldc.i4 255
{byte_store}
ldloc cell
{int_load}
ldloc octet
{byte_load}
add
ldc.i4 213
sub
call Check()
add
ret
read_only:
ldloc cell
{int_load}
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
    int32_t result = 12345;
    if (neoclr_entry_v2(0, &result) || result != 0) return 1;
    for (int mode = 1; mode <= 4; ++mode) {
        result = 12345;
        if (neoclr_entry_v2(mode, &result) != 3 || result != 12345) return 2;
    }
    return 0;
}
"#,
        )
        .unwrap();
        let resolver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../native/interop/imports.c");
        let app = dir.0.join("app");
        cc(
            &[&host, &dir.0.join("app.o"), &resolver],
            &["-pthread"],
            &app,
        );
        assert!(Command::new(&app).status().unwrap().success());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn scalar_jit_imports_share_pointer_access_and_loader_fault_contracts() {
    let dir = Temp::new();
    let fixture = dir.0.join("jit.c");
    let library = dir.0.join("jit.dylib");
    fs::write(
        &fixture,
        r#"
#include <stdint.h>
#include <stdlib.h>
static int live;
void *allocate(size_t size) {
    if (size != sizeof(int32_t) || live) return NULL;
    int32_t *p = malloc(size);
    if (p) { *p = 41; ++live; }
    return p;
}
void release(void *p) { if (p) { --live; free(p); } }
uintptr_t high(void) { return UINT64_C(0x123456780000002a); }
"#,
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib", "-fPIC"], &library);
    let source = format!(
        r#".module JitImports
.function Allocate(UIntPtr size) -> Ptr<Int32>
.foreign {library:?} "allocate" cdecl
.end
.function Release(Ptr<Void> pointer) -> void
.foreign {library:?} "release" cdecl
.end
.function High() -> UIntPtr
.foreign {library:?} "high" cdecl
.end
.function Calculate(Int32 size) -> Int32
.local Ptr<Int32> pointer
.local Int32 result
ldarg size
conv.u
call Allocate(UIntPtr)
stloc pointer
ldloc pointer
ldloc pointer
ldobj Int32
ldc.i4 1
add
stobj Int32
ldloc pointer
ldobj Int32
stloc result
ldloc pointer
ptr.cast Void
call Release(Ptr<Void>)
call High()
ldc.i8 1311768464867721258
conv.u
ceq
brfalse failure
ldloc result
ret
failure:
ldc.i4 -1
ret
.end
"#,
        library = library.to_str().unwrap()
    );
    let input = dir.0.join("jit.neoil");
    fs::write(&input, &source).unwrap();
    let run = |enabled: bool, argument: &str| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        command.arg("--jit-scalar").arg(&input).args([
            "Calculate",
            argument,
            "--repetitions",
            "32",
        ]);
        if enabled {
            command.arg("--native-imports");
        }
        command.output().unwrap()
    };
    assert!(
        !run(false, "4").status.success(),
        "native imports must remain opt-in"
    );
    for (argument, status, value) in [("4", 0, Some(42)), ("0", 3, None)] {
        let output = run(true, argument);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let invocations = result["invocations"].as_array().unwrap();
        assert_eq!(invocations.len(), 32);
        for invocation in invocations {
            assert_eq!(invocation["status"], status);
            assert_eq!(invocation["value"], serde_json::json!(value));
        }
    }
    fs::write(
        &input,
        source.replace("\"allocate\"", "\"missing_allocate\""),
    )
    .unwrap();
    let output = run(true, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        result["invocations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["status"] == 3 && i["value"].is_null())
    );
}

#[test]
fn floating_transport_requires_opt_in_and_rejects_unimplemented_operations() {
    for (ty, constant) in [("Single", "ldc.r4 -1.25"), ("Double", "ldc.r8 -1.25")] {
        let valid = format!(
            ".module FloatingTransport\n.function Unused({ty} value) -> {ty}\n.foreign \"unused\" \"unused\" cdecl\n.end\n.function Calculate(Int32 value) -> Int32\n.local {ty} number\nldloca number\ninitobj {ty}\nldloc number\npop\n{constant}\ncall Forward({ty})\npop\nldarg value\nret\n.end\n.function Forward({ty} value) -> {ty}\nldarg value\nret\n.end\n"
        );
        let dir = Temp::new();
        let result = compile(&dir, &valid, &["--native-imports"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        for (text, flags) in [
            (valid.clone(), vec![]),
            (
                valid.replace(
                    "ldarg value\nret\n.end\n",
                    "ldarg value\ndup\nadd\nret\n.end\n",
                ),
                vec!["--native-imports"],
            ),
            (
                valid.replace("ldloc number\npop", "ldloc number\nconv.i4\npop"),
                vec!["--native-imports"],
            ),
            (
                valid.replace("ldloc number\npop", "ldloc number\ndup\nceq\npop"),
                vec!["--native-imports"],
            ),
        ] {
            let dir = Temp::new();
            let result = compile(&dir, &text, &flags);
            assert!(!result.status.success(), "unexpected admission: {text}");
            assert!(!dir.0.join("app.o").exists());
            assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[allow(clippy::too_many_arguments)]
fn qualify_aot_provider(
    project: &Path,
    sample: &Path,
    bundle: &Path,
    imports: &Path,
    fixture: &Path,
    library: &Path,
    root: &Path,
    dir: &Temp,
) {
    fs::copy(sample.join("Exports.rvn"), project.join("Main.rvn")).unwrap();
    let compiled = Command::new("dotnet")
        .arg(bundle.join("sdk/tools/rvnc/rvnc.dll"))
        .args(["neoclr", "--project"])
        .arg(project.join("App.rvnproj"))
        .env("NeoClrBundleRoot", bundle)
        .env("NeoClrAotImports", imports)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    let object = dir.0.join("exports.o");
    let header = dir.0.join("exports.h");
    let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
    command
        .arg("--native-library")
        .arg(project.join("bin/neoclr/RavenAotImports.dll"))
        .arg(&object)
        .arg("--native-imports")
        .arg("--header")
        .arg(&header);
    command
        .arg("--system")
        .arg(bundle.join("lib/System.runtime.neox"))
        .arg("--module")
        .arg(bundle.join("lib/System.Runtime.dll"))
        .arg("--object-root")
        .arg(bundle.join("lib/System.Runtime.dll"));
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let symbol = |name: &str| {
        report["exports"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["function"] == name)
            .unwrap()["symbol"]
            .as_str()
            .unwrap()
    };
    use object::{Object, ObjectSymbol};
    let bytes = fs::read(&object).unwrap();
    let binary = object::File::parse(bytes.as_slice()).unwrap();
    let globals: Vec<_> = binary
        .symbols()
        .filter(|s| s.is_global() && s.is_definition())
        .map(|s| s.name().unwrap().trim_start_matches('_').to_owned())
        .collect();
    assert_eq!(globals.len(), 8);
    for entry in report["exports"].as_array().unwrap() {
        assert!(
            globals
                .iter()
                .any(|s| s == entry["symbol"].as_str().unwrap())
        );
    }
    cc(
        &[
            fixture,
            &object,
            &root.join("runtime/native/interop/imports.c"),
        ],
        &["-dynamiclib", "-fPIC", "-pthread"],
        library,
    );
    let host = dir.0.join("export-host.c");
    fs::write(
        &host,
        format!(
            r#"
#include "exports.h"
int main(int argc, char **argv) {{
    if (argc > 1) return {divide}(0);
    int32_t value = 10;
    {write}(&value, 42);
    return {read}(&value) != 42 || {divide}(2) != 21;
}}
"#,
            read = symbol("ReadExport"),
            write = symbol("WriteExport"),
            divide = symbol("DivideExport")
        ),
    )
    .unwrap();
    let host_app = dir.0.join("export-host");
    cc(&[&host, library], &[], &host_app);
    assert!(Command::new(&host_app).status().unwrap().success());
    let cpp = dir.0.join("export-host.cpp");
    fs::copy(&host, &cpp).unwrap();
    let cpp_app = dir.0.join("export-host-cpp");
    cc(&[&cpp, library], &["-std=c++17"], &cpp_app);
    assert!(Command::new(&cpp_app).status().unwrap().success());
    use std::os::unix::process::ExitStatusExt;
    let fault = Command::new(&host_app).arg("fault").output().unwrap();
    assert!(
        fault.status.signal().is_some(),
        "escaping guest fault must terminate at the C boundary"
    );
    let mut mapped = fs::read_to_string(imports).unwrap();
    for (native, exported) in [
        ("allocate_value", "AllocateExport"),
        ("read_value", "ReadExport"),
        ("release_value", "ReleaseExport"),
        ("single_value", "SingleExport"),
        ("double_value", "DoubleExport"),
        ("unsigned_wide", "WideExport"),
    ] {
        mapped = mapped.replace(
            &format!("EntryPoint: \"{native}\""),
            &format!("EntryPoint: \"{}\"", symbol(exported)),
        );
    }
    fs::write(imports, mapped).unwrap();
}

#[test]
fn native_library_export_selection_rejects_invalid_contracts_before_output() {
    let valid = ".module Library
.function Echo(Int32 value) -> Int32
ldarg value
ret
.end
";
    for (source, exports) in [
        (valid.to_owned(), vec![]),
        (valid.to_owned(), vec!["Missing"]),
        (valid.to_owned(), vec!["Echo", "Echo"]),
        (source("unused", "echo"), vec!["Imported"]),
        (valid.replace("Int32", "String"), vec!["Echo"]),
        (valid.replace("Int32", "Ptr<Int64>"), vec!["Echo"]),
    ] {
        let dir = Temp::new();
        let input = dir.0.join("library.neoil");
        let object = dir.0.join("library.o");
        fs::write(&input, source).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        command.arg("--native-library").arg(&input).arg(&object);
        for export in exports {
            command.args(["--export", export]);
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        assert!(!object.exists());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_struct_pointers_and_generated_layout_work_across_engines() {
    let dir = Temp::new();
    let fixture = dir.0.join("pair.c");
    let library = dir.0.join("pair.dylib");
    fs::write(&fixture, r#"
#include <stdint.h>
#include <stdlib.h>
typedef struct { int32_t X; double Y; } Pair;
Pair *create(int32_t value) { Pair *p = malloc(sizeof(*p)); if(p) { p->X=value; p->Y=2.0; } return p; }
int32_t probe(Pair *p) { return p ? p->X + (int32_t)p->Y : -1; }
void release(Pair *p) { free(p); }
"#).unwrap();
    cc(&[&fixture], &["-dynamiclib"], &library);
    let source = format!(
        r#".module Structs
.type Pair
.field X Int32
.field Y Double
.end
.function Create(Int32 value) -> Ptr<Pair>
.foreign {library:?} "create" cdecl
.end
.function Probe(Ptr<Pair> value) -> Int32
.foreign {library:?} "probe" cdecl
.end
.function Release(Ptr<Pair> value) -> void
.foreign {library:?} "release" cdecl
.end
.function Forward(Ptr<Pair> value) -> Int32
ldarg value
call Probe(Ptr<Pair>)
ret
.end
.function Calculate(Int32 value) -> Int32
.local Ptr<Pair> pointer
.local Int32 result
ldarg value
call Create(Int32)
stloc pointer
ldloc pointer
call Forward(Ptr<Pair>)
stloc result
ldloc pointer
call Release(Ptr<Pair>)
ldloc result
ret
.end
.function Main() -> Int32
ldc.i4 40
call Calculate(Int32)
ret
.end
.entry Main
"#,
        library = library.to_str().unwrap()
    );
    let module = neoclr::assemble(&source).unwrap();
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    // SAFETY: the fixture owns allocation, layout, reads and destruction.
    assert_eq!(
        unsafe { program.run_with_native(neoclr::Limits::default()) }
            .unwrap()
            .value,
        neoclr::Value::Int32(42)
    );
    let output = compile(&dir, &source, &["--native-imports"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let input = dir.0.join("app.neoil");
    let jit = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
        .args(["--jit-scalar"])
        .arg(&input)
        .args(["Calculate", "40", "--native-imports"])
        .output()
        .unwrap();
    assert!(
        jit.status.success(),
        "{}",
        String::from_utf8_lossy(&jit.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&jit.stdout).unwrap();
    assert_eq!(result["invocations"][0]["value"], 42);
    let resolver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../native/interop/imports.c");
    let host = dir.0.join("host.c");
    fs::write(&host, "#include <stdint.h>\nextern int32_t neoclr_entry_v2(int32_t,int32_t*);\nint main(void){int32_t r=0;return neoclr_entry_v2(40,&r) || r!=42;}\n").unwrap();
    let executable = dir.0.join("app");
    cc(&[&host, &dir.0.join("app.o"), &resolver], &[], &executable);
    assert!(Command::new(&executable).status().unwrap().success());
    let object = dir.0.join("exports.o");
    let header = dir.0.join("exports.h");
    let exported = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
        .arg("--native-library")
        .arg(&input)
        .arg(&object)
        .args(["--export", "Forward", "--native-imports", "--header"])
        .arg(&header)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&exported.stdout).unwrap();
    let symbol = report["exports"][0]["symbol"].as_str().unwrap();
    let text = fs::read_to_string(&header).unwrap();
    let name = text
        .lines()
        .find(|l| l.starts_with("typedef struct "))
        .unwrap()
        .split_whitespace()
        .nth(2)
        .unwrap();
    fs::write(&host, format!("#include \"exports.h\"\nint main(void) {{ {name} p; p.field_58=40; p.field_59=2.0; return {symbol}(&p)!=42; }}\n")).unwrap();
    cc(&[&host, &object, &resolver], &[], &executable);
    assert!(Command::new(&executable).status().unwrap().success());
    let cpp = dir.0.join("host.cpp");
    fs::copy(&host, &cpp).unwrap();
    let resolver_object = dir.0.join("resolver.o");
    cc(&[&resolver], &["-c"], &resolver_object);
    cc(
        &[&cpp, &object, &resolver_object],
        &["-std=c++17"],
        &executable,
    );
    assert!(Command::new(&executable).status().unwrap().success());
}

#[test]
#[ignore = "requires matching NEOCLR_RAVEN_BUNDLE with nominal pointer emission"]
fn raven_value_pointer_exports_compile_and_execute() {
    qualify_raven_layout(
        "public struct Point {\n    field X: int\n    field Y: int\n}",
        Some(8),
    );
}

#[test]
#[ignore = "requires matching NEOCLR_RAVEN_BUNDLE with layout attributes"]
fn raven_layout_attributes_control_native_headers_and_reject_invalid_contracts() {
    for (declaration, size) in [
        (
            "[StructLayout(LayoutKind.Sequential, Pack: 1)]\npublic struct Point {\n    [MarshalAs(NativeType.Bool8)]\n    field Flag: bool\n    field Value: int\n}",
            Some(5),
        ),
        (
            "[StructLayout(LayoutKind.Explicit, Pack: 1, Size: 12)]\npublic struct Point {\n    [FieldOffset(1), MarshalAs(NativeType.Bool32)]\n    field Flag: bool\n    [FieldOffset(1)]\n    field Value: int\n}",
            Some(12),
        ),
        (
            "[StructLayout(LayoutKind.Auto)]\npublic struct Point { field Value: int }",
            None,
        ),
        (
            "[StructLayout(LayoutKind.Sequential, Pack: 3)]\npublic struct Point { field Value: int }",
            None,
        ),
        (
            "[StructLayout(LayoutKind.Explicit)]\npublic struct Point { field Value: int }",
            None,
        ),
        (
            "public struct Point {\n    [MarshalAs(NativeType.Bool8)]\n    field Value: int\n}",
            None,
        ),
    ] {
        qualify_raven_layout(declaration, size);
    }
}

#[test]
#[ignore = "requires matching NEOCLR_RAVEN_BUNDLE with layout attributes"]
fn generated_raven_bindings_preserve_nested_and_recursive_pointer_layouts() {
    for (declaration, size) in [
        (
            "public struct Inner { field Value: long }\npublic struct Point {\n    field Tag: byte\n    field Data: Inner\n}",
            16,
        ),
        (
            "public struct Point {\n    field Value: int\n    field Next: *Point\n}",
            16,
        ),
        (
            "public struct Point { field Other: *Other }\npublic struct Other { field Back: *Point }",
            8,
        ),
    ] {
        qualify_raven_layout(declaration, Some(size));
    }
}

fn qualify_raven_layout(declaration: &str, expected_size: Option<usize>) {
    let bundle = PathBuf::from(std::env::var_os("NEOCLR_RAVEN_BUNDLE").unwrap())
        .canonicalize()
        .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = Temp::new();
    fs::copy(
        root.join("tools/native/samples/native-interop/App.rvnproj"),
        dir.0.join("App.rvnproj"),
    )
    .unwrap();
    fs::write(
        dir.0.join("Main.rvn"),
        r#"
import System.Runtime.InteropServices.*
DECLARATION
[NativeExport("point_identity")]
unsafe func Identity(pointer: *Point) -> *Point {
    return Echo(pointer)
}
unsafe func Echo(pointer: *Point) -> *Point {
    return pointer
}
func Main() -> int { return 0 }
"#
        .replace("DECLARATION", declaration),
    )
    .unwrap();
    let output = Command::new("dotnet")
        .arg(bundle.join("sdk/tools/rvnc/rvnc.dll"))
        .args(["neoclr", "--project"])
        .arg(dir.0.join("App.rvnproj"))
        .env("NeoClrBundleRoot", &bundle)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = fs::read(dir.0.join("bin/neoclr/NativeInteropSample.dll")).unwrap();
    let module = neoclr::metadata_container::decode_envelope(&bytes).unwrap();
    let point = module
        .types
        .iter()
        .find(|t| t.origin.as_ref().is_some_and(|o| o.name == "Point"))
        .unwrap();
    let identity = module
        .functions
        .iter()
        .find(|f| f.origin.as_ref().is_some_and(|o| o.name == "Identity"))
        .unwrap();
    let pointer =
        neoclr::metadata::Type::Ptr(Box::new(neoclr::metadata::Type::Named(point.name.clone())));
    assert_eq!(identity.parameters, vec![pointer.clone()]);
    assert_eq!(identity.returns, pointer);
    let object = dir.0.join("exports.o");
    let header = dir.0.join("exports.h");
    let raven = dir.0.join("Imports.rvn");
    let library = dir.0.join("exports.dylib");
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
        .arg("--native-library")
        .arg(dir.0.join("bin/neoclr/NativeInteropSample.dll"))
        .arg(&object)
        .arg("--header")
        .arg(&header)
        .arg("--raven-imports")
        .arg(&raven)
        .arg("--import-library")
        .arg(&library)
        .arg("--system")
        .arg(bundle.join("lib/System.runtime.neox"))
        .arg("--module")
        .arg(bundle.join("lib/System.Runtime.dll"))
        .arg("--object-root")
        .arg(bundle.join("lib/System.Runtime.dll"))
        .output()
        .unwrap();
    if expected_size.is_none() {
        assert!(
            !output.status.success(),
            "invalid native layout admitted: {declaration}"
        );
        assert!(!object.exists());
        assert!(!header.exists());
        return;
    }
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = fs::read_to_string(&header).unwrap();
    let name = text
        .lines()
        .find(|line| line.contains(" point_identity("))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    let host = dir.0.join("caller.c");
    fs::write(&host, format!("#include \"exports.h\"\nint main(void) {{ {name} p = {{0}}; return sizeof(p) != {} || point_identity(&p) != &p || point_identity(0) != 0; }}\n", expected_size.unwrap())).unwrap();
    let executable = dir.0.join("caller");
    cc(&[&host, &object], &["-std=c11"], &executable);
    assert!(Command::new(&executable).status().unwrap().success());
    let cpp = dir.0.join("caller.cpp");
    fs::copy(&host, &cpp).unwrap();
    cc(&[&cpp, &object], &["-std=c++17"], &executable);
    assert!(Command::new(&executable).status().unwrap().success());
    cc(&[&object], &["-dynamiclib"], &library);
    let consumer = dir.0.join("consumer");
    fs::create_dir(&consumer).unwrap();
    fs::copy(dir.0.join("App.rvnproj"), consumer.join("App.rvnproj")).unwrap();
    fs::copy(&raven, consumer.join("Imports.rvn")).unwrap();
    fs::write(
        consumer.join("Main.rvn"),
        format!(
            r#"
import System.Runtime.InteropServices.*

[NativeImport("/usr/lib/libSystem.B.dylib", EntryPoint: "malloc")]
unsafe extern func Allocate(size: nuint) -> *unit;

[NativeImport("/usr/lib/libSystem.B.dylib", EntryPoint: "free")]
unsafe extern func Release(pointer: *unit) -> unit;

unsafe func Main() -> int {{
    if NativeExport0(default(*{name})) != default(*{name}) {{
        return 1
    }}
    let storage = Allocate((nuint){size})
    if storage == default(*unit) {{
        return 2
    }}
    let pointer = (*{name})storage
    let same = NativeExport0(pointer) == pointer
    Release(storage)
    if same {{
        return 0
    }}
    return 3
}}
"#,
            size = expected_size.unwrap()
        ),
    )
    .unwrap();
    let result = Command::new("dotnet")
        .arg(bundle.join("sdk/tools/rvnc/rvnc.dll"))
        .args(["neoclr", "--project"])
        .arg(consumer.join("App.rvnproj"))
        .env("NeoClrBundleRoot", &bundle)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let decode = |path: PathBuf| {
        let bytes = fs::read(path).unwrap();
        if bytes.starts_with(b"NEOX") {
            neoclr::metadata_container::decode_envelope(&bytes).unwrap()
        } else {
            neoclr::metadata_container::decode(&bytes).unwrap()
        }
    };
    let app = decode(consumer.join("bin/neoclr/NativeInteropSample.dll"));
    let seed = decode(bundle.join("lib/System.runtime.neox"));
    let core = decode(bundle.join("lib/System.Runtime.dll"));
    let object_root = neoclr::metadata::TypeDefId {
        module: core.name.clone(),
        revision: core.revision.clone(),
        index: core
            .types
            .iter()
            .position(|t| t.name == "System.Object")
            .unwrap() as u32,
    };
    let program =
        neoclr::LoadedProgram::with_modules_and_object_root(&app, &seed, &[core], &object_root)
            .unwrap();
    program.verify().unwrap();
    // SAFETY: exact C ABI exports receive null or a live C allocation; the consumer frees it.
    let result = unsafe { program.run_with_native(neoclr::Limits::default()) }.unwrap();
    assert_eq!(result.value, neoclr::Value::Int32(0));
    let artifact = consumer.join("bin/neoclr/NativeInteropSample.dll");
    let context = |command: &mut Command| {
        command
            .arg("--native-imports")
            .arg("--system")
            .arg(bundle.join("lib/System.runtime.neox"))
            .arg("--module")
            .arg(bundle.join("lib/System.Runtime.dll"))
            .arg("--object-root")
            .arg(bundle.join("lib/System.Runtime.dll"));
    };
    let mut jit = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
    jit.arg("--jit-scalar").arg(&artifact).args(["@entry", "0"]);
    context(&mut jit);
    let executed = jit.output().unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&executed.stdout).unwrap();
    assert_eq!(result["invocations"][0]["status"], 0);
    assert_eq!(result["invocations"][0]["value"], 0);
    let object = consumer.join("app.o");
    let mut aot = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
    aot.arg(&artifact).arg("@entry").arg(&object);
    context(&mut aot);
    let emitted = aot.output().unwrap();
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let host = consumer.join("host.c");
    fs::write(&host, "#include <stdint.h>\nextern int32_t neoclr_entry_v2(int32_t, int32_t *);\nint main(void) { int32_t result = -1; return neoclr_entry_v2(0, &result) || result != 0; }\n").unwrap();
    let resolver = root.join("runtime/native/interop/imports.c");
    let executable = consumer.join("app");
    cc(&[&host, &object, &resolver], &[], &executable);
    assert!(Command::new(executable).status().unwrap().success());
}

#[test]
fn export_selection_preserves_source_verification_and_selected_admission() {
    let source = ".module Selection\n.function Export(Int32 value) -> Int32\nldarg value\nret\n.end\n.function Unused(String value) -> String\nldarg value\nret\n.end\n";
    for (source, export, succeeds) in [
        (source.to_owned(), "Export", true),
        (source.to_owned(), "Unused", false),
        (
            source.replace(
                ".function Unused(String value) -> String\nldarg value",
                ".function Unused(String value) -> String\nldc.i4 1",
            ),
            "Export",
            false,
        ),
    ] {
        let dir = Temp::new();
        let input = dir.0.join("app.neoil");
        let object = dir.0.join("app.o");
        fs::write(&input, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
            .arg("--native-library")
            .arg(&input)
            .arg(&object)
            .args(["--export", export])
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            succeeds,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(object.exists(), succeeds);
        let executable_object = dir.0.join("executable.o");
        for jit in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
            if jit {
                command.arg("--jit-scalar").arg(&input).args([export, "0"]);
            } else {
                command.arg(&input).arg(export).arg(&executable_object);
            }
            let output = command.arg("--native-imports").output().unwrap();
            assert_eq!(
                output.status.success(),
                succeeds,
                "jit={jit}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            if !jit {
                assert_eq!(executable_object.exists(), succeeds);
            }
        }
    }
}

#[test]
fn private_boolean_calls_and_defaults_do_not_admit_native_boolean_abi() {
    let dir = Temp::new();
    let source = ".module BooleanStorage\n.function Forward(Boolean value) -> Boolean\nldarg value\nret\n.end\n.function Calculate(Int32 value) -> Int32\n.local Boolean flag\nldloca flag\ninitobj Boolean\nldloc flag\nbrtrue fail\nldc.bool true\ncall Forward(Boolean)\nstloc flag\nldloc flag\nbrfalse fail\nldarg value\nret\nfail:\nldc.i4 -1\nret\n.end\n";
    let emitted = compile(&dir, source, &["--native-imports"]);
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
        .arg("--jit-scalar")
        .arg(dir.0.join("app.neoil"))
        .args(["Calculate", "7", "--native-imports"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["invocations"][0]["status"], 0);
    assert_eq!(result["invocations"][0]["value"], 7);
    let rejected = Temp::new();
    let imported = source.replace(
        "ldarg value\nret\n.end\n.function Calculate",
        ".foreign \"unused\" \"forward\" cdecl\n.end\n.function Calculate",
    );
    assert!(
        !compile(&rejected, &imported, &["--native-imports"])
            .status
            .success()
    );
    assert!(!rejected.0.join("app.o").exists());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn import_callsite_cache_retries_failure_and_publishes_concurrently() {
    let dir = Temp::new();
    let fixture = dir.0.join("fixture.c");
    let library = dir.0.join("fixture.dylib");
    fs::write(
        &fixture,
        "#include <stdint.h>\nint32_t increment(int32_t value) { return value + 1; }\n",
    )
    .unwrap();
    cc(&[&fixture], &["-dynamiclib"], &library);
    let resolver = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../native/interop/imports.c")
        .canonicalize()
        .unwrap();
    let host = dir.0.join("host.c");
    fs::write(&host, format!("#define neoclr_native_import_resolve_v1 actual_resolve\n#include {:?}\n#undef neoclr_native_import_resolve_v1\n", resolver.to_str().unwrap()) + r#"
#include <stdatomic.h>
#include <stdio.h>
#include <time.h>
#include <sched.h>
static atomic_int calls, start;
extern int32_t neoclr_entry_v2(int32_t, int32_t *);
int32_t neoclr_native_import_resolve_v1(const char *library, const char *symbol, void **output) {
    if (atomic_fetch_add(&calls, 1) == 0) return 3;
    return actual_resolve(library, symbol, output);
}
static void *worker(void *unused) {
    (void)unused;
    while (!atomic_load(&start)) sched_yield();
    for (int i = 0; i < 100; ++i) {
        int32_t result = -1;
        if (neoclr_entry_v2(i, &result) || result != i + 1) return (void *)1;
    }
    return NULL;
}
static double now(void) {
    struct timespec value;
    clock_gettime(CLOCK_MONOTONIC, &value);
    return value.tv_sec + value.tv_nsec / 1e9;
}
int main(int argc, char **argv) {
    (void)argv;
    int32_t value = 12345;
    if (neoclr_entry_v2(41, &value) != 3 || value != 12345) return 1;
    pthread_t threads[8];
    for (int i = 0; i < 8; ++i) if (pthread_create(&threads[i], NULL, worker, NULL)) return 2;
    atomic_store(&start, 1);
    for (int i = 0; i < 8; ++i) {
        void *result;
        if (pthread_join(threads[i], &result) || result) return 3;
    }
    int cold_calls = atomic_load(&calls);
    if (argc == 1 && (cold_calls < 2 || cold_calls > 9)) return 4;
    for (int round = 0; round < 5; ++round) {
        double begin = now();
        for (int i = 0; i < 1000000; ++i) {
            if (neoclr_entry_v2(i, &value) || value != i + 1) return 5;
        }
        printf("warm_million_seconds=%.9f\n", now() - begin);
    }
    printf("resolver_calls_cold=%d total=%d\n", cold_calls, atomic_load(&calls));
    if (argc == 1 && atomic_load(&calls) != cold_calls) return 6;
    return 0;
}
"#).unwrap();
    let source = source(library.to_str().unwrap(), "increment");
    assert!(
        compile(&dir, &source, &["--native-imports"])
            .status
            .success()
    );
    let executable = dir.0.join("app");
    cc(
        &[&host, &dir.0.join("app.o")],
        &["-O2", "-pthread"],
        &executable,
    );
    let result = Command::new(&executable).output().unwrap();
    assert!(result.status.success(), "{result:?}");
    println!("cached:\n{}", String::from_utf8_lossy(&result.stdout));
    if let Some(baseline) = std::env::var_os("NEOCLR_IMPORT_BASELINE_CODEGEN") {
        let object = dir.0.join("baseline.o");
        let result = Command::new(baseline)
            .arg(dir.0.join("app.neoil"))
            .arg("Calculate")
            .arg(&object)
            .arg("--native-imports")
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        cc(&[&host, &object], &["-O2", "-pthread"], &executable);
        let result = Command::new(&executable).arg("baseline").output().unwrap();
        assert!(result.status.success(), "{result:?}");
        println!("baseline:\n{}", String::from_utf8_lossy(&result.stdout));
    }
}
