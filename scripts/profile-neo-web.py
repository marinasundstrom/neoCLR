#!/usr/bin/env python3
"""Compare stress/paced native GC on one prebuilt Neo.Web benchmark object (macOS)."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import threading
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
CASES = ['direct', 'direct-context', 'direct-scope', 'context', 'context-middleware',
         'generic', 'service', 'query', 'cancellation']


def profile(binary, output, seconds):
    process = subprocess.Popen([str(binary), 'direct'], stdout=subprocess.PIPE,
                               stderr=subprocess.DEVNULL, text=True, start_new_session=True)
    stop = threading.Event()
    failures = []
    completed = []
    worker = None
    try:
        # The native executable has already passed readiness checks in measure().
        line = process.stdout.readline()
        if not line.startswith('LISTENING '):
            raise RuntimeError('Profile server did not start: ' + line)
        url = line.strip().split(' ', 1)[1]

        def consume():
            try:
                while not stop.is_set():
                    with urllib.request.urlopen(url, timeout=30) as response:
                        if response.status != 200 or response.read() != b'42':
                            raise RuntimeError('Unexpected profile response')
                    completed.append(1)
            except Exception as error:
                failures.append(str(error))

        worker = threading.Thread(target=consume, daemon=True)
        worker.start()
        subprocess.run(['/usr/bin/sample', str(process.pid), str(seconds), '1',
                        '-file', str(output)], capture_output=True, text=True, check=True, timeout=seconds + 30)
        stop.set()
        worker.join(timeout=35)
        if worker.is_alive() or failures:
            raise RuntimeError('Profile consumer failed: ' + repr(failures))
        return dict(validatedResponses=len(completed), durationSeconds=seconds,
                    note='Sampled diagnostic run; excluded from throughput comparisons.')
    finally:
        stop.set()
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        if worker:
            worker.join(timeout=35)
        process.stdout.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', required=True, type=Path, help='Successful build-native-project.py benchmark output')
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--policy', action='append', choices=['stress', 'paced'])
    parser.add_argument('--requests', type=int, default=30)
    parser.add_argument('--warmup', type=int, default=5)
    parser.add_argument('--repetitions', type=int, default=3)
    parser.add_argument('--sample-seconds', type=int, default=5)
    args = parser.parse_args()
    if platform.system() != 'Darwin' or min(args.requests, args.repetitions) < 1 or min(args.warmup, args.sample_seconds) < 0:
        parser.error('Requires macOS, positive requests/repetitions and nonnegative warmup/sample duration')
    build, out = args.build.resolve(), args.output.resolve()
    report = json.loads((build / 'build.json').read_text())
    if not report['passed']:
        parser.error('A successful benchmark build is required')
    out.mkdir(parents=True, exist_ok=False)
    spec = importlib.util.spec_from_file_location('benchmark_neo_web', ROOT / 'scripts/benchmark-neo-web.py')
    benchmark = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(benchmark)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    result = dict(platform=platform.platform(), machine=platform.machine(),
                  sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  benchmarkSourceSha256=sha(ROOT / 'benchmarks/neo-web/Main.rvn'),
                  buildReportSha256=sha(build / 'build.json'), objectSha256=sha(build / 'app.o'),
                  scope='AOT, same object and heap budget; fresh sequential loopback connections; no runtime default promotion.',
                  commands=[], binaries={}, cases=[], profiles={})
    recipe = next(row['command'] for row in report['commands'] if row.get('phase') == 'host-compile-link')
    binaries = {}
    # Compile both hosts before timed execution; policy is the only flag difference.
    selected_policies = args.policy or ['stress', 'paced']
    for policy in selected_policies:
        binary = out / ('app-' + policy)
        command = recipe.copy()
        command[command.index('-o') + 1] = str(binary)
        if policy == 'paced':
            command.insert(1, '-DNEOCLR_GC_PACED')
        compiled = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        (out / (policy + '-build.log')).write_text(compiled.stdout + compiled.stderr)
        compiled.check_returncode()
        result['commands'].append(command)
        binaries[policy] = binary
        result['binaries'][policy] = sha(binary)
    for repetition in range(args.repetitions):
        policies = selected_policies if repetition % 2 == 0 else list(reversed(selected_policies))
        cases = CASES[repetition % len(CASES):] + CASES[:repetition % len(CASES)]
        for policy in policies:
            for handler in cases:
                row = dict(policy=policy, handler=handler, repetition=repetition + 1)
                row.update(benchmark.measure([str(binaries[policy]), handler], args.requests,
                                             args.warmup, '/?value=42' if handler == 'query' else '/', b'42', None))
                result['cases'].append(row)
                (out / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
                print(f'{policy}/{handler}/{repetition + 1}: {row["requestsPerSecond"]:.2f} req/s', flush=True)
    for policy, binary in binaries.items():
        if args.sample_seconds:
            result['profiles'][policy] = profile(binary, out / (policy + '-sample.txt'), args.sample_seconds)
    result['passed'] = True
    (out / 'results.json').write_text(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    main()
