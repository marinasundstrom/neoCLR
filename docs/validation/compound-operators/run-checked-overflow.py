#!/usr/bin/env python3
"""Compile Raven checked-overflow consumers and validate all six native instructions."""
import argparse
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
runner = Path(__file__).with_name('run.py')
cases = [
    ('signed-add', 'int', '+', '2147483647', '1'),
    ('signed-subtract', 'int', '-', '-2147483648', '1'),
    ('signed-multiply', 'long', '*', '9223372036854775807L', '2L'),
    ('unsigned-add', 'uint', '+', '(uint)4294967295L', '(uint)1'),
    ('unsigned-subtract', 'ulong', '-', '(ulong)0', '(ulong)1'),
    ('unsigned-multiply', 'uint', '*', '(uint)4294967295L', '(uint)2'),
]
reports = []
for name, kind, operator, left, right in cases:
    source = output / (name + '.rvn')
    source.write_text(f'''class Program {{
    static func Overflow(a: {kind}, b: {kind}) -> {kind} => checked(a {operator} b)
    static func Main() -> int {{
        let result = Overflow({left}, {right})
        return 42
    }}
}}
''')
    subprocess.run(['python3', str(runner), '--compiler', str(args.compiler),
                    '--bundle', str(args.bundle), '--output', str(output / name),
                    '--source', str(source), '--overflow'], check=True)
    reports.append({'name': name, 'source': source.read_text(),
                    'result': json.loads((output / name / 'results.json').read_text())})
(output / 'results.json').write_text(json.dumps({'passed': True, 'cases': reports}, indent=2) + '\n')
