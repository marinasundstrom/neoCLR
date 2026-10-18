#!/usr/bin/env python3
"""Reuse the native walker on worker-private snapshots; with experimental owner-side reclamation."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
WORKER = HERE.parent / 'concurrent-gc-worker'
SOURCE = ROOT / 'runtime/native/gc/native-gc.c'

def instrument(source):
    assert hashlib.sha256(source.encode()).hexdigest() == 'fefe1387031fa7c68bf9680e7abf34ce7d6b8ae66ec7187747b5b69ec813c13c', 'Review source changes before reusing the walker'
    def replace(old, new):
        nonlocal source
        assert source.count(old) == 1, old
        source = source.replace(old, new)
    # Share the unchanged cleanup/sweep body between synchronous and POC paths.
    start = source.index('    for (neoclr_gc_resource **link = &resource_head; *link;) {\n        neoclr_gc_resource *r = *link;\n        if (r->context == context && !(((const block *)r->owner - 1)->state & MARKED))')
    end = source.index('invalid_descriptor:', start)
    finish = source[start:end]
    source = source[:start] + '    return experiment_finish(context, conditional, conditional_count);\n' + source[end:]
    before = source.index('static int32_t collect_indexed(')
    source = source[:before] + 'static int32_t experiment_finish(neoclr_aot_context *context, neoclr_gc_conditional_root *conditional, uint32_t conditional_count) {\n    neoclr_aot_text_arena *a = &context->text;\n' + finish + '}\n' + source[before:]
    # Conservative owner-thread invalidation, including failed calls.
    import re
    functions = ['host_root_create',
        'neoclr_gc_host_root_replace_v1', 'neoclr_gc_host_root_release_v1',
        'neoclr_gc_resource_attach_v1', 'neoclr_gc_resource_release_v1',
        'neoclr_gc_complete_invocation_v1', 'neoclr_gc_entry_check_v1',
        'neoclr_gc_collect_conditional_v1', 'neoclr_gc_allocation_info_v1']
    for name in functions:
        pattern = r'((?:static )?int32_t ' + name + r'\([^;{]*\) \{)'
        source, count = re.subn(pattern, r'\1\n    experiment_invalidate();', source)
        assert count == 1, name
    # A successful unchanged strong-handle read exposes a value already captured
    # as a strong root. Creation/replacement/release still invalidate. Conditional
    # reads can resurrect snapshot-white values; failed reads remain conservative.
    read_prefix = ('int32_t neoclr_gc_host_root_read_v1(neoclr_aot_context *context, uint64_t handle, void **output) {\n'
                   '    host_root *root = find_host_root(context, handle);')
    replace(read_prefix, read_prefix + '\n    if (!root || root->conditional || root->expired || !output) experiment_invalidate();')
    # Callback reads validate a strong handle and callback shape but do not invoke
    # application code. Preserve successful reads; all rejected entries invalidate.
    callback_start = source.index('int32_t neoclr_gc_callback_read_v1(')
    callback_end = source.index('int32_t neoclr_gc_roots_park_owned_v1(', callback_start)
    callbacks = source[callback_start:callback_end]
    assert callbacks.count('return 3;') == 2
    assert callbacks.count('return callback_read(context, handle, output);') == 2
    callbacks = callbacks.replace('return 3;', '{ experiment_invalidate(); return 3; }')
    callbacks = callbacks.replace('return callback_read(context, handle, output);',
        'int32_t status = callback_read(context, handle, output);\n'
        '    if (status) experiment_invalidate();\n    return status;')
    source = source[:callback_start] + callbacks + source[callback_end:]
    source = source.replace('neoclr_roots_parked_v1()', 'experiment_parked_roots()')
    replace('static _Thread_local neoclr_gc_statistics statistics;', '''static _Thread_local neoclr_gc_statistics statistics;
static _Thread_local uint64_t experiment_epoch;
void experiment_invalidate(void) {
    if (experiment_epoch == UINT64_MAX) abort();
    experiment_epoch++;
}
static _Thread_local int experiment_snapshot;
static _Thread_local uintptr_t experiment_source;
static _Thread_local const atomic_bool *experiment_cancel;
/* Deterministic test interleaving; disabled for ordinary jobs. */
static _Thread_local atomic_bool *experiment_midwalk, *experiment_resume;
static _Thread_local unsigned experiment_steps;
static _Thread_local neoclr_parked_roots *experiment_parked;
static const neoclr_parked_roots *experiment_parked_roots(void) {
    return experiment_snapshot ? experiment_parked : neoclr_roots_parked_v1();
}
static uintptr_t experiment_original(neoclr_aot_text_arena *a, uintptr_t value) {
    if (!experiment_snapshot || !value) return value;
    return experiment_source + (value - (uintptr_t)a->data);
}''')
    replace('    block *b = indexed_block(a, index, word);\n    if (b && b->state', '''    if (experiment_snapshot) {
        if (word < experiment_source || word - experiment_source >= a->used) return;
        word = (uintptr_t)a->data + (word - experiment_source);
    }
    block *b = indexed_block(a, index, word);
    if (b && b->state''')
    replace('mark(a, &index, (uintptr_t)conditional[i].value, &pending);',
            'mark(a, &index, experiment_original(a, (uintptr_t)conditional[i].value), &pending);')
    # Intern seed pointers come from the copied index, not copied payload words.
    replace('mark(a, &index, (uintptr_t)(b + 1), &pending);',
            'mark(a, &index, experiment_original(a, (uintptr_t)(b + 1)), &pending);')
    replace('mark(a, &index, (uintptr_t)(index.interns[i] + 1), &pending);',
            'mark(a, &index, experiment_original(a, (uintptr_t)(index.interns[i] + 1)), &pending);')
    replace('        uint64_t atomic_before = index.atomic_marks;', '''        if (experiment_cancel && atomic_load(experiment_cancel)) goto invalid_descriptor;
        uint64_t atomic_before = index.atomic_marks;''')
    replace('        while (pending) {\n            block *b = pending;', '''        while (pending) {
            if (experiment_cancel && atomic_load(experiment_cancel)) goto invalid_descriptor;
            if (experiment_midwalk && ++experiment_steps == 64) {
                atomic_store(experiment_midwalk, true);
                while (!atomic_load(experiment_resume) && !atomic_load(experiment_cancel)) sched_yield();
                if (atomic_load(experiment_cancel)) goto invalid_descriptor;
            }
            block *b = pending;''')
    replace('    return experiment_finish(context, conditional, conditional_count);', '''    if (experiment_snapshot) {
        for (uint32_t i = 0; i < conditional_count; i++)
            conditional[i].retained = conditional[i].owner &&
                (((const block *)conditional[i].owner - 1)->state & MARKED);
        return 0;
    }
    return experiment_finish(context, conditional, conditional_count);''')
    return '#include "worker.h"\n#include <stdlib.h>\n#include <sched.h>\n' + source


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    generated = out / 'native-gc.c'
    generated.write_text(instrument(SOURCE.read_text()))
    roots_source = ROOT / 'runtime/native/gc/root-probe.c'
    roots = roots_source.read_text()
    assert hashlib.sha256(roots.encode()).hexdigest() == 'a23a8401b56cb8222b8ba384cb132469171b3e607cdada9f7b662ef1db7e911a', 'Review root transition changes before applying invalidation hooks'
    import re
    for name in ['neoclr_roots_park_v1', 'neoclr_roots_resume_v1', 'neoclr_roots_discard_v1']:
        roots, count = re.subn(r'(void ' + name + r'\([^;{]*\) \{)', r'\1\n    experiment_invalidate();', roots)
        assert count == 1, name
    generated_roots = out / 'root-probe.c'
    generated_roots.write_text('extern void experiment_invalidate(void);\n' + roots)
    files = [SOURCE, generated, generated_roots, HERE / 'snapshot.inc', HERE / 'test.c', HERE / 'parked-tests.inc', HERE / 'publication-tests.inc', HERE / 'reclaim.inc', HERE / 'reclaim-tests.inc', HERE / 'callback-churn.inc', Path(__file__),
             WORKER / 'worker.c', WORKER / 'worker.h', ROOT / 'runtime/native/gc/root-probe.c']
    report = dict(passed=False, productionReady=False, platform=platform.platform(), commands=[],
                  inputs={str(f): hashlib.sha256(f.read_bytes()).hexdigest() for f in files})
    def run(command):
        command = list(map(str, command))
        try:
            r = subprocess.run(command, capture_output=True, text=True, timeout=60)
        except subprocess.TimeoutExpired:
            report['commands'].append(dict(command=command, timedOut=True))
            raise
        report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
        r.check_returncode()
    try:
        run(['clang', '-std=c11', '-O2', '-g', '-Wall', '-Wextra', '-Werror', '-pthread',
             '-fsanitize=undefined,bounds', '-I', out, '-I', HERE, '-I', WORKER,
             '-I', ROOT / 'runtime/native/gc', HERE / 'test.c', WORKER / 'worker.c',
             generated_roots, '-o', out / 'test'])
        run([out / 'test'])
        report['passed'] = True
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(report['commands'][-1]['stdout'], end='')

if __name__ == '__main__':
    main()
