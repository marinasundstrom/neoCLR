"""Generate diagnostic-only phase timing; no production collector edits."""
from pathlib import Path
import argparse
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
s=Path('runtime/native/gc/native-gc.c').read_text();at=s.index('static int32_t collect_indexed(')
s=s[:at]+'''/* Experiment only: wall-clock phase totals, not a concurrent collector. */
static _Thread_local uint64_t phase_values[6], phase_tick;
static _Thread_local unsigned phase_current;
static uint64_t phase_now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
static void phase_start(void) { phase_current=0; phase_tick=phase_now(); }
static void phase_next(unsigned next) {
    uint64_t now=phase_now(); phase_values[phase_current]+=now-phase_tick;
    phase_current=next; phase_tick=now;
}
static void phase_stop(void) { phase_next(0); phase_values[5]++; }
void experiment_gc_phases(uint64_t *output) { memcpy(output,phase_values,sizeof(phase_values)); }
''' +s[at:]
s=s.replace('    neoclr_aot_text_arena *a = &context->text;\n    if (conditional_count', '    phase_next(1);\n    neoclr_aot_text_arena *a = &context->text;\n    if (conditional_count',1)
s=s.replace('    /* Each newly marked reference-bearing allocation enters the worklist once.', '    phase_next(2);\n    /* Each newly marked reference-bearing allocation enters the worklist once.',1)
s=s.replace('    for (neoclr_gc_resource **link = &resource_head; *link;) {\n        neoclr_gc_resource *r = *link;', '    phase_next(3);\n    for (neoclr_gc_resource **link = &resource_head; *link;) {\n        neoclr_gc_resource *r = *link;',1)
s=s.replace('    /* Sweep and coalesce in one forward walk.', '    phase_next(4);\n    /* Sweep and coalesce in one forward walk.',1)
s=s.replace('    return 0;\ninvalid_descriptor:', '    phase_stop();\n    return 0;\ninvalid_descriptor:',1)
s=s.replace('    if (!context || gc_state.cleanup_active) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;\n    block *starts[256]', '    if (!context || gc_state.cleanup_active) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;\n    phase_start();\n    block *starts[256]',1)
a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(s)
