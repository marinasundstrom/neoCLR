#!/usr/bin/env python3
"""Qualify the private native conditional-root collector with a known host toolchain."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / 'docs/experiments/aot-console'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True, help='Passing async iteration validation report')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    recipe = json.loads(args.recipe.read_text())
    if not recipe['passed']:
        raise ValueError('Expected a passing host recipe')
    host = next(row['command'] for row in recipe['commands'] if row['label'] == 'host')
    cc = [host[0], '-isysroot', host[host.index('-isysroot') + 1],
          '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
          '-DNEOCLR_NATIVE_GC', '-fsanitize=undefined,bounds', '-I', str(BASE)]
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    inputs = [Path(__file__).resolve(), args.recipe.resolve(),
              *[BASE / name for name in ('native-gc.c', 'native-gc.h', 'native-gc-test.c',
                'host-roots-test.c', 'root-probe.c', 'root-probe.h', 'text-arena.h', 'native-stack.h')],
              ROOT / 'docs/experiments/aot-fault-details/fault-details.h']
    report = {'passed': False, 'commands': [], 'inputsSha256': {
        str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}}

    def run(label, command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=120)
        report['commands'].append(dict(label=label, command=list(map(str, command)),
            exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode or result.stderr:
            raise RuntimeError(f'{label}: {result.returncode}\n{result.stdout}\n{result.stderr}')

    try:
        run('toolchain', [host[0], '--version'])
        for name in ('native-gc-test', 'host-roots-test'):
            binary = out / name
            run(name + '-build', [*cc, BASE / (name + '.c'), BASE / 'native-gc.c',
                                 BASE / 'root-probe.c', '-pthread', '-o', binary])
            run(name + '-execute', [binary])
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'passed': True, 'report': str(out / 'validation.json')}))


if __name__ == '__main__':
    main()
