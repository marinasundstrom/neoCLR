#!/usr/bin/env python3
"""Same scalar IL, retained compiled/resolved code, increasing CPU work."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def expected(count):
    value = 42
    for _ in range(count):
        value = (value * 1664525 + 1013904223) & 0xffffffff
    return value if value < 0x80000000 else value - 0x100000000


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    jit = ROOT / 'target/release/neoclr-aot-poc'
    interpreter = ROOT / 'target/release/examples/jit_cpu_interpreter'
    source = HERE / 'workload.neoil'
    obj, aot = out / 'workload.o', out / 'workload'
    commands = [[str(jit), str(source), 'Calculate', str(obj)],
                ['xcrun', 'clang', '-O2', '-Wall', '-Wextra', '-Werror', str(HERE / 'host.c'), str(obj), '-o', str(aot)]]
    builds = []
    for cmd in commands:
        started = time.perf_counter()
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        builds.append(dict(command=cmd, seconds=time.perf_counter()-started, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stderr)
    evidence = dict(passed=False, platform=platform.platform(), sourceSha256=sha(source),
                    jitSha256=sha(jit), interpreterSha256=sha(interpreter),
                    aotSha256=sha(aot), objectSha256=sha(obj), builds=builds,
                    repetitionsPerProcess=4, rounds=3, runs=[], summaries=[])
    try:
        for count in [0, 10000, 100000, 1000000]:
            checksum = expected(count)
            modes = ['jit', 'aot', 'interpreter']
            for round_index in range(3):
                for mode in modes[round_index:] + modes[:round_index]:
                    cmd = ([str(jit), '--jit-scalar', str(source), 'Calculate', str(count), '--repetitions', '4'] if mode == 'jit' else
                           [str(aot), str(count), '4'] if mode == 'aot' else
                           [str(interpreter), str(source), str(count), '4'])
                    started = time.perf_counter()
                    result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
                    seconds = time.perf_counter()-started
                    if result.returncode or result.stderr:
                        raise RuntimeError(f'{mode}: {result.stdout} {result.stderr}')
                    data = json.loads(result.stdout)
                    calls = data['invocations']
                    if len(calls) != 4 or any(c['status'] != 0 or c['value'] != checksum for c in calls):
                        raise RuntimeError(f'{mode}: checksum mismatch for {count}')
                    evidence['runs'].append(dict(mode=mode, count=count, round=round_index,
                                                 command=cmd, processSeconds=seconds, report=data))
                print(f'PASS count={count} round={round_index+1}', flush=True)
            for mode in modes:
                runs = [r for r in evidence['runs'] if r['mode'] == mode and r['count'] == count]
                # Retain first-call data but exclude it from repeated invocation summaries.
                samples = [c['invocationSeconds'] for r in runs for c in r['report']['invocations'][1:]]
                evidence['summaries'].append(dict(mode=mode, count=count, expected=checksum,
                    invocationMedianSeconds=statistics.median(samples), invocationMinSeconds=min(samples),
                    invocationMaxSeconds=max(samples), processMedianSeconds=statistics.median(r['processSeconds'] for r in runs)))
        evidence['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(evidence, indent=2)+'\n')
    print(json.dumps(evidence['summaries'], indent=2))


if __name__ == '__main__':
    main()
