use std::{fs, path::PathBuf, process::Command};
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_probes_observe_real_spills_across_calls_faults_and_reentry() {
    let dir = Temp(std::env::temp_dir().join(format!("neoclr-root-probes-{}", std::process::id())));
    fs::create_dir(&dir.0).unwrap();
    let input = dir.0.join("input.neoil");
    fs::write(
        &input,
        r#".module Probes
.function Id(Value value) -> Value
ldarg value
ret
.end
.function Echo(Value value) -> Value
ldarg value
call Id(Value)
ret
.end
.function Main(Int32 fail) -> Int32
ldc.i8 99
ldstr "keep"
ldstr "argument"
value.pack String
call Echo(Value)
pop
pop
pop
ldarg fail
brfalse Done
ldc.i4 1
ldc.i4 0
div
ret
Done:
ldc.i4 42
ret
.end
"#,
    )
    .unwrap();
    for probe in [false, true] {
        let output = dir.0.join(if probe { "probe.o" } else { "plain.o" });
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
        cmd.arg(&input)
            .arg("Main")
            .arg(&output)
            .arg("--fault-details");
        if probe {
            cmd.arg("--probe-stack-roots");
        }
        let r = cmd.output().unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let imports = Command::new("nm").arg("-u").arg(&output).output().unwrap();
        assert!(imports.status.success());
        assert_eq!(
            String::from_utf8_lossy(&imports.stdout).contains("neoclr_probe_stack_roots_v1"),
            probe
        );
    }
    let host = dir.0.join("host.c");
    fs::write(&host, r#"
#include "fault-details.h"
#include <stdlib.h>
#include <string.h>
static unsigned calls;
static int text(uint64_t value, const char *expected) {
    const neoclr_aot_text *t = (const void *)(uintptr_t)value;
    return t && t->length == strlen(expected) && !memcmp(t->bytes, expected, t->length);
}
void neoclr_probe_stack_roots_v1(uint32_t function, uint32_t pc,
    const uint64_t *lanes, uint32_t count, const char *plan, uint32_t length) {
    if (!plan || strlen(plan) != length || !strstr(plan, "requiredSpillLanes")) abort();
    if (function == 2 && pc == 1) {
        if (count != 1 || lanes[0]) abort();
    } else if (function == 2 && pc == 2) {
        if (count != 2 || lanes[0] || !text(lanes[1], "keep")) abort();
    } else if (function == 2 && pc == 4) {
        if (count != 4 || lanes[0] || !text(lanes[1], "keep") || lanes[2] != 4 || !text(lanes[3], "argument")) abort();
    } else if (function == 1 && pc == 1) {
        if (count != 2 || lanes[0] != 4 || !text(lanes[1], "argument")) abort();
    } else { abort(); }
    calls++;
}
int main(void) {
    neoclr_aot_fault fault = {0};
    for (int i = 0; i < 3; i++) {
        int32_t result = -99;
        int32_t status = neoclr_entry_v3(i == 1, &result, &fault);
        if (i == 1) {
            if (status != 1 || result != -99 || fault.code != 1 || fault.frame_count != 1) return 1;
        } else if (status || result != 42 || fault.code || fault.frame_count) return 2;
    }
    return calls == 12 ? 0 : 3;
}
"#).unwrap();
    let binary = dir.0.join("app");
    let r = Command::new("clang")
        .args([
            "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/experiments/aot-fault-details"),
        )
        .arg(host)
        .arg(dir.0.join("probe.o"))
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = Command::new(&binary)
        .env_clear()
        .current_dir(&dir.0)
        .output()
        .unwrap();
    assert!(r.status.success(), "{r:?}");
}

#[test]
fn probes_require_context_and_reject_duplicates() {
    for flags in [
        vec!["--probe-stack-roots"],
        vec!["--probe-stack-roots", "--probe-stack-roots"],
    ] {
        let r = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
            .args(flags)
            .output()
            .unwrap();
        assert!(!r.status.success());
        assert!(String::from_utf8_lossy(&r.stderr).contains("--probe-stack-roots"));
    }
}
