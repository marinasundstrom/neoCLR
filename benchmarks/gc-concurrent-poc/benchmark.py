#!/usr/bin/env python3
"""Alternating process-isolated sync/concurrent POC comparisons on one host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
POC = ROOT / 'docs/experiments/concurrent-gc-walker'
WORKER = POC.parent / 'concurrent-gc-worker'
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--validated-build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--phase-timing', action='store_true')
p.add_argument('--compare-build', type=Path, help='Prior benchmark directory containing bench and results.json')
p.add_argument('--read-every', type=int, default=0)
p.add_argument('--churn-every', type=int, default=0)
p.add_argument('--host-roots', type=int, default=0)
p.add_argument('--live-count', type=int, default=512)
p.add_argument('--rounds', type=int, default=2000)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
build = a.validated_build.resolve()
qualification = json.loads((build / 'results.json').read_text())
assert qualification['passed']
for name, digest in qualification['inputs'].items():
    assert hashlib.sha256(Path(name).read_bytes()).hexdigest() == digest, name
command = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-pthread',
    '-I', str(build), '-I', str(POC), '-I', str(WORKER), '-I', str(ROOT / 'runtime/native/gc'),
    str(HERE / 'workload.c'), str(WORKER / 'worker.c'), str(build / 'root-probe.c'), '-o', str(out / 'bench')]
if a.phase_timing:
    command.insert(1, '-DPOC_PHASE_TIMING')
report = dict(passed=False, platform=platform.platform(), loadBefore=os.getloadavg(),
    commands=[], trials=[], qualification=qualification,
    inputs={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in [Path(__file__), HERE / 'workload.c']})
baseline = None
if a.compare_build:
    baseline = a.compare_build.resolve()
    prior = json.loads((baseline / 'results.json').read_text())
    assert prior['passed']
    assert prior['inputs'][str(HERE / 'workload.c')] == report['inputs'][str(HERE / 'workload.c')]
    report['baseline'] = prior
    report['baselineBinarySha256'] = hashlib.sha256((baseline / 'bench').read_bytes()).hexdigest()
try:
    r = subprocess.run(command, capture_output=True, text=True, timeout=60)
    report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    for work in [0, 50000]:
        order = [('candidate', mode) for mode in ['sync', 'concurrent', 'concurrent', 'sync']]
        if baseline:
            first = [('baseline', 'sync'), ('candidate', 'sync'),
                     ('baseline', 'concurrent'), ('candidate', 'concurrent')]
            order = first + list(reversed(first))
        for version, mode in order:
            binary = (baseline if version == 'baseline' else out) / 'bench'
            command = [str(binary), mode, str(a.rounds), str(work), str(a.live_count), str(a.host_roots), str(a.churn_every), str(a.read_every)]
            r = subprocess.run(command, capture_output=True, text=True, timeout=60)
            report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
            r.check_returncode()
            trial = json.loads(r.stdout)
            trial['version'] = version
            report['trials'].append(trial)
            print(version, mode, work, 'elapsed', round(trial['elapsedNs']/1e6, 2), 'ms CPU', round(trial['cpuNs']/1e6, 2), flush=True)
        group = report['trials'][-len(order):]
        assert len({t['checksum'] for t in group}) == 1
        assert len({t['collections'] for t in group}) == 1
        assert len({t['allocations'] for t in group}) == 1
    report['passed'] = True
finally:
    report['loadAfter'] = os.getloadavg()
    (out / 'results.json').write_text(json.dumps(report, indent=2)+'\n')
