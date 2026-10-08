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
        vec!["--native-stack-budget"],
        vec!["--native-stack-budget", "--native-stack-budget"],
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

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn scalar_array_kernel_keeps_numeric_payload_atomic_and_failure_output_unchanged() {
    let dir = std::env::temp_dir().join(format!("neoclr-scalar-array-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for gc in [false, true] {
        let binary = dir.join(if gc { "gc" } else { "arena" });
        let mut command = Command::new("clang");
        command.args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
            .arg(base.join("scalar-array-test.c")).arg(base.join("text-arena.c"));
        if gc { command.arg("-DNEOCLR_NATIVE_GC").arg(base.join("native-gc.c")).arg(base.join("root-probe.c")); }
        let r = command.arg("-o").arg(&binary).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(binary).output().unwrap();
        assert!(r.status.success(), "{r:?}");
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn string_join_kernel_validates_separators_and_atomic_failure() {
    let dir = std::env::temp_dir().join(format!("neoclr-string-join-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for gc in [false, true] {
        let binary = dir.join(if gc { "gc" } else { "arena" });
        let mut command = Command::new("clang");
        command.args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
            .arg(base.join("join-parts-test.c")).arg(base.join("text-arena.c"));
        if gc { command.arg("-DNEOCLR_NATIVE_GC").arg(base.join("native-gc.c")).arg(base.join("root-probe.c")); }
        let r = command.arg("-o").arg(&binary).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(binary).output().unwrap();
        assert!(r.status.success(), "{r:?}");
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn host_roots_retain_release_and_validate_context_thread_and_quota() {
    let dir = std::env::temp_dir().join(format!("neoclr-host-roots-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let binary = dir.join("heap");
    let r = Command::new("clang")
        .args(["-DNEOCLR_NATIVE_GC", "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
        .arg(base.join("host-roots-test.c"))
        .arg(base.join("native-gc.c"))
        .arg(base.join("root-probe.c"))
        .arg("-o").arg(&binary).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(binary).output().unwrap();
    assert!(r.status.success(), "{r:?}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_accept_defers_delivery_and_releases_completion_roots() {
    let dir = std::env::temp_dir().join(format!("neoclr-native-accept-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let binary = dir.join("heap");
    let r = Command::new("clang")
        .args(["-DNEOCLR_NATIVE_GC", "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
        .arg(base.join("socket-accept-test.c"))
        .arg(base.join("socket-listener.c"))
        .arg(base.join("text-arena.c"))
        .arg(base.join("native-gc.c"))
        .arg(base.join("root-probe.c"))
        .arg("-o").arg(&binary).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(binary).output().unwrap();
    assert!(r.status.success(), "{r:?}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_transfer_snapshots_sends_and_initializes_only_received_bytes() {
    let dir = std::env::temp_dir().join(format!("neoclr-native-transfer-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let binary = dir.join("heap");
    let r = Command::new("clang")
        .args(["-DNEOCLR_NATIVE_GC", "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
        .arg(base.join("socket-transfer-test.c"))
        .arg(base.join("socket-listener.c"))
        .arg(base.join("text-arena.c"))
        .arg(base.join("native-gc.c"))
        .arg(base.join("root-probe.c"))
        .arg("-o").arg(&binary).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(binary).output().unwrap();
    assert!(r.status.success(), "{r:?}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_task_scope_preserves_default_and_active_queue_ownership() {
    let dir = std::env::temp_dir().join(format!("neoclr-task-scope-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let binary = dir.join("heap");
    let r = Command::new("clang")
        .args(["-DNEOCLR_NATIVE_GC", "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
        .arg(base.join("task-queue-test.c"))
        .arg(base.join("task-queue.c"))
        .arg(base.join("text-arena.c"))
        .arg(base.join("native-gc.c"))
        .arg(base.join("root-probe.c"))
        .arg("-o").arg(&binary).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(binary).output().unwrap();
    assert!(r.status.success(), "{r:?}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_stack_probe_respects_worker_stack_bounds_and_unwinds() {
    let dir = std::env::temp_dir().join(format!("neoclr-native-stack-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for optimization in ["-O0", "-O2"] {
        let binary = dir.join(optimization);
        let r = Command::new("clang")
            .args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds", optimization])
            .arg(base.join("native-stack-test.c")).arg(base.join("native-stack.c"))
            .arg("-o").arg(&binary).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(binary).output().unwrap();
        assert!(r.status.success(), "{r:?}");
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_gc_sparse_index_preserves_interior_roots_and_rejects_header_padding() {
    let dir = std::env::temp_dir().join(format!("neoclr-native-gc-index-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let binary = dir.join("heap");
    let r = Command::new("clang")
        .args(["-arch", "arm64", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
        .arg(base.join("native-gc-index-test.c")).arg(base.join("native-gc.c")).arg(base.join("root-probe.c"))
        .arg("-o").arg(&binary).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(binary).output().unwrap();
    assert!(r.status.success(), "{r:?}");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn lexical_path_kernel_preserves_failure_outputs_and_owned_utf8() {
    let dir = std::env::temp_dir().join(format!("neoclr-path-text-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    for gc in [false, true] {
        let binary = dir.join(if gc { "gc" } else { "arena" });
        let mut command = Command::new("clang");
        command.args(["-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-fsanitize=undefined,bounds"])
            .arg(base.join("path-text-test.c")).arg(base.join("text-arena.c"));
        if gc { command.arg("-DNEOCLR_NATIVE_GC").arg(base.join("native-gc.c")).arg(base.join("root-probe.c")); }
        let r = command.arg("-o").arg(&binary).output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = Command::new(binary).output().unwrap();
        assert!(r.status.success(), "{r:?}");
    }
    fs::remove_dir_all(dir).unwrap();
}
