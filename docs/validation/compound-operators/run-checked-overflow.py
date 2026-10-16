#!/usr/bin/env python3
"""Compile Raven checked arithmetic/conversion consumers and validate native overflow faults."""
import argparse
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
mode = parser.add_mutually_exclusive_group()
mode.add_argument('--unary', action='store_true', help='Qualify signed int/long unary negation instead of the binary matrix')
mode.add_argument('--conversions', action='store_true', help='Qualify narrowing, unsigned-source and floating conversions')
mode.add_argument('--mutations', action='store_true', help='Qualify compound arithmetic and increment narrowing overflow')
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
if args.unary:
    cases = [('negate-int', 'int', '-', '-2147483648', None),
             ('negate-long', 'long', '-', '-9223372036854775808L', None)]
if args.conversions:
    cases = [('narrow-byte', 'int', 'byte', '256', None),
             ('unsigned-to-signed', 'uint', 'int', '(uint)4294967295L', None),
             ('floating-to-int', 'double', 'int', '2147483648.0', None)]
if args.mutations:
    cases = [('compound-int', 'int', '+= 1', '2147483647', None),
             ('increment-byte', 'byte', '++', '(byte)255', None)]
reports = []
for name, kind, operator, left, right in cases:
    source = output / (name + '.rvn')
    parameters = f'a: {kind}, b: {kind}' if right is not None else f'a: {kind}'
    expression = f'a {operator} b' if right is not None else '-a'
    result_type = operator if args.conversions else kind
    if args.conversions:
        expression = f'({result_type})a'
    arguments = f'{left}, {right}' if right is not None else left
    source_text = f'''class Program {{
    static func Overflow({parameters}) -> {result_type} => checked({expression})
    static func Main() -> int {{
        let result = Overflow({arguments})
        return 42
    }}
}}
'''
    if args.mutations:
        source_text = f'''class Program {{
    static func Overflow(a: {kind}) {{
        var value = a
        checked(value{operator})
    }}
    static func Main() -> int {{
        Overflow({left})
        return 42
    }}
}}
'''
    source.write_text(source_text)
    subprocess.run(['python3', str(runner), '--compiler', str(args.compiler),
                    '--bundle', str(args.bundle), '--output', str(output / name),
                    '--source', str(source), '--overflow'], check=True)
    reports.append({'name': name, 'source': source.read_text(),
                    'result': json.loads((output / name / 'results.json').read_text())})
(output / 'results.json').write_text(json.dumps({'passed': True, 'cases': reports}, indent=2) + '\n')
