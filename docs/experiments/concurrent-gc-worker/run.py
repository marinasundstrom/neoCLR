#!/usr/bin/env python3
"""Build/run isolated POSIX GC-worker lifecycle tests with retained evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

here = Path(__file__).resolve().parent
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--sanitizer', choices=['address,undefined', 'undefined,bounds', 'thread'], default='undefined,bounds')
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, platform=platform.platform(), commands=[], inputs={
    name: hashlib.sha256((here / name).read_bytes()).hexdigest()
    for name in ['worker.h', 'worker.c', 'test.c', 'run.py']})

def run(command):
    command = list(map(str, command))
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=60)
    except subprocess.TimeoutExpired as error:
        report['commands'].append(dict(command=command, timedOut=True,
                                       stdout=str(error.stdout), stderr=str(error.stderr)))
        raise
    report['commands'].append(dict(command=result.args, exitCode=result.returncode,
                                   stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()

try:
    run(['clang', '-std=c11', '-O2', '-g', '-Wall', '-Wextra', '-Werror', '-pthread',
         f'-fsanitize={a.sanitizer}', here / 'worker.c', here / 'test.c', '-o', out / 'test'])
    run([out / 'test'])
    report['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(report['commands'][-1]['stdout'], end='')
