#!/usr/bin/env python3
"""Run qualified offered-load callbacks alongside bounded, owned CPU competitors."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import select
import subprocess
import time

HERE = Path(__file__).resolve().parent
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--load-build', type=Path, required=True)
p.add_argument('--competitors', type=int, choices=[0, 4, 8], required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
prior = json.loads((a.load_build.resolve() / 'results.json').read_text())
assert prior['passed']
def sha(f):
    return hashlib.sha256(f.read_bytes()).hexdigest()
inputs = dict(prior['inputs'])
for f in [Path(__file__).resolve(), HERE / 'cpu-competitor.c']:
    inputs[str(f)] = sha(f)
for name, digest in inputs.items():
    assert sha(Path(name)) == digest, name
report = dict(passed=False, platform=platform.platform(), logicalCpus=os.cpu_count(),
              competitors=a.competitors, loadBefore=os.getloadavg(), qualification=prior,
              inputs=inputs, commands=[], trials=[])
def run(cmd):
    r = subprocess.run(list(map(str, cmd)), capture_output=True, text=True, timeout=60)
    report['commands'].append(dict(command=r.args, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    return r.stdout
try:
    # Rebuild from the exact qualified recipe instead of trusting an unhashed executable.
    command = next(c['command'] for c in prior['commands'] if c['command'][0] == 'clang' and c['command'][-1].endswith('/bench'))[:]
    command[-1] = str(out / 'bench')
    run(command)
    cc = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', HERE / 'cpu-competitor.c']
    run([*cc, '-fsanitize=undefined,bounds', '-o', out / 'competitor-checked'])
    run([out / 'competitor-checked', 100])
    run([*cc, '-o', out / 'competitor'])
    for mode in ['sync', 'concurrent', 'concurrent', 'sync']:
        workers = []
        trial = dict(mode=mode, competitorResults=[])
        report['trials'].append(trial)
        try:
            for _ in range(a.competitors):
                proc = subprocess.Popen([str(out / 'competitor'), '60000'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                workers.append(proc)
                assert select.select([proc.stdout], [], [], 5)[0], 'competitor readiness timeout'
                assert proc.stdout.readline().strip() == 'ready'
            time.sleep(0.25)  # Let the controlled workload settle before guest startup.
            lines = run([out / 'bench', mode, 21000, 0, 4096, 8]).splitlines()
            trial['benchmark'] = json.loads(lines[0])
            trial['load'] = json.loads(lines[1])
            assert all(proc.poll() is None for proc in workers), 'competitor exited during trial'
            assert sum(b['completedBatches'] for b in trial['load']['bins']) == 21000
        finally:
            for proc in workers:
                if proc.poll() is None: proc.terminate()
            for proc in workers:
                try:
                    stdout, stderr = proc.communicate(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill(); stdout, stderr = proc.communicate(timeout=5)
                trial['competitorResults'].append(dict(pid=proc.pid, exitCode=proc.returncode, stdout=stdout, stderr=stderr))
        for result in trial['competitorResults']:
            assert result['exitCode'] == 0 and not result['stderr']
            result['metrics'] = json.loads(result['stdout'])
        print(a.competitors, mode, 'burst', sum(b['completedBatches'] for b in trial['load']['bins'] if 2000 <= b['startMs'] < 3000), 'p99 ms', round(trial['load']['responseP99Ns']/1e6, 2), flush=True)
    for key in ['checksum', 'allocations', 'collections', 'finalArenaBytes']:
        assert len({t['benchmark'][key] for t in report['trials']}) == 1, key
    for name, digest in inputs.items():
        assert sha(Path(name)) == digest, name
    report['passed'] = True
finally:
    report['loadAfter'] = os.getloadavg()
    report['binaries'] = {name: sha(out / name) for name in ('bench', 'competitor') if (out / name).exists()}
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
