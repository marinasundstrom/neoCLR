#!/usr/bin/env python3
"""Compare direct HttpServer, Neo.Web context and generic handlers on one matched bundle."""
import argparse
import hashlib
import platform
import json
import os
from pathlib import Path
import queue
import signal
import statistics
import subprocess
import sys
import threading
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def measure(command, count, warmup, path, expected, caching):
    started = time.perf_counter()
    process = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, start_new_session=True)
    messages, output, errors = queue.Queue(), [], []

    def consume():
        for line in process.stdout:
            output.append(line)
            messages.put(line)
        messages.put(None)

    threading.Thread(target=consume, daemon=True).start()
    reader = threading.Thread(target=lambda: errors.append(process.stderr.read()), daemon=True)
    reader.start()
    try:
        while True:
            line = messages.get(timeout=180)
            if line is None:
                raise RuntimeError('Server exited before listening: ' + ''.join(output + errors))
            if line.startswith('LISTENING '):
                url = line.strip().split(' ', 1)[1] + path
                break
        ready = time.perf_counter() - started

        def request():
            before = time.perf_counter()
            with urllib.request.urlopen(url, timeout=60) as response:
                if response.status != 200 or response.read() != expected:
                    raise RuntimeError('Unexpected benchmark response')
            return time.perf_counter() - before

        first = request()
        for _ in range(warmup):
            request()
        wall = time.perf_counter()
        latencies = [request() for _ in range(count)]
        elapsed = time.perf_counter() - wall
        ordered = sorted(latencies)
        result = dict(readySeconds=ready, firstRequestSeconds=first, warmup=warmup,
                    requests=count, elapsedSeconds=elapsed, requestsPerSecond=count / elapsed,
                    medianSeconds=statistics.median(latencies),
                    p95Seconds=ordered[min(count - 1, int(count * .95))],
                    latenciesSeconds=latencies)
        if caching is not None:
            with urllib.request.urlopen(url.rsplit('/', 1)[0] + '/cache-stats', timeout=60) as response:
                counters = dict((key, int(value)) for key, value in
                                (field.split('=') for field in response.read().decode().split(';')))
            requests = count + warmup + 1
            expected_counts = dict(hits=requests - 1, misses=1, executions=1) if caching else dict(hits=0, misses=0, executions=requests)
            if counters != expected_counts:
                raise RuntimeError('Cache counters differ: ' + repr(counters))
            result['cacheCounters'] = counters
        return result
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        reader.join(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', required=True, type=Path)
    parser.add_argument('--codegen', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--requests', type=int, default=30)
    parser.add_argument('--warmup', type=int, default=5)
    args = parser.parse_args()
    if args.requests < 1 or args.warmup < 0:
        parser.error('requests must be positive and warmup nonnegative')
    bundle, out, codegen = args.bundle.resolve(), args.output.resolve(), args.codegen.resolve()
    out.mkdir(parents=True, exist_ok=False)
    build = out / 'build'
    env = dict(os.environ, NeoClrBundleRoot=str(bundle))
    command = [sys.executable, str(ROOT / 'scripts/build-native-project.py'), '--profile', 'http',
               '--bundle', str(bundle), '--project', str(ROOT / 'benchmarks/neo-web/NeoWebBenchmark.rvnproj'),
               '--aot', str(codegen), '--output', str(build)]
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    (out / 'build.log').write_text(result.stdout + result.stderr)
    result.check_returncode()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    build_report = json.loads((build / 'build.json').read_text())
    report = dict(description='macOS loopback HTTP/1.1; sequential fresh connections; constant 2-byte response or identical computed response with cache disabled/enabled',
                  caveat='Cold readiness and first request are separate; throughput includes client and transport costs. One process per case, no statistical regression claim.',
                  bundle=str(bundle), platform=platform.platform(), machine=platform.machine(),
                  bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest(),
                  codegenSha256=hashlib.sha256(codegen.read_bytes()).hexdigest(), cases=[])
    for mode in ['interpreter', 'native', 'jit']:
        if mode == 'native':
            base = [str(build / 'app')]
        else:
            base = [str(bundle / 'bin/neoclr'), 'run', str(build / 'app.dll'), '--execution', mode,
                    '--system', str(bundle / 'lib/System.runtime.neox'),
                    '--object-root', str(bundle / 'lib/System.Runtime.dll')]
            if mode == 'interpreter':
                base += ['--instructions', '1000000000']
            else:
                base += ['--jit-compiler', str(codegen), '--jit-host', str(bundle / 'bin/neoclr-jit-host.dylib'),
                         '--reflection-roots', str(build / 'app.reflection.json')]
            for module in manifest['modules']:
                base += ['--module', str(bundle / module)]
            for dependency in build_report.get('projectModules', []):
                base += ['--module', dependency]
            base += ['--']
        for handler in ['direct', 'context', 'generic', 'uncached', 'cached']:
            case = dict(mode=mode, handler=handler, command=base + [handler])
            computed = 0
            for index in range(5000):
                computed = (computed * 17 + index) % 1009
            caching = handler in ('cached', 'uncached')
            expected = ('Computed ' + str(computed)).encode() if caching else b'42'
            case['workload'] = 'computed' if caching else 'constant'
            case.update(measure(base + [handler], args.requests, args.warmup,
                                '/cached' if caching else '/', expected, handler == 'cached' if caching else None))
            report['cases'].append(case)
            (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
            print(f'{mode}/{handler}: {case["requestsPerSecond"]:.2f} requests/s', flush=True)


if __name__ == '__main__':
    main()
