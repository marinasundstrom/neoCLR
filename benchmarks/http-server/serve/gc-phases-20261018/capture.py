#!/usr/bin/env python3
"""Sample native collection phases in fixed Serve; not a capacity benchmark."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def instrument(source):
    def replace(old, new):
        nonlocal source
        assert source.count(old) == 1, old
        source = source.replace(old, new)

    replace('    neoclr_aot_text_arena *a = &context->text;\n    if (conditional_count',
            '    phase_next(1);\n    neoclr_aot_text_arena *a = &context->text;\n    if (conditional_count')
    replace('    block *pending = NULL;\n    /* Header validation',
            '    phase_next(2);\n    block *pending = NULL;\n    /* Header validation')
    replace('    /* Each newly marked reference-bearing allocation',
            '    phase_next(3);\n    /* Each newly marked reference-bearing allocation')
    replace('    for (neoclr_gc_resource **link = &resource_head; *link;) {\n        neoclr_gc_resource *r = *link;\n        if (r->context == context && !(((const block *)r->owner - 1)->state & MARKED))',
            '    phase_next(4);\n    for (neoclr_gc_resource **link = &resource_head; *link;) {\n        neoclr_gc_resource *r = *link;\n        if (r->context == context && !(((const block *)r->owner - 1)->state & MARKED))')
    replace('    /* Sweep and coalesce in one forward walk.',
            '    phase_next(5);\n    /* Sweep and coalesce in one forward walk.')
    replace('    int32_t status = collect_conditional(context, head, conditional, count);',
            '    phase_begin();\n    int32_t status = collect_conditional(context, head, conditional, count);\n    phase_end(status);')
    return '#include "phases.h"\n' + source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--stride', type=int, default=64)
    parser.add_argument('--rate', type=float, default=300)
    parser.add_argument('--duration', type=float, default=20)
    args = parser.parse_args()
    assert args.stride > 0
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    manifest = args.build.resolve() / 'build.json'
    build = json.loads(manifest.read_text())
    assert build['passed']
    source = ROOT / 'runtime/native/gc/native-gc.c'
    generated = out / 'native-gc.c'
    generated.write_text(instrument(source.read_text()))
    command = next(c['command'][:] for c in reversed(build['commands']) if c['phase'] == 'host-compile-link')
    assert sum(x.endswith('/aot-console/native-gc.c') for x in command) == 1
    command = [str(generated) if x.endswith('/aot-console/native-gc.c') else x for x in command]
    command[command.index('-o') + 1] = str(out / 'app')
    command += ['-I', str(HERE), '-I', str(ROOT / 'runtime/native/gc'), f'-DGC_PHASE_SAMPLE={args.stride}']
    result = subprocess.run(command, capture_output=True, text=True)
    inputs = [Path(x) for x in command if Path(x).is_file()]
    inputs += [source, HERE / 'phases.h', Path(__file__), manifest]
    (out / 'build.json').write_text(json.dumps(dict(command=command, exitCode=result.returncode,
        stderr=result.stderr, inputs={str(x): hashlib.sha256(x.read_bytes()).hexdigest() for x in inputs}), indent=2) + '\n')
    result.check_returncode()
    subprocess.run(['python3', str(ROOT / 'benchmarks/http-server/serve/diagnose.py'),
        '--executable', str(out / 'app'), '--rate', str(args.rate), '--duration', str(args.duration),
        '--output', str(out / 'requests')], env=dict(os.environ, NEOCLR_GC_PHASES=str(out / 'phases.json')), check=True)


if __name__ == '__main__':
    main()
