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
.function Id(Value value,Int32 fail) -> Value
ldarg fail
brfalse Success
ldc.i4 1
ldc.i4 0
div
pop
Success:
ldarg value
ret
.end
.function Echo(Value value,Int32 fail) -> Value
ldarg value
ldarg fail
call Id(Value,Int32)
ret
.end
.function Main(Int32 fail) -> Int32
.local String held
.local Value never
ldc.i8 99
ldstr "keep"
dup
stloc held
ldstr "argument"
value.pack String
ldarg fail
call Echo(Value,Int32)
pop
pop
pop
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
            String::from_utf8_lossy(&imports.stdout).contains("neoclr_probe_stack_roots_v2"),
            probe
        );
    }
    let host = dir.0.join("host.c");
    fs::write(&host, r#"
#include "fault-details.h"
#include "root-probe.h"
#include <stdlib.h>
#include <string.h>
static unsigned calls, enters, leaves, depth, peak, results;
static neoclr_probe_frame *head;
static int text(uint64_t value, const char *expected) {
    const neoclr_aot_text *t = (const void *)(uintptr_t)value;
    return t && t->length == strlen(expected) && !memcmp(t->bytes, expected, t->length);
}
void neoclr_probe_enter_v3(neoclr_probe_frame *frame, const void *context, uint32_t function,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (!frame || !context || (head && head->context != context)) abort();
    memset(frame, 0, sizeof(*frame));
    frame->previous = head; frame->context = context; frame->function = function;
    frame->storage = storage; frame->storage_count = count;
    frame->storage_plan = plan; frame->storage_length = length;
    if (!plan || strlen(plan) != length) abort();
    if (function == 2) {
        if (count != 3 || storage[0].read_bytes != 8 || *(const uint64_t *)storage[0].address ||
            storage[1].read_bytes != 4 || storage[1].flags != 1 || *(const uint32_t *)storage[1].address ||
            *(const uint64_t *)storage[2].address) abort();
    } else {
        if (count != 2 || storage[0].read_bytes != 4 || *(const uint32_t *)storage[0].address != 4 ||
            !text(*(const uint64_t *)storage[1].address, "argument")) abort();
    }
    head = frame; enters++; depth++; if (depth > peak) peak = depth;
    if (function == 0) {
        const neoclr_probe_frame *caller = frame->previous;
        if (!caller || caller->function != 1 || !caller->previous || caller->previous->function != 2 ||
            !text(caller->previous->lanes[1], "keep") ||
            !text(*(const uint64_t *)caller->previous->storage[0].address, "keep")) abort();
    }
}
void neoclr_probe_transient_v1(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (head != frame || phase != 2 || count != 2 || !plan || strlen(plan) != length ||
        storage[0].read_bytes != 4 || *(const uint32_t *)storage[0].address != 4 ||
        !text(*(const uint64_t *)storage[1].address, "argument")) abort();
    results++;
}
void neoclr_probe_leave_v1(neoclr_probe_frame *frame) {
    if (head != frame || !depth) abort();
    head = frame->previous; depth--; leaves++;
    memset(frame, 0, sizeof(*frame));
}
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *frame, uint32_t pc,
    const uint64_t *lanes, uint32_t count, const char *plan, uint32_t length) {
    if (head != frame) abort();
    uint32_t function = frame->function;
    frame->lanes = lanes; frame->lane_count = count; frame->plan = plan;
    frame->length = length; frame->instruction = pc;
    if (!plan || strlen(plan) != length || !strstr(plan, "requiredSpillLanes")) abort();
    if (function == 2 && pc == 1) {
        if (count != 1 || lanes[0]) abort();
    } else if (function == 2 && pc == 4) {
        if (count != 2 || lanes[0] || !text(lanes[1], "keep")) abort();
    } else if (function == 2 && pc == 7) {
        if (count != 5 || lanes[4] || lanes[0] || !text(lanes[1], "keep") || lanes[2] != 4 || !text(lanes[3], "argument")) abort();
    } else if (function == 1 && pc == 2) {
        if (count != 3 || lanes[2] || lanes[0] != 4 || !text(lanes[1], "argument")) abort();
    } else { abort(); }
    calls++;
}
int main(void) {
    neoclr_aot_fault fault = {0};
    for (int i = 0; i < 3; i++) {
        int32_t result = -99;
        int32_t status = neoclr_entry_v3(i == 1, &result, &fault);
        if (head || depth || enters != leaves) return 4;
        if (i == 1) {
            if (status != 1 || result != -99 || fault.code != 1 || fault.frame_count != 3) return 1;
        } else if (status || result != 42 || fault.code || fault.frame_count) return 2;
    }
    return calls == 12 && enters == 9 && leaves == 9 && peak == 3 && results == 4 ? 0 : 3;
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
        .arg("-I")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console"))
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

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn adapter_reads_slots_without_following_uninitialized_borrow_pointees() {
    let dir =
        Temp(std::env::temp_dir().join(format!("neoclr-probe-borrows-{}", std::process::id())));
    fs::create_dir(&dir.0).unwrap();
    let host = dir.0.join("host.c");
    fs::write(
        &host,
        r#"
#include "root-probe.h"
#include <string.h>
int main(void) {
    uint64_t inaccessible_pointee = 1;
    uint32_t tag = 4;
    neoclr_probe_storage storage[] = {{&inaccessible_pointee, 8, 2}, {&tag, 4, 1}};
    neoclr_probe_frame frame;
    uint64_t lane = 0;
    const char *plan = "{\"requiredSpillLanes\":[]}";
    neoclr_probe_enter_v3(&frame, &frame, 0, storage, 2, plan, (uint32_t)strlen(plan));
    neoclr_probe_stack_roots_v2(&frame, 0, &lane, 1, plan, (uint32_t)strlen(plan));
    if (neoclr_root_probe_head_v1() != &frame || neoclr_root_probe_depth_v1() != 1) return 1;
    neoclr_probe_leave_v1(&frame);
    return neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1();
}
"#,
    )
    .unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-console");
    let binary = dir.0.join("host");
    let r = Command::new("clang")
        .args([
            "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(&base)
        .arg(&host)
        .arg(base.join("root-probe.c"))
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(Command::new(binary).status().unwrap().success());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn constructor_storage_is_live_before_call_and_faults_never_publish_results() {
    let dir = Temp(std::env::temp_dir().join(format!("neoclr-probe-ctor-{}", std::process::id())));
    fs::create_dir(&dir.0).unwrap();
    let input = dir.0.join("input.neoil");
    fs::write(
        &input,
        r#".module ConstructorRoots
.type Payload
.field Text String
.method instance byref .ctor(Int32 fail) -> noresult
ldarg this
ldstr "ready"
stfld 0
pop
ldarg fail
brfalse Done
ldc.i4 1
ldc.i4 0
div
pop
Done:
ret
.end
.end
.function Main(Int32 fail) -> Int32
ldarg fail
newobj.ctor instance Payload::.ctor(Int32)
pop
ldc.i4 42
ret
.end
"#,
    )
    .unwrap();
    let object = dir.0.join("app.o");
    let r = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(input)
        .arg("Main")
        .arg(&object)
        .args(["--fault-details", "--probe-stack-roots"])
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let host = dir.0.join("host.c");
    fs::write(&host, r#"
#include "root-probe.h"
#include "fault-details.h"
#include <string.h>
#include <stdlib.h>
static neoclr_probe_frame *head;
static unsigned before, after;
static int ready(const neoclr_probe_storage *s) {
    const neoclr_aot_text *text = *(const neoclr_aot_text *const *)s->address;
    return text && text->length == 5 && !memcmp(text->bytes, "ready", 5);
}
void neoclr_probe_enter_v3(neoclr_probe_frame *f, const void *ctx, uint32_t fn,
    const neoclr_probe_storage *s, uint32_t n, const char *p, uint32_t len) {
    memset(f, 0, sizeof(*f)); f->previous = head; f->context = ctx; f->function = fn;
    f->storage = s; f->storage_count = n; f->storage_plan = p; f->storage_length = len; head = f;
}
void neoclr_probe_leave_v1(neoclr_probe_frame *f) {
    if (head != f) abort();
    if (f->previous && (f->previous->transient_phase != 1 || !ready(f->previous->transient))) abort();
    head = f->previous;
}
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *f, uint32_t pc, const uint64_t *s,
    uint32_t n, const char *p, uint32_t len) {
    f->instruction = pc; f->lanes = s; f->lane_count = n; f->plan = p; f->length = len;
    f->transient = NULL; f->transient_phase = 0; f->transient_count = 0;
}
void neoclr_probe_transient_v1(neoclr_probe_frame *f, uint32_t phase,
    const neoclr_probe_storage *s, uint32_t n, const char *p, uint32_t len) {
    if (head != f || n != 1 || s->read_bytes != 8 || !p || strlen(p) != len) abort();
    if (phase == 1) { if (*(const uint64_t *)s->address) abort(); before++; }
    else if (phase == 2) { if (!ready(s)) abort(); after++; }
    else abort();
    f->transient = s; f->transient_count = n; f->transient_phase = phase;
}
int main(void) {
    neoclr_aot_fault fault = {0};
    for (int i = 0; i < 2; i++) {
        int32_t result = -99;
        int status = neoclr_entry_v3(i, &result, &fault);
        if (head || status != i || (i ? result != -99 : result != 42)) return 1;
    }
    return before == 2 && after == 1 ? 0 : 2;
}
"#).unwrap();
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments");
    let binary = dir.0.join("host");
    let r = Command::new("clang")
        .args([
            "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(base.join("aot-console"))
        .arg("-I")
        .arg(base.join("aot-fault-details"))
        .arg(host)
        .arg(object)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(Command::new(binary).status().unwrap().success());
}
