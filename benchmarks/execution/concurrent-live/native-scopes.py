"""Generate a bounded synchronous native-call fallback for the owned live candidate."""
import argparse
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent
subprocess.run(['python3',base/'automatic.py','--output',a.output],check=True)
s=a.output.read_text()
def replace(old,new):
    global s
    assert s.count(old)==1,(old,s.count(old))
    s=s.replace(old,new)
replace('#include "execution-state.h"','''#include "execution-state.h"
/* Private owned-thread fallback. Tokens enforce nesting without heap allocation. */
static _Thread_local neoclr_aot_context *native_scope_context;
static _Thread_local uint64_t native_scope_tokens[64], native_scope_serial;
static _Thread_local uint32_t native_scope_depth;
static _Thread_local uint64_t native_scope_counts[4];''')
replace('int32_t neoclr_gc_heap_release_v1(neoclr_aot_text_arena *arena) {','''int32_t neoclr_gc_heap_release_v1(neoclr_aot_text_arena *arena) {
    if (native_scope_depth) return 3;''')
replace('if (candidate_autostart && context && owned_heap.arena == &context->text)', 'if (candidate_autostart && !native_scope_depth && context && owned_heap.arena == &context->text)')
replace('if (!status && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)', 'if (!status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)')
replace('if (!context || candidate_active || gc_state.cleanup_active) return 3;', 'if (!context || native_scope_depth || candidate_active || gc_state.cleanup_active) return 3;')
needle='''int32_t neoclr_gc_collect_conditional_v1(neoclr_aot_context *context,
    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {'''
replace(needle,needle+'''
    if (native_scope_depth) {
        if (context != native_scope_context) return 3;
        native_scope_counts[2]++;
    }''')
replace('NEOCLR_LIVE_REQUIRED | NEOCLR_LIVE_BOUNDARY_FINISH)', 'NEOCLR_LIVE_REQUIRED | NEOCLR_LIVE_BOUNDARY_FINISH | UINT64_C(16))')
s+='''
/* Entry cancels without collection: native inputs/results may be unpublished.
 * Collection explicitly requested inside a scope uses existing synchronous roots.
 * Leave never collects or starts an epoch before the caller publishes its result. */
int32_t experiment_gc_native_enter_v1(neoclr_aot_context *context, uint64_t *token) {
    const neoclr_probe_frame *head = neoclr_root_probe_head_v1();
    if (!context || !token || !head || head->context != context ||
        owned_heap.arena != &context->text || gc_state.cleanup_active || candidate_phase || native_scope_depth == 64 ||
        native_scope_serial == UINT64_MAX ||
        (native_scope_depth && native_scope_context != context)) return 3;
    if (candidate_active) {
        candidate_cancel();
        native_scope_counts[1]++;
    }
    native_scope_context = context;
    uint64_t next = ++native_scope_serial;
    native_scope_tokens[native_scope_depth++] = next;
    if (native_scope_depth > native_scope_counts[3]) native_scope_counts[3] = native_scope_depth;
    native_scope_counts[0]++;
    *token = next;
    return 0;
}
int32_t experiment_gc_native_leave_v1(neoclr_aot_context *context, uint64_t token) {
    if (!native_scope_depth || context != native_scope_context ||
        owned_heap.arena != &context->text || !token ||
        native_scope_tokens[native_scope_depth-1] != token) return 3;
    native_scope_tokens[--native_scope_depth] = 0;
    if (!native_scope_depth) native_scope_context = NULL;
    return 0;
}
void experiment_gc_native_scope_counts(uint64_t *out) {
    memcpy(out,native_scope_counts,sizeof(native_scope_counts));
    out[4] = native_scope_depth;
}
'''
s+='''
/* Exact synchronous leaf adapter; never use this to wrap an async pump.
 * The generated caller supplies stack result storage and publishes after return. */
int32_t experiment_gc_string_slice_utf8_rooted_v1(const neoclr_aot_text *text,
    int32_t start, int32_t length, neoclr_aot_text_arena *arena, void *output) {
    const neoclr_probe_frame *head = neoclr_root_probe_head_v1();
    if (!head || !head->context || !output) return 3;
    neoclr_aot_context *context = (neoclr_aot_context *)head->context;
    if (&context->text != arena) return 3;
    uint64_t token;
    int32_t status = experiment_gc_native_enter_v1(context, &token);
    if (status) return status;
    status = neoclr_string_slice_utf8_rooted_v1(text, start, length, arena, output);
    int32_t leave_status = experiment_gc_native_leave_v1(context, token);
    return leave_status ? leave_status : status;
}
'''
a.output.write_text(s)
