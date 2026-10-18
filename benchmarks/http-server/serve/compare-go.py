#!/usr/bin/env python3
"""Alternate matched Serve/net-http offered-load runs; never infer Go maximum capacity."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--neoclr-build', type=Path, required=True)
p.add_argument('--go-build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--duration', type=float, default=30)
p.add_argument('--rates', type=float, nargs='+', default=[400, 500])
p.add_argument('--rounds', type=int, default=2)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[3]
runner = root/'benchmarks/http-server/load/run.py'
fixtures = dict(neoclr=(a.neoclr_build.resolve(), 'app-paced', Path(__file__).with_name('Server.rvn').resolve()),
                go=(a.go_build.resolve(), 'app', Path(__file__).with_name('go')/'main.go'))
inputs = [Path(__file__).resolve(), runner]
for build, executable, source in fixtures.values(): inputs.extend([build/executable, build/'build.json', source])
report = dict(completed=False, contract='Same host, ten x bytes, HTTP/1.1 connection close, queued operation in handler, 16 accepted-connection budget, 64 client slots. Go GOMAXPROCS=1 and GOGC=100; neoCLR experimental paced GC. Alternating entry order and reversing rate order on round two. Different runtime implementations, not an exact identical work instruction count. No Go maximum-capacity claim.',
              inputs={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}, runs=[])
env = dict(os.environ, GOMAXPROCS='1', GOGC='100')
env.pop('GOMEMLIMIT', None)
try:
    for round_index in range(a.rounds):
        order = ['neoclr', 'go'] if round_index % 2 == 0 else ['go', 'neoclr']
        rates = a.rates if round_index % 2 == 0 else list(reversed(a.rates))
        for runtime in order:
            build, executable, source = fixtures[runtime]
            out = a.output/f'{runtime}-{round_index+1}'
            command = [sys.executable, str(runner), '--runtime', runtime, '--build', str(build),
                       '--executable', executable, '--server-source', str(source), '--control-stop',
                       '--workers', '16', '--concurrency', '64', '--rates', *map(str, rates),
                       '--duration', str(a.duration), '--rounds', '1', '--output', str(out)]
            run = dict(round=round_index+1, runtime=runtime, command=command,
                       environment={key: env.get(key) for key in ['GOMAXPROCS','GOGC','GOMEMLIMIT']},
                       startedUnixSeconds=time.time())
            report['runs'].append(run)
            print('comparison', round_index+1, runtime, flush=True)
            result = subprocess.run(command, env=env)
            run.update(exitCode=result.returncode, finishedUnixSeconds=time.time())
            (a.output/'comparison.json').write_text(json.dumps(report, indent=2)+'\n')
            result.check_returncode()
    for path, digest in report['inputs'].items(): assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
    report['completed'] = True
finally:
    (a.output/'comparison.json').write_text(json.dumps(report, indent=2)+'\n')
