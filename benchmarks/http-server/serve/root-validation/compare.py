#!/usr/bin/env python3
"""Alternate baseline/candidate Serve and Go with identical offered-load settings."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--baseline', type=Path, required=True)
p.add_argument('--candidate', type=Path, required=True)
p.add_argument('--go', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rates', type=float, nargs='+', default=[400, 500])
p.add_argument('--duration', type=float, default=30)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[4]
env = dict(os.environ, GOMAXPROCS='1', GOGC='100')
env.pop('GOMEMLIMIT', None)
report = dict(completed=False, runs=[])
try:
    for round_index, order in enumerate([['baseline','candidate','go'], ['go','candidate','baseline']], 1):
        rates = a.rates if round_index == 1 else list(reversed(a.rates))
        for label in order:
            runtime = 'go' if label == 'go' else 'neoclr'
            source = root/'benchmarks/http-server/serve'/('go/main.go' if label == 'go' else 'Server.rvn')
            command = [sys.executable, str(root/'benchmarks/http-server/load/run.py'),
                       '--runtime', runtime, '--build', str(getattr(a,label).resolve()),
                       '--executable', 'app' if label == 'go' else 'app-paced',
                       '--server-source', str(source), '--control-stop', '--workers', '16',
                       '--concurrency', '64', '--rates', *map(str,rates), '--duration', str(a.duration),
                       '--rounds', '1', '--output', str(a.output/f'{label}-{round_index}')]
            print(round_index, label, flush=True)
            result = subprocess.run(command, env=env)
            report['runs'].append(dict(label=label, round=round_index, command=command, exitCode=result.returncode))
            result.check_returncode()
    report['completed'] = True
finally:
    (a.output/'comparison.json').write_text(json.dumps(report, indent=2)+'\n')
