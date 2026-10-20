#!/usr/bin/env python3
"""Compare native managed allocation counts for 128 key equality/hash iterations."""
import argparse
import json
from pathlib import Path
import re
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('build', type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
build = args.build.resolve()
report = json.loads((build / 'build.json').read_text())
command = next(row['command'] for row in report['commands'] if row.get('phase') == 'host-compile-link').copy()
command[command.index(str(root / 'tools/native/console-host.c'))] = str(root / 'docs/experiments/console-read-key/allocation-host.c')
command[command.index('-o') + 1] = str(build / 'allocation-probe')
subprocess.run(command, check=True, capture_output=True)
rows = []
for mode in ['baseline', 'allocation']:
    result = subprocess.run([build / 'allocation-probe', mode], capture_output=True, text=True, timeout=60)
    assert result.returncode == 0 and result.stdout.strip() == 'PASS', result
    counters = {key: int(value) for key, value in re.findall(r'(allocations|collections|reclaimed)=(\d+)', result.stderr)}
    assert counters['allocations'] == counters['reclaimed'], counters
    rows.append(dict(mode=mode, **counters))
assert rows[0]['allocations'] == rows[1]['allocations'], rows
args.output.write_text(json.dumps(dict(iterations=128, measurements=rows,
    scope='Native managed allocations for typed key operations; excludes boxed calls, host allocator counts and throughput',
    command=command), indent=2) + '\n')
print(json.dumps(rows, indent=2))
