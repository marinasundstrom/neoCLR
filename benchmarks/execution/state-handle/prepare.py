"""Generate an isolated explicit-state prototype from a fixed production revision."""
import argparse,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
s=subprocess.check_output(['git','show','efeea6a66:runtime/native/gc/root-probe.c'],text=True)
# Shared implementation retains all validation; only state access is parameterized.
for name,end in [('stack_roots','void neoclr_probe_stack_roots_v2'),('publish_transient','void neoclr_probe_transient_v2')]:
 start=s.index('static void '+name);stop=s.index(end,start);part=s[start:stop]
 part=part.replace('uint32_t map_flags) {','uint32_t map_flags, neoclr_execution_state *state) {').replace('uint32_t flags) {','uint32_t flags, neoclr_execution_state *state) {')
 part=part.replace('probe_state','state->roots').replace('&neoclr_execution_state_v1','state')
 s=s[:start]+part+s[stop:]
s=s.replace('stack_roots(frame, instruction, lanes, count, plan, length, 0);','stack_roots(frame, instruction, lanes, count, plan, length, 0, &neoclr_execution_state_v1);')
s=s.replace('stack_roots(frame, instruction, lanes, count, map, length, NEOCLR_GC_TYPED_PLAN);','stack_roots(frame, instruction, lanes, count, map, length, NEOCLR_GC_TYPED_PLAN, &neoclr_execution_state_v1);')
for flag in ['0','NEOCLR_GC_OWNED_TRANSIENT','NEOCLR_GC_OWNED_TRANSIENT | NEOCLR_GC_TRANSIENT_MAP']:
 for plan in ['plan','map']:
  old=f'publish_transient(frame, phase, storage, count, {plan}, length, {flag});'
  s=s.replace(old,old[:-2]+', &neoclr_execution_state_v1);')
s+='''
/* Experimental trusted generated-code contract: same owner thread for the
 * entire state-handle lifetime, including park/resume. Never export to managed
 * code, retain beyond thread lifetime, or pass through a migrating continuation.
 * Ordinary/native hooks above keep resolving and checking the current TLS state. */
void *experiment_state(void) { return &neoclr_execution_state_v1; }
void experiment_stack(void *handle, neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t count, const void *map, uint32_t length) {
    if (!handle) abort();
    stack_roots(frame, instruction, lanes, count, map, length, NEOCLR_GC_TYPED_PLAN, handle);
}
void experiment_transient(void *handle, neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const void *map, uint32_t length) {
    if (!handle) abort();
    publish_transient(frame, phase, storage, count, map, length,
        NEOCLR_GC_OWNED_TRANSIENT | NEOCLR_GC_TRANSIENT_MAP, handle);
}
void experiment_leave(void *handle, neoclr_probe_frame *frame) {
    neoclr_execution_state *state = handle;
    if (!state || !frame || state->roots.head != frame || !state->roots.depth) abort();
    observe_fault(frame->context);
    state->roots.head = frame->previous;
    state->roots.depth--;
    memset(frame, 0, sizeof(*frame));
}
'''
a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(s)
