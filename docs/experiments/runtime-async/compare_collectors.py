#!/usr/bin/env python3
"""Alternate native collector variants using identical runtime-async guest objects."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--benchmark', type=Path, required=True)
parser.add_argument('--baseline', required=True, help='Git revision supplying only baseline native-gc.c')
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
recipe = json.loads(args.benchmark.read_text())
if not recipe['passed']:
    parser.error('a passing native benchmark recipe is required')
out = args.output.resolve()
out.mkdir(exist_ok=False)
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
old = out / 'baseline-gc.c'
old.write_bytes(subprocess.check_output(
    ['git', 'show', args.baseline + ':docs/experiments/aot-console/native-gc.c'], cwd=ROOT))
revision = subprocess.check_output(['git', 'rev-parse', args.baseline], cwd=ROOT, text=True).strip()
report = dict(passed=False, baselineRevision=revision, commands=[], runs=[], inputsSha256={
    str(old): sha(old), str(Path(__file__).resolve()): sha(__file__),
    str(args.benchmark.resolve()): sha(args.benchmark)})
for directory in ('aot-console', 'runtime-async', 'jit-managed'):
    for path in (ROOT / 'docs/experiments' / directory).iterdir():
        if path.suffix in ('.c', '.h'):
            report['inputsSha256'][str(path)] = sha(path)


def run(command, sample_output=False):
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(command=command, exitCode=result.returncode,
        stderr=result.stderr if not sample_output or result.returncode else '', stdout=result.stdout))
    if result.returncode:
        raise RuntimeError(result.stderr or result.stdout)
    return result


try:
    for case in ('completed', 'pending'):
        command = next(row['command'].copy() for row in recipe['commands']
            if '-DNEOCLR_BENCH_AOT' in row['command']
            and Path(row['command'][row['command'].index('-o') + 1]).name == case + '-100-runtime')
        command += ['-DNEOCLR_BENCH_REPETITIONS=40']
        for variant in ('baseline', 'optimized'):
            link = command.copy()
            link[link.index('-o') + 1] = str(out / (case + '-' + variant))
            if variant == 'baseline':
                link[link.index(str(ROOT / 'docs/experiments/aot-console/native-gc.c'))] = str(old)
            for value in link:
                if Path(value).is_file():
                    report['inputsSha256'][value] = sha(value)
            run(link)
    for round_index in range(6):
        for case in ('completed', 'pending'):
            order = ('baseline', 'optimized') if round_index % 2 == 0 else ('optimized', 'baseline')
            for variant in order:
                result = run([str(out / (case + '-' + variant))], sample_output=True)
                samples = [json.loads(line.removeprefix('NATIVE_SAMPLE '))
                           for line in result.stderr.splitlines()]
                if len(samples) != 40 or result.stdout:
                    raise RuntimeError('unexpected guest output or invocation count')
                if any(s['allocatedBlocks'] != s['reclaimedBlocks'] or s['reservedStackBytes'] for s in samples):
                    raise RuntimeError('guest heap or native stack was not reclaimed')
                report['runs'].append(dict(round=round_index, case=case, variant=variant, samples=samples))
    report['summaries'] = []
    for case in ('completed', 'pending'):
        medians = {variant: statistics.median(sample['seconds']
            for invocation in report['runs'] if invocation['case'] == case and invocation['variant'] == variant
            for sample in invocation['samples'][1:]) for variant in ('baseline', 'optimized')}
        report['summaries'].append(dict(case=case, **medians,
            optimizedOverBaseline=medians['optimized'] / medians['baseline']))
    report['measurement'] = ('Six alternating rounds, 40 invocations per process, first omitted per process; '
        'identical AOT object, current host with only collector implementation replaced. Instrumentation disabled.')
    if any(sha(path) != digest for path, digest in report['inputsSha256'].items()):
        raise RuntimeError('input changed during measurement')
    report['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(report.get('summaries'))
