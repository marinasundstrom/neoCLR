#!/usr/bin/env python3
"""Matched offered HTTP load for the fresh baseline, incoming-only and readiness candidates and Go."""
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
p.add_argument('--previous-build', type=Path, required=True)
p.add_argument('--candidate-build', type=Path, required=True)
p.add_argument('--go-build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rates', type=int, nargs='+', default=[200, 300])
p.add_argument('--duration', type=int, default=30)
a = p.parse_args()
a.output = a.output.resolve()
a.output.mkdir(parents=True, exist_ok=False)
runner = ROOT / 'benchmarks/http-server/load/run.py'
fixtures = {'baseline': (a.baseline_build.resolve(), 'app-paced', 'neoclr'),
            'previous': (a.previous_build.resolve(), 'app-paced', 'neoclr'),
            'candidate': (a.candidate_build.resolve(), 'app-paced', 'neoclr'),
            'go': (a.go_build.resolve(), 'app', 'go')}
files = [Path(__file__), runner]
for build, executable, runtime in fixtures.values():
    files.extend([build / executable, build / 'build.json'])
inputs = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
report = dict(completed=False, inputs=inputs, runs=[],
    contract='Same freshly built Serve source, 16 server lanes, 64 client slots, connection-close ten-byte response with queued await; loopback. Each rate/runtime gets a fresh server. Reverse rate and runtime order on round two. Go GOMAXPROCS=1 GOGC=100. All neoCLR arms use paced synchronous GC. Previous has direct incoming construction; candidate relinks that exact guest with readiness waits. Baseline is the fresh pre-URI-change build. No maximum-capacity claim.')
env = dict(os.environ, GOMAXPROCS='1', GOGC='100')
env.pop('GOMEMLIMIT', None)
try:
    for index, order in enumerate([['baseline', 'previous', 'candidate', 'go'], ['go', 'candidate', 'previous', 'baseline']], 1):
        rates = a.rates if index == 1 else list(reversed(a.rates))
        for rate in rates:
            for name in order:
                build, executable, runtime = fixtures[name]
                out = a.output / f'{name}-{index}-{rate}'
                source = ROOT / ('benchmarks/http-server/serve/go/main.go' if runtime == 'go' else 'benchmarks/http-server/serve/Server.rvn')
                command = [sys.executable, str(runner), '--runtime', runtime, '--build', str(build), '--executable', executable,
                    '--server-source', str(source), '--control-stop', '--workers', '16', '--concurrency', '64',
                    '--rates', str(rate), '--duration', str(a.duration), '--rounds', '1', '--output', str(out)]
                run = dict(mode=name, round=index, rate=rate, command=command,
                    environment={key: env.get(key) for key in ['GOMAXPROCS', 'GOGC', 'GOMEMLIMIT', 'NEOCLR_GC_SHADOW_REPORT']}, started=time.time())
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
