#!/usr/bin/env python3
"""Fixed-rate burst/recovery comparison for compiled callbacks."""
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
BASE = POC.parent / 'aot-console'

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiled-build', type=Path, required=True)
p.add_argument('--walker-build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rounds', type=int, choices=[21000], default=21000)
p.add_argument('--live-count', type=int, default=4096)
p.add_argument('--callbacks', type=int, choices=[8], default=8)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build, compiled = a.walker_build.resolve(), a.compiled_build.resolve()
qualification = json.loads((compiled / 'results.json').read_text())
assert qualification['passed'] and qualification['qualification']['passed']
assert json.loads((build / 'results.json').read_text()) == qualification['qualification']
inputs = {**qualification['inputs'], **qualification['qualification']['inputs']}
inputs.update({str(compiled / n): h for n, h in qualification['artifacts'].items()})
inputs.update({str(f): sha(f) for f in [Path(__file__), HERE / 'compiled-callback.c', HERE / 'compiled-load.c']})
for name, digest in inputs.items():
    assert sha(Path(name)) == digest, name
report = dict(passed=False, platform=platform.platform(), loadBefore=os.getloadavg(),
              qualification=qualification, inputs=inputs, commands=[], trials=[])
def run(command):
    try:
        r = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=60)
    except subprocess.TimeoutExpired:
        report['commands'].append(dict(command=list(map(str, command)), timedOut=True))
        raise
    report['commands'].append(dict(command=r.args, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    return r.stdout
try:
    command = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-pthread',
               '-DNEOCLR_NATIVE_GC', '-I', build, '-I', POC, '-I', WORKER,
               '-I', ROOT / 'runtime/native/gc', '-I', BASE,
               HERE / 'compiled-load.c', WORKER / 'worker.c', compiled / 'root-probe.c',
               BASE / 'text-arena.c', BASE.parent / 'aot-fault-details/render.c', compiled / 'guest.o']
    run([*command, '-fsanitize=undefined,bounds', '-o', out / 'checked'])
    for mode in ['sync', 'concurrent']:
        run([out / 'checked', mode, 21000, 0, 64, a.callbacks])
    run([*command, '-o', out / 'bench'])
    for work in [0]:
        for mode in ['sync', 'concurrent', 'concurrent', 'sync']:
            lines = run([out / 'bench', mode, a.rounds, work, a.live_count, a.callbacks]).splitlines()
            trial = json.loads(lines[0])
            trial['load'] = json.loads(lines[1])
            assert trial['load']['offeredBatches'] == a.rounds
            assert sum(b['completedBatches'] for b in trial['load']['bins']) == a.rounds
            report['trials'].append(trial)
            print(mode, work, 'elapsed', round(trial['elapsedNs']/1e6, 2), 'ms CPU', round(trial['cpuNs']/1e6, 2), flush=True)
        group = report['trials'][-4:]
        for key in ['checksum', 'allocations', 'collections', 'finalArenaBytes']:
            assert len({t[key] for t in group}) == 1, key
    for name, digest in inputs.items():
        assert sha(Path(name)) == digest, name
    report['passed'] = True
finally:
    report['loadAfter'] = os.getloadavg()
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
