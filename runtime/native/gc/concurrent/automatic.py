"""Schedule the admitted-writer live candidate; never substitute into unqualified hosts."""
import argparse
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
base = Path(__file__).resolve().parent
subprocess.run(['python3', base/'live.py','--output',args.output],check=True)
s = args.output.read_text()
def replace(old, new, count=1):
    global s
    assert s.count(old) == count, (old[:80], s.count(old), count)
    s = s.replace(old, new)
# Replace explicit-boundary completion with the scheduled protocol below.
old = "    if (candidate_active && !candidate_phase && context == candidate_context && !gc_state.cleanup_active)\n        return experiment_gc_finish(context,head,conditional,count);"
assert s.count(old) == 1
replace(old, '')

s = '#include <stdint.h>\n#ifndef NEOCLR_GC_PROFILE\n#error Experimental pacing requires profiling\n#endif\n#ifdef NEOCLR_WORKER_GC\n#error Shared heaps are not qualified\n#endif\n' + s
s = '#include <stdint.h>\nstatic _Thread_local int candidate_autostart;\nstatic void candidate_cancel(void);\nstatic _Thread_local uint64_t candidate_counts[5];\n' + s
replace('    if (candidate_active) return 3;\n    shadow_stop();', '    if (candidate_active) candidate_cancel();\n    shadow_stop();',1)
replace('    /* Removing pool ownership never frees an exposed String.', '    if (candidate_active) candidate_cancel();\n    /* Removing pool ownership never frees an exposed String.', 1)
# A successful public entry is followed by arena reset in generated code. Join
# the old marker first; rejected reentry must retain its current epoch/roots.
needle = '    return 0;\n}\nint32_t neoclr_gc_owned_roots_check_v1'
replace(needle, '    if (candidate_active) candidate_cancel(); /* admitted entry resets the arena */\n' + needle)
# Explicit/pressure operations finish and synchronously recheck the current
# context. Context changes discard detached state without freeing old-heap objects.
needle = 'int32_t neoclr_gc_collect_conditional_v1(neoclr_aot_context *context,\n    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {'
assert s.count(needle)==1
replace(needle, '''int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
static void candidate_cancel(void) {
    candidate_dispose();
    candidate_counts[2]++;
    candidate_context = NULL;
    candidate_active = 0;
}
''' + needle + '''
    if (!candidate_phase) {
        if (candidate_autostart && context && owned_heap.arena == &context->text) {
            if (candidate_active && candidate_context == context)
                return experiment_gc_finish(context, head, conditional, count);
            if (candidate_active) candidate_cancel();
            return experiment_gc_begin(context, head, conditional, count);
        }
        if (candidate_active) {
            if (candidate_context == context) {
                int32_t status = experiment_gc_finish(context, head, conditional, count);
                if (status) return status;
                candidate_counts[3]++;
            } else candidate_cancel();
        }
    }
''',1)
needle = '    int32_t status = collect_requested(context, boundary, reason);'
assert s.count(needle)==1
replace(needle, '''    /* Owned allocation/publication budgets toggle asynchronous epochs.
     * Other requests recheck synchronously, or cancel on context change. */
    candidate_autostart = reason == NEOCLR_GC_TRIGGER_PUBLICATION || reason == NEOCLR_GC_TRIGGER_ALLOCATION;
    int32_t status = collect_requested(context, boundary, reason);
    candidate_autostart = 0;
#ifdef NEOCLR_GC_PACED
    /* Preserve idle cleanup, then capture survivors for the next interval. */
    if (!status && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text) {
        status = experiment_gc_begin(context, boundary, NULL, 0);
        if (!status) candidate_counts[4]++;
    }
#endif
''',1)
assert s.count('        candidate_active = 1;') == 1
replace('        candidate_active = 1;', '        candidate_active = 1; candidate_counts[0]++;', 1)
replace('    candidate_dispose();\n    return status;', '    candidate_dispose();\n    if (!status) candidate_counts[1]++;\n    return status;', 1)
s += '\nvoid experiment_gc_candidate_counts(uint64_t *out) { memcpy(out, candidate_counts, sizeof(candidate_counts)); out[5] = candidate_active; }\n'
assert s.count('        candidate_active=0;') == 1
replace('        candidate_active=0;', '        candidate_active=0; candidate_counts[2]++;', 1)
s += r"""
/* Internal live-entry transition. Admission rejects all outstanding ownership;
 * the successful check joins the marker before any storage is invalidated. */
int32_t experiment_gc_prepare_entry_v1(neoclr_aot_context *context) {
    if (!context || owned_heap.arena != &context->text || gc_state.cleanup_active ||
        candidate_phase || !blocks_valid(&context->text)) return 3;
    int32_t status = neoclr_gc_entry_check_v1(context);
    if (status) return status;
    context->static_storage = NULL;
    context->text.used = 0;
    owned_heap.used = owned_heap.free_head = 0;
    memset(owned_heap.bases, 0, sizeof(owned_heap.bases));
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
    owned_index_reset();
#endif
    /* Keep aggregate statistics and unrelated root state; force fresh pacing. */
    gc_state.pacing.context = NULL;
    return 0;
}
"""
args.output.write_text(s)
