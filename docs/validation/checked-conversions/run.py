#!/usr/bin/env python3
"""Qualify SDK checked conversions in the interpreter and retain AOT rejection evidence."""
from concurrent.futures import ThreadPoolExecutor
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--images', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
images, output = args.images.resolve(), args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
runtime = ROOT / 'target/debug/neoclr'
codegen = ROOT / 'target/debug/neoclr-codegen'
cases = json.loads((images / 'cases.json').read_text())
report = {'passed': False, 'cases': [], 'aotRejections': [], 'commands': []}
tested = set()

def run(label, command):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=60)
    log = result.stdout + result.stderr
    (output / (label + '.log')).write_text(log)
    report['commands'].append({'command': list(map(str, command)), 'exitCode': result.returncode,
                               'logSha256': hashlib.sha256(log.encode()).hexdigest()})
    return result, log

def interpret(case):
    name = case['name']
    image = images / (name + '.dll')
    result, log = run(name, [runtime, 'run', image, '--execution', 'interpreter'])
    if case['overflow']:
        assert result.returncode != 0 and 'ArithmeticOverflow:' in log, (name, log)
    else:
        assert result.returncode == 42, (name, log)
    return {**case, 'passed': True, 'imageSha256': hashlib.sha256(image.read_bytes()).hexdigest()}

# Independent guest processes keep this boundary matrix inexpensive to repeat.
with ThreadPoolExecutor(max_workers=8) as workers:
    for result in workers.map(interpret, cases):
        report['cases'].append(result)
        if len(report['cases']) % 100 == 0:
            print(len(report['cases']), 'PASS', flush=True)

for case in cases:
    name = case['name']
    opcode = name.split('-')[0]
    if opcode not in tested:
        image = images / (name + '.dll')
        obj = output / (opcode + '.o')
        result, log = run(opcode + '-aot-rejection', [codegen, '--closed-world', image, '@entry', obj])
        assert result.returncode != 0 and 'unsupported value instruction: Checked' in log and not obj.exists(), (name, log)
        report['aotRejections'].append(opcode)
        tested.add(opcode)

report['passed'] = True
report['files'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in [runtime, codegen, Path(__file__).resolve(), images / 'cases.json',
                             ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/bin/Debug/net10.0/NeoCLR.Metadata.Experimental.dll']}
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(len(cases), 'interpreter cases PASS;', len(tested), 'AOT opcodes reject explicitly')
