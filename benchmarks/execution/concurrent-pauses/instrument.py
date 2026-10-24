"""Time outer collector operations, including nested finish/recheck/capture work."""
import argparse
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--source', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
s = a.source.read_text()
helpers = '''
/* Experiment only. Outermost operations partition blocking intervals; nested
 * collector calls do not become separate pauses. Report is non-mutating. */
#include <time.h>
static _Thread_local unsigned pause_depth;
static _Thread_local uint64_t pause_data[74];
static uint64_t pause_now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
static void pause_record(uint64_t start, unsigned kind, int32_t status) {
    uint64_t ns = pause_now() - start;
    unsigned bucket = 0;
    for (uint64_t n = ns; n > 1; n >>= 1) bucket++;
    pause_data[bucket]++;
    pause_data[64]++;
    pause_data[65] += ns;
    if (ns > pause_data[66]) pause_data[66] = ns;
    pause_data[67] += status != 0;
    pause_data[68 + kind]++;
}
void experiment_gc_pauses(uint64_t *output) {
    memcpy(output, pause_data, sizeof(pause_data));
}
'''
# Insert after the standard includes: the first declaration is after all includes.
at = s.index('static _Thread_local neoclr_gc_trigger_profile')
s = s[:at] + helpers + s[at:]
wrappers = []
specs = [
 ('collect_requested', 'static ', 'neoclr_aot_context *context, const neoclr_probe_frame *head, unsigned reason', 'context, head, reason'),
 ('collect_idle', 'static ', 'neoclr_aot_context *context, const neoclr_probe_frame *boundary, unsigned reason', 'context, boundary, reason'),
 ('neoclr_gc_collect_conditional_v1', '', 'neoclr_aot_context *context,\n    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count', 'context, head, conditional, count'),
 ('neoclr_gc_complete_invocation_v1', '', 'neoclr_aot_context *context', 'context'),
 ('neoclr_gc_heap_release_v1', '', 'neoclr_aot_text_arena *arena', 'arena'),
]
prototypes = []
for kind, (name, storage, arguments, call) in enumerate(specs):
    signature = storage + 'int32_t ' + name + '(' + arguments + ')'
    assert s.count(signature + ' {') == 1, name
    s = s.replace(signature + ' {', 'static int32_t pause_impl_' + name + '(' + arguments + ') {', 1)
    prototypes.append(signature + ';')
    wrappers.append(signature + ''' {
    int outer = pause_depth++ == 0;
    uint64_t start = outer ? pause_now() : 0;
    int32_t status = pause_impl_''' + name + '(' + call + ''');
    pause_depth--;
    if (outer) pause_record(start, ''' + str(kind) + ''', status);
    return status;
}
''')
# Original static declarations may occur earlier; identical prototypes are valid.
at = s.index('static _Thread_local neoclr_gc_trigger_profile')
s = s[:at] + '\n'.join(prototypes) + '\n' + s[at:]
s += '\n' + '\n'.join(wrappers)
s = s.replace('static void candidate_wait(void) {', 'static void candidate_wait(void) {\n#ifdef NEOCLR_PAUSE_TEST_DELAY\n    struct timespec delay = {0, 5000000};\n    if (nanosleep(&delay, NULL)) abort();\n#endif', 1)
a.output.parent.mkdir(parents=True, exist_ok=True)
a.output.write_text(s)
