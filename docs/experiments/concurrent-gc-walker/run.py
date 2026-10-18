#!/usr/bin/env python3
"""Reuse the native walker on worker-private snapshots; never reclaim live objects."""
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
    replace('static _Thread_local neoclr_gc_statistics statistics;', '''static _Thread_local neoclr_gc_statistics statistics;
static _Thread_local int experiment_snapshot;
static _Thread_local uintptr_t experiment_source;
static _Thread_local const atomic_bool *experiment_cancel;
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
            block *b = pending;''')
    replace('    for (neoclr_gc_resource **link = &resource_head; *link;) {\n        neoclr_gc_resource *r = *link;\n        if (r->context == context && !(((const block *)r->owner - 1)->state & MARKED))', '''    if (experiment_snapshot) {
        for (uint32_t i = 0; i < conditional_count; i++)
            conditional[i].retained = conditional[i].owner &&
                (((const block *)conditional[i].owner - 1)->state & MARKED);
        return 0; /* No cleanup, expiry or sweep on the worker. */
    }
    for (neoclr_gc_resource **link = &resource_head; *link;) {
        neoclr_gc_resource *r = *link;
        if (r->context == context && !(((const block *)r->owner - 1)->state & MARKED))''')
    return '#include "worker.h"\n' + source


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    generated = out / 'native-gc.c'
    generated.write_text(instrument(SOURCE.read_text()))
    files = [SOURCE, generated, HERE / 'snapshot.inc', HERE / 'test.c', Path(__file__),
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
             ROOT / 'runtime/native/gc/root-probe.c', '-o', out / 'test'])
        run([out / 'test'])
        report['passed'] = True
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(report['commands'][-1]['stdout'], end='')

if __name__ == '__main__':
    main()
