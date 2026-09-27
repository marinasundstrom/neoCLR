#!/usr/bin/env python3
"""Compare already-built release runners; exclude one warmup per process."""
import argparse
import hashlib
import json
import platform
import statistics
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--baseline', type=Path, required=True)
parser.add_argument('--candidate', type=Path, required=True)
parser.add_argument('--baseline-commit', required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
runs = []
for kind in ['baseline', 'candidate', 'candidate', 'baseline']:
    runner = getattr(args, kind).resolve()
    output = subprocess.check_output([str(runner)], text=True)
    samples = [float(line.split(',')[1]) for line in output.splitlines()]
    if len(samples) != 6 or any(value <= 0 for value in samples):
        raise SystemExit('Expected six successful positive timing observations')
    runs.append({'kind': kind, 'sha256': hashlib.sha256(runner.read_bytes()).hexdigest(),
                 'seconds': samples})
medians = {kind: statistics.median(value for run in runs if run['kind'] == kind
                                  for value in run['seconds'][1:])
           for kind in ['baseline', 'candidate']}
result = {
    'platform': platform.platform(),
    'baseline_commit': args.baseline_commit,
    'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(),
    'probe_sha256': hashlib.sha256(Path(__file__).with_name('measure-storage.rs').read_bytes()).hexdigest(),
    'candidate_source_sha256': {name: hashlib.sha256((root / name).read_bytes()).hexdigest()
                               for name in ['src/slots.rs', 'src/vm.rs']},
    'runs': runs,
    'median_seconds': medians,
    'ratio': medians['candidate'] / medians['baseline'],
}
args.output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'median_seconds': medians, 'ratio': result['ratio']}))
