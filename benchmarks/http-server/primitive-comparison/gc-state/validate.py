#!/usr/bin/env python3
"""Qualify grouped GC state: pacing, thread isolation, cleanup and profile resets."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
base = root/'docs/experiments/aot-console'
a.output.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={})

def run(cmd):
    result = subprocess.run(list(map(str, cmd)), capture_output=True, text=True, timeout=90)
    report['commands'].append(dict(command=list(map(str, cmd)), exitCode=result.returncode,
                                   stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()

try:
    for fixture in ['native-gc-pacing-test', 'native-gc-test', 'native-gc-profile-test']:
        for sanitized in [False, True]:
            binary = a.output.resolve()/f'{fixture}-{sanitized}'
            sources = [base/(fixture+'.c'), base/'root-probe.c', base/'native-gc.c']
            cmd = ['xcrun', '--sdk', 'macosx', 'clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC']
            if fixture == 'native-gc-pacing-test':
                cmd += ['-DNEOCLR_GC_PACED', '-DNEOCLR_TEST_THREADS', '-pthread']
                sources += [base/'text-arena.c']
            if fixture == 'native-gc-profile-test': cmd += ['-DNEOCLR_GC_PROFILE']
            if sanitized: cmd += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
            run(cmd + sources + ['-o', binary])
            run([binary])
            for path in sources: report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for path in [Path(__file__).resolve(), * (root/'runtime/native').rglob('*.[ch]')]:
        report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    report['passed'] = True
finally:
    (a.output/'validation.json').write_text(json.dumps(report, indent=2)+'\n')
