#!/usr/bin/env python3
"""Run the same sequential release measurements on macOS ARM64 and Windows x64."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from host import target, cpu_description, load_average


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def steps(bundle, output, rounds):
    execution = output / 'execution'
    manifest = execution / 'manifest.json'
    def command(script, *args):
        return [sys.executable, str(ROOT / script), *map(str, args)]
    return [
        ('prepare', command('benchmarks/execution/prepare.py', '--bundle', bundle, '--output', execution), manifest, False),
        ('execution', command('benchmarks/framework/runner.py', '--manifest', manifest, '--output', execution / 'results.json', '--rounds', rounds, '--warmups', 2, '--operations', 16, '--timeout', 300), execution / 'results.json', True),
        ('cpu-native', command('benchmarks/framework/runner.py', '--manifest', manifest, '--output', execution / 'cpu-native-longer.json', '--filter', 'cpu.sum', '--mode', 'jit', '--mode', 'aot', '--rounds', rounds, '--warmups', 20, '--operations', 10000, '--timeout', 300), execution / 'cpu-native-longer.json', True),
        *[(f'http-{workload}', command('benchmarks/http-server/benchmark.py', '--bundle', bundle, '--output', output / f'http-{workload}', '--workload', workload, '--rounds', rounds), output / f'http-{workload}/results.json', True) for workload in ('plain', 'json')],
    ]


def run(bundle, output, rounds):
    if rounds < 1:
        raise ValueError('Rounds must be positive')
    host = target()
    manifest = bundle / 'manifest.json'
    digest = sha(manifest)
    output.mkdir(parents=True, exist_ok=False)
    report = dict(schemaVersion=1, passed=False, platform=platform.platform(),
                  architecture=platform.machine(), cpu=cpu_description(), host=host,
                  rounds=rounds, bundleManifestSha256=digest, steps=[],
                  revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  dirty=bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()),
                  runnerSha256=sha(Path(__file__)), loadBefore=load_average())
    try:
        for name, command, result, measured in steps(bundle, output, rounds):
            row = dict(name=name, command=command, result=str(result))
            report['steps'].append(row)
            started = time.monotonic()
            with (output / f'{name}.log').open('w') as log:
                completed = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
            row.update(exitCode=completed.returncode, elapsedSeconds=time.monotonic() - started)
            if completed.returncode:
                raise RuntimeError(f'{name} failed; see {name}.log')
            evidence = json.loads(result.read_text())
            if measured and evidence.get('passed') is not True:
                raise RuntimeError(f'{name} did not produce passing measurement evidence')
            row['resultSha256'] = sha(result)
        if sha(manifest) != digest:
            raise RuntimeError('Bundle manifest changed during measurement')
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        report['loadAfter'] = load_average()
        (output / 'release-benchmarks.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=5)
    args = parser.parse_args()
    run(args.bundle.resolve(), args.output.resolve(), args.rounds)
