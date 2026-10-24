#!/usr/bin/env python3
"""Paired loading/execution trials; requires retained mode-comparison artifacts."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import resource
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--baseline-runtime', type=Path, default=Path('target/gc-mode-comparison-final/baseline-neoclr'))
parser.add_argument('--baseline-codegen', type=Path, default=Path('target/gc-mode-comparison-final/baseline-codegen'))
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[3]
corpus = json.loads(gzip.decompress((root / 'benchmarks/execution/gc-mode-comparison/ordinary.json.gz').read_bytes()))
report = dict(passed=False, runs=[], objects=[], inputs={})


def run(command, **labels):
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.perf_counter()
    result = subprocess.run(command, capture_output=True, text=True, timeout=120)
    wall = time.perf_counter() - started
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    row = dict(labels, command=command, exitCode=result.returncode, stdout=result.stdout,
               stderr=result.stderr, wallSeconds=wall,
               cpuSeconds=after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime)
    report['runs'].append(row)
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    result.check_returncode()
    return row


try:
    for case in corpus['cases']:
        for repetition in range(3):
            for variant in (['baseline', 'candidate'] if repetition % 2 == 0 else ['candidate', 'baseline']):
                for mode in ['interpreter', 'jit']:
                    original = next(x for x in case['runs'] if x['mode'] == mode
                                    and not x['diagnostic'] and (mode == 'interpreter' or x['policy'] == 'paced'))
                    command = original['command'].copy()
                    binary = 'neoclr' if mode == 'interpreter' else 'neoclr-codegen'
                    command[0] = str(root / ('target/release/' + binary) if variant == 'candidate'
                                     else (args.baseline_codegen if mode == 'jit' else args.baseline_runtime).resolve())
                    row = run(command, sample=case['sample'], mode=mode, variant=variant, repetition=repetition)
                    assert row['stdout'] == case['expected']
                    if mode == 'jit':
                        row['jit'] = json.loads(next(line[11:] for line in row['stderr'].splitlines() if line.startswith('JIT_RESULT ')))
                        assert row['jit']['guestExitCode'] == 0
        original = next(x['command'] for x in corpus['commands'] if x['label'] == 'aot-compile'
                        and f"/{case['sample']}/bin/" in x['command'][2])
        hashes = {}
        for variant in ['baseline', 'candidate']:
            command = original.copy()
            command[0] = str(root / 'target/release/neoclr-codegen' if variant == 'candidate'
                             else args.baseline_codegen.resolve())
            obj = out / f"{case['sample']}-{variant}.o"
            command[4] = str(obj)
            row = run(command, sample=case['sample'], mode='aot-compile', variant=variant)
            row['stdoutSha256'] = hashlib.sha256(row['stdout'].encode()).hexdigest()
            row['stdout'] = '(inspection output omitted; hash retained)'
            hashes[variant] = hashlib.sha256(obj.read_bytes()).hexdigest()
        assert hashes['baseline'] == hashes['candidate']
        report['objects'].append(dict(sample=case['sample'], hashes=hashes))
    for path in [Path(__file__), root/'src/inheritance.rs', root/'src/runtime_lookup.rs',
                 *[Path(row['command'][0]) for row in report['runs']]]:
        report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    report['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(dict(passed=report['passed'], runs=len(report['runs']))))
