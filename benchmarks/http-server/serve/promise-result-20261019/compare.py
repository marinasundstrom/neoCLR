#!/usr/bin/env python3
"""Matched synchronous-GC HTTP load for Promise result storage."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--baseline-build', type=Path, required=True)
p.add_argument('--candidate-build', type=Path, required=True)
p.add_argument('--workload', choices=['plain', 'bulk'], default='plain')
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rates', type=int, nargs='+', default=[200, 300])
p.add_argument('--duration', type=int, default=30)
a = p.parse_args()
a.output = a.output.resolve()
a.output.mkdir(parents=True, exist_ok=False)
runner = ROOT / 'benchmarks/http-server/load/run.py'
fixtures = {'baseline': (a.baseline_build.resolve(), 'app-paced', 'neoclr'),
            'candidate': (a.candidate_build.resolve(), 'app-paced', 'neoclr')}
files = [Path(__file__), runner]
for build, executable, runtime in fixtures.values():
    files.extend([build / executable, build / 'build.json'])
inputs = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
report = dict(completed=False, inputs=inputs, workload=a.workload, runs=[],
    contract='Same Serve source, compiler, native runtime and 1 MiB heap; Promise result storage changes. Paced synchronous GC, 16 server lanes, 64 client slots, connection-close. Reverse arms and rates in round two. No profiler or build during timing. No maximum-capacity claim.')
env = dict(os.environ)
try:
    for index, order in enumerate([['baseline', 'candidate'], ['candidate', 'baseline']], 1):
        rates = a.rates if index == 1 else list(reversed(a.rates))
        for rate in rates:
            for name in order:
                build, executable, runtime = fixtures[name]
                out = a.output / f'{name}-{index}-{rate}'
                source = ROOT / 'benchmarks/http-server/serve/Server.rvn'
                command = [sys.executable, str(runner), '--runtime', runtime, '--build', str(build), '--executable', executable,
                    '--server-source', str(source), '--control-stop', '--workers', '16', '--concurrency', '64',
                    '--workload', a.workload, '--rates', str(rate), '--duration', str(a.duration), '--rounds', '1', '--output', str(out)]
                run = dict(mode=name, round=index, rate=rate, command=command,
                    environment={key: env.get(key) for key in ['NEOCLR_GC_SHADOW_REPORT']}, started=time.time())
                report['runs'].append(run)
                print(f'comparison {index} {name} {rate}', flush=True)
                result = subprocess.run(command, env=env)
                run.update(exitCode=result.returncode, finished=time.time())
                result.check_returncode()
                (a.output / 'comparison.json').write_text(json.dumps(report, indent=2) + '\n')
    for path, digest in inputs.items():
        assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
    report['completed'] = True
finally:
    (a.output / 'comparison.json').write_text(json.dumps(report, indent=2) + '\n')
