#!/usr/bin/env python3
"""Repeat timing on already validated binaries, after builds and tests have finished."""
import argparse
import hashlib
import json
import platform
import statistics
import subprocess
import time
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('validation', type=Path)
p.add_argument('output', type=Path)
a = p.parse_args()
validation = json.loads(a.validation.read_text())
commands = {}
for name in ('BenchmarkConcat', 'BenchmarkBuilder', 'BenchmarkJoin'):
    for mode in ('interpreted', 'native'):
        command = next(row['command'] for row in validation['commands'] if
                       (mode == 'native' and Path(row['command'][0]).name == name + '-standalone') or
                       (mode == 'interpreted' and len(row['command']) > 2 and row['command'][1] == 'run'
                        and Path(row['command'][2]).name == name + '.dll'))
        commands[name + '/' + mode] = command
# Revalidate binaries and their native metadata inputs; no compilation during timing.
for path, digest in validation['inputs'].items():
    if Path(path).suffix in ('.dll', '.neox', '.o') or any(path == c[0] for c in commands.values()):
        if hashlib.sha256(Path(path).read_bytes()).hexdigest() != digest:
            raise RuntimeError('Changed benchmark input: ' + path)

def run(command):
    start = time.perf_counter()
    result = subprocess.run(command, capture_output=True, timeout=180)
    elapsed = time.perf_counter() - start
    if result.returncode or result.stdout or result.stderr:
        raise RuntimeError(result)
    return elapsed

for command in commands.values():
    run(command)
samples = {key: [] for key in commands}
for iteration in range(5):
    for key in (list(commands) if iteration % 2 == 0 else list(reversed(commands))):
        samples[key].append(run(commands[key]))
report = {
    'validationSha256': hashlib.sha256(a.validation.read_bytes()).hexdigest(),
    'scriptSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'platform': platform.platform(),
    'hardware': subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip(),
    'scope': validation['timingScope'],
    'commands': commands,
    'timings': {key: {'seconds': values, 'medianSeconds': statistics.median(values)} for key, values in samples.items()}}
a.output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report['timings'], indent=2))
