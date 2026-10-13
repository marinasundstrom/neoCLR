use std::{fs, path::PathBuf, process::Command};

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_static_context_roots_preserve_children_and_release_at_teardown() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = std::env::temp_dir().join(format!("neoclr-static-context-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for gc in [false, true] {
        let binary = dir.join(if gc { "gc" } else { "arena" });
        let mut command = Command::new("xcrun");
        command.args(["--sdk", "macosx", "clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
            .arg(root.join("runtime/native/memory/static-storage-test.c"))
            .arg(root.join("runtime/native/memory/text-arena.c"));
        if gc {
            command.arg("-DNEOCLR_NATIVE_GC")
                .arg(root.join("runtime/native/gc/native-gc.c"))
                .arg(root.join("runtime/native/gc/root-probe.c"));
        }
        let result = command.arg("-o").arg(&binary).output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let result = Command::new(binary).output().unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn closed_generic_static_fields_execute_and_reset_in_aot() {
    use neoclr::metadata::{Instruction as Op, StaticField, Type, Visibility};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = std::env::temp_dir().join(format!("neoclr-static-aot-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut app = neoclr::assemble(".module Statics\n.entry Main\n.type class Cache<T>\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end").unwrap();
    app.types[0].static_fields.push(StaticField { name: "Count".into(), ty: Type::Int32, visibility: Visibility::Public, readonly: false });
    let a = Type::Constructed { definition: "Cache".into(), arguments: vec![Type::Int32] };
    let b = Type::Constructed { definition: "Cache".into(), arguments: vec![Type::String] };
    app.functions[0].body = vec![
        Op::StaticField { owner: a.clone(), index: 0 }, Op::Int(40), Op::Add,
        Op::SetStaticField { owner: a.clone(), index: 0 },
        Op::StaticField { owner: b.clone(), index: 0 }, Op::Int(2), Op::Add,
        Op::SetStaticField { owner: b.clone(), index: 0 },
        Op::StaticField { owner: a, index: 0 }, Op::StaticField { owner: b, index: 0 }, Op::Add, Op::Return];
    fs::write(dir.join("app.neox"), neoclr::metadata_container::write_module(&app).unwrap()).unwrap();
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    fs::write(dir.join("seed.neox"), neoclr::metadata_container::write_module(&seed).unwrap()).unwrap();
    fs::write(dir.join("helpers.neoil"), ".module Helpers\n.references ()\n").unwrap();
    fs::write(dir.join("host.c"), r#"
#include "native-gc.h"
int main(void) {
    uint64_t heap[1024] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    for (int i = 0; i < 3; i++) {
        int32_t result = 0;
        if (neoclr_entry_v4(0, &result, &c) || result != 42 || c.static_storage) return 1;
#ifdef NEOCLR_NATIVE_GC
        if (neoclr_gc_collect_v1(&c, NULL) || c.text.used) return 2;
#endif
    }
    return 0;
}
"#).unwrap();
    for gc in [false, true] {
        let object = dir.join(if gc { "gc.o" } else { "arena.o" });
        let mut compiler = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"));
        compiler.arg("--closed-world").arg(dir.join("app.neox")).arg("Main").arg(&object)
            .arg("--system").arg(dir.join("seed.neox")).arg("--module").arg(dir.join("helpers.neoil")).args(["--compile-system", "--reference-arena"]);
        if gc { compiler.arg("--native-gc"); }
        let result = compiler.output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let base = root.join("docs/experiments/aot-console");
        let mut link = Command::new("xcrun");
        link.args(["--sdk", "macosx", "clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds", "-I"])
            .arg(&base).arg(dir.join("host.c")).arg(&object).arg(base.join("text-arena.c"))
            .arg(root.join("docs/experiments/aot-fault-details/render.c"));
        if gc { link.arg("-DNEOCLR_NATIVE_GC").arg(base.join("native-gc.c")).arg(base.join("root-probe.c")); }
        let result = link.arg("-o").arg(dir.join("app")).output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let result = Command::new(dir.join("app")).output().unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    // RuntimeTypeHandle has a checked-uninitialized managed default. Native
    // storage must reject it rather than exposing zero as a valid type token.
    app.types[0].static_fields[0].ty = Type::RuntimeTypeHandle;
    app.functions[0].body = vec![
        Op::StaticField { owner: Type::Constructed { definition: "Cache".into(), arguments: vec![Type::Int32] }, index: 0 },
        Op::Pop, Op::Int(0), Op::Return];
    fs::write(dir.join("app.neox"), neoclr::metadata_container::write_module(&app).unwrap()).unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_neoclr-codegen"))
        .arg("--closed-world").arg(dir.join("app.neox")).arg("Main").arg(dir.join("invalid.o"))
        .arg("--system").arg(dir.join("seed.neox")).arg("--module").arg(dir.join("helpers.neoil"))
        .args(["--compile-system", "--reference-arena"]).output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("zero-initializable"), "{rejected:?}");
    fs::remove_dir_all(dir).unwrap();
}
