#!/usr/bin/env python3
"""Compare native managed allocation counts for 128 generic and direct equality/hash iterations."""
import argparse
import json
import hashlib
from pathlib import Path
import re
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('build', type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--expected-extra-per-iteration', type=int, help='Assert the qualified allocation cost against the direct control')
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
build = args.build.resolve()
report = json.loads((build / 'build.json').read_text())
command = next(row['command'] for row in report['commands'] if row.get('phase') == 'host-compile-link').copy()
command[command.index(str(root / 'tools/native/console-host.c'))] = str(root / 'docs/experiments/console-read-key/allocation-host.c')
command[command.index('-o') + 1] = str(build / 'allocation-probe')
subprocess.run(command, check=True, capture_output=True)
rows = []
for mode in ['baseline', 'direct', 'generic']:
    result = subprocess.run([build / 'allocation-probe', mode], capture_output=True, text=True, timeout=60)
    assert result.returncode == 0 and result.stdout.strip() == 'PASS', result
    counters = {key: int(value) for key, value in re.findall(r'(allocations|collections|reclaimed)=(\d+)', result.stderr)}
    assert counters['allocations'] == counters['reclaimed'], counters
    rows.append(dict(mode=mode, **counters))
assert rows[0]['allocations'] == rows[1]['allocations'], rows
assert rows[2]['allocations'] > rows[1]['allocations'], rows
extra = rows[2]['allocations'] - rows[1]['allocations']
if args.expected_extra_per_iteration is not None:
    assert extra == 128 * args.expected_extra_per_iteration, (extra, rows)
compiler = Path(next(row['command'][1] for row in report['commands'] if row.get('phase') == 'source-compilation'))
bundle = compiler.parents[3]
paths = [build / 'app.dll', build / 'app.o', build / 'allocation-probe', compiler,
         compiler.with_name('Raven.CodeAnalysis.NeoClr.dll'), bundle / 'lib/System.Runtime.dll',
         Path(__file__).with_name('Allocation.rvn')]
args.output.write_text(json.dumps(dict(iterations=128, measurements=rows,
    extraAllocationsPerIteration=extra / 128,
    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
    scope='Native managed allocations for 128 generic or nongeneric Int32 record typed Equals and two hashes; includes any generic boxes and type descriptors; argument and mode-string materialization is outside the loop; excludes host allocator counts and throughput',
    command=command), indent=2) + '\n')
print(json.dumps(rows, indent=2))
