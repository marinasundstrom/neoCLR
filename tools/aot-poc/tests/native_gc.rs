use std::{fs, path::PathBuf, process::Command};

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn bounded_nonmoving_heap_reclaims_cycles_and_preserves_interior_and_fault_roots() {
    let dir = std::env::temp_dir().join(format!("neoclr-native-gc-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let binary = dir.join("heap");
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
        .arg(base.join("native-gc-test.c"))
        .arg(base.join("native-gc.c"))
        .arg(base.join("root-probe.c"))
        .arg("-o").arg(&binary).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(binary).output().unwrap();
    assert!(r.status.success(), "{r:?}");
    fs::remove_dir_all(dir).unwrap();
}


#[test]
fn native_gc_requires_explicit_heap_contract_and_distinct_mode() {
    for flags in [
        vec!["--native-gc"],
        vec!["--native-gc", "--native-gc"],
        vec!["--native-gc", "--probe-stack-roots"],
        vec!["--native-gc", "--reference-arena"],
    ] {
        let r = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
            .args(flags).output().unwrap();
        assert!(!r.status.success());
        let error = String::from_utf8_lossy(&r.stderr);
        assert!(error.contains("requires") || error.contains("duplicate") || error.contains("distinct modes"), "{error}");
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn reserved_record_array_kernel_validates_extents_and_traces_initialized_elements() {
    let dir = std::env::temp_dir().join(format!("neoclr-record-array-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for gc in [false, true] {
        let binary = dir.join(if gc { "gc" } else { "arena" });
        let mut command = Command::new("clang");
        command.args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
            .arg(base.join("record-array-test.c")).arg(base.join("text-arena.c"));
        if gc { command.arg("-DNEOCLR_NATIVE_GC").arg(base.join("native-gc.c")).arg(base.join("root-probe.c")); }
        let r = command.arg("-o").arg(&binary).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(binary).output().unwrap();
        assert!(r.status.success(), "{r:?}");
    }
    fs::remove_dir_all(dir).unwrap();
}
