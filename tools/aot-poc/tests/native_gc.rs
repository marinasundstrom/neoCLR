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
