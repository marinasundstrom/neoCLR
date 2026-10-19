#!/usr/bin/env python3
"""Relink the search allocation consumer with the existing GC counter host."""
import argparse
import json
from pathlib import Path
import re
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('build', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
build = args.build.resolve()
report = json.loads((build / 'build.json').read_text())
command = next(row['command'] for row in report['commands'] if row.get('phase') == 'host-compile-link').copy()
host = root / 'docs/experiments/string-building/reuse-20261019/allocation-host.c'
# Replace only the generated console entry, preserving the matched kernel/link inputs.
entries = [i for i, value in enumerate(command) if value == str(root / 'tools/native/console-host.c')]
if len(entries) != 1:
    raise RuntimeError('Expected one generated console host in link command')
command[entries[0]] = str(host)
command[command.index('-o') + 1] = str(build / 'allocation-probe')
subprocess.run(command, check=True, capture_output=True, text=True)
rows = []
for mode in ['baseline', 'check']:
    result = subprocess.run([str(build / 'allocation-probe'), mode], text=True, capture_output=True, timeout=60)
    result.check_returncode()
    assert result.stdout.strip() == 'PASS search allocation', result.stdout
    counters = {key: int(value) for key, value in re.findall(r'(allocations|collections|reclaimed)=(\d+)', result.stderr)}
    assert counters['allocations'] == counters['reclaimed'], result.stderr
    rows.append(dict(mode=mode, **counters))
assert rows[0]['allocations'] == rows[1]['allocations'], rows
args.output.write_text(json.dumps(dict(command=command, iterations=128, measurements=rows,
    scope='Whole-process native managed allocations; not CPU, host allocations or throughput'), indent=2) + '\n')
print(json.dumps(rows, indent=2))
