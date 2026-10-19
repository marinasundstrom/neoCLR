#!/usr/bin/env python3
"""Paired whole-process JSON input probe timing; not HTTP throughput."""
import argparse
import hashlib
import json
from pathlib import Path
import resource
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--baseline', type=Path, required=True)
p.add_argument('--candidate', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
assert not a.output.exists()
executables = {name: path.resolve() for name, path in [('baseline', a.baseline), ('candidate', a.candidate)]}
report = dict(scope='Whole process: 100 HTTP reads and canonical serialization checks for each of three fixtures and two storage kinds (600 reads). Includes setup, validation, runtime startup/shutdown. No server/socket throughput or GC-policy comparison.', inputs={name: dict(path=str(path), sha256=hashlib.sha256(path.read_bytes()).hexdigest()) for name, path in executables.items()}, runs=[], passed=False)
try:
    for round in range(4):
        for name in (['baseline', 'candidate'] if round % 2 == 0 else ['candidate', 'baseline']):
            before = resource.getrusage(resource.RUSAGE_CHILDREN)
            start = time.perf_counter()
            result = subprocess.run([str(executables[name])], capture_output=True, text=True, timeout=180)
            elapsed = time.perf_counter() - start
            after = resource.getrusage(resource.RUSAGE_CHILDREN)
            row = dict(round=round + 1, arm=name, elapsedSeconds=elapsed, cpuSeconds=(after.ru_utime + after.ru_stime) - (before.ru_utime + before.ru_stime), exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
            report['runs'].append(row)
            print(name, row['cpuSeconds'], elapsed, flush=True)
            assert result.returncode == 0 and result.stdout == 'PASS JSON byte input probe\n'
    report['passed'] = True
finally:
    a.output.write_text(json.dumps(report, indent=2) + '\n')
