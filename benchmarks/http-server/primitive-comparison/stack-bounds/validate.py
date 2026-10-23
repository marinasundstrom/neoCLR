#!/usr/bin/env python3
"""Check retained/OS stack bounds, thread isolation and mutable root-plan rejection."""
import argparse
import hashlib
import json
from pathlib import Path
import signal
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
base = root/'docs/experiments/aot-console'
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={})

def run(cmd, expected=0):
    r = subprocess.run(list(map(str, cmd)), capture_output=True, text=True, timeout=60)
    report['commands'].append(dict(command=list(map(str, cmd)), exitCode=r.returncode,
                                   expected=expected, stdout=r.stdout, stderr=r.stderr))
    assert r.returncode == expected and 'runtime error:' not in r.stderr, report['commands'][-1]

try:
    for sanitize in [False, True]:
        flags = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-pthread', '-I', base]
        if sanitize: flags += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
        binary = out/f'stack-{sanitize}'
        run([*flags, base/'native-stack-test.c', base/'native-stack.c', '-o', binary])
        run([binary])
        for gc in [False, True]:
            binary = out/f'roots-{gc}-{sanitize}'
            sources = [base/'root-validation-test.c', base/'root-probe.c']
            if gc: sources = ['-DNEOCLR_NATIVE_GC', *sources, base/'native-gc.c']
            run([*flags, *sources, '-o', binary])
            for case in ['plan-missing', 'plan-hint-missing', 'plan-length', 'plan-short', 'plan-moved']:
                run([binary, case], 0 if case == 'plan-moved' else -signal.SIGABRT)
    paths = [Path(__file__), base/'native-stack.c', base/'native-stack.h', base/'native-stack-test.c',
             base/'root-validation-test.c', *(root/'runtime/native').rglob('*.[ch]')]
    report['inputs'] = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report, indent=2)+'\n')
print('Stack bounds and root-plan validation passed')
