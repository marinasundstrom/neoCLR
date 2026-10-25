"""Build explicit concurrent snapshot-mark / changed-block-remark candidate."""
from pathlib import Path
import argparse
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
base = Path(__file__).resolve().parent
subprocess.run(['python3', str(base/'shadow.py'), '--output', str(args.output)], check=True)
s = args.output.read_text()
helper = (base/'candidate.h').read_text()
s = s.replace('static int32_t collect_indexed(', helper + '\nstatic int32_t collect_indexed(', 1)
s = s.replace('    shadow_begin(context, conditional, conditional_count);', '''    if (candidate_phase == 1) {
        shadow_begin(context, conditional, conditional_count);
        for (uint64_t at = 0; at < a->used;) {
            block *b = (void *)(a->data + at);
            b->state &= ALLOCATED;
            at += b->span;
        }
        return 0;
    }
    if (candidate_phase == 2 && candidate_seed(context, &index, &pending))
        goto invalid_descriptor;''', 1)
s = s.replace('    shadow_reference(context);', '', 1)
# shadow_reference is deliberately unused by the authoritative candidate.
s = s.replace('static void shadow_reference(neoclr_aot_context *context)', '__attribute__((unused)) static void shadow_reference(neoclr_aot_context *context)', 1)
s = s.replace('    if (!context || gc_state.cleanup_active) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;\n    block *starts[256]', '''    if (!context || gc_state.cleanup_active || (candidate_context &&
        (!candidate_phase || candidate_context != context))) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    block *starts[256]''', 1)
# Release must not throw away marks that finish is still expected to consume.
s = s.replace('static void shadow_stop(void);', 'static void shadow_stop(void);\nstatic _Thread_local int candidate_active;', 1)
s = s.replace('    shadow_stop();\n    if (!arena', '    if (candidate_active) return 3;\n    shadow_stop();\n    if (!arena', 1)
s = s.replace('void experiment_gc_shadow(uint64_t *output) {', 'void experiment_gc_shadow(uint64_t *output) {\n    if (candidate_active) abort();', 1)
s += '''
int32_t experiment_gc_begin(neoclr_aot_context *context, const neoclr_probe_frame *head,
    neoclr_gc_conditional_root *conditional, uint32_t count) {
    if (!context || candidate_active || gc_state.cleanup_active) return 3;
    candidate_context = context;
    candidate_phase = 1;
    int32_t status = neoclr_gc_collect_conditional_v1(context, head, conditional, count);
    candidate_phase = 0;
    if (status) candidate_context = NULL;
    else candidate_active = 1;
    return status;
}
int32_t experiment_gc_finish(neoclr_aot_context *context, const neoclr_probe_frame *head,
    neoclr_gc_conditional_root *conditional, uint32_t count) {
    if (!candidate_active || context != candidate_context || gc_state.cleanup_active) return 3;
    candidate_wait();
    candidate_phase = 2;
    int32_t status = neoclr_gc_collect_conditional_v1(context, head, conditional, count);
    candidate_phase = 0;
    candidate_context = NULL;
    candidate_active = 0;
    candidate_dispose();
    return status;
}
'''
args.output.write_text(s)
