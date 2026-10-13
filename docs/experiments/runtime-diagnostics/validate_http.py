#!/usr/bin/env python3
"""Build the runtime-async HTTP showcase and qualify live provisional GC attachment."""
import argparse
import hashlib
import http.client
import json
import os
from pathlib import Path
import platform
import select
import socket
import statistics
import subprocess
import sys
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
CLIENT = ROOT / 'tools/profiler/neoclr-profiler.py'
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, default=HERE.parent / 'runtime-async/collector-walk-showcase.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('this host recipe is qualified for macOS ARM64 only')
    recipe = json.loads(args.recipe.read_text())
    assert recipe['passed']
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), commands=[], cases=[], performance=[],
                  recipeSha256=sha(args.recipe), inputsSha256={})

    def command(label):
        return next(row['command'].copy() for row in recipe['commands'] if row.get('label') == label)

    def run(label, cmd):
        for value in cmd:
            if Path(value).is_file() and not Path(value).is_relative_to(out):
                report['inputsSha256'][str(Path(value).resolve())] = sha(value)
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=180, cwd=ROOT)
        record = dict(label=label, command=cmd, exitCode=result.returncode, stderr=result.stderr)
        if label == 'native':
            artifact = out / 'selection.json'; artifact.write_text(result.stdout)
            record.update(stdoutFile=str(artifact), stdoutSha256=sha(artifact))
        else: record['stdout'] = result.stdout
        report['commands'].append(record)
        if result.returncode: raise RuntimeError(label + ': ' + result.stderr + result.stdout[-2000:])

    def serve(invocation, enabled, proof=False, measure=False):
        with tempfile.TemporaryDirectory(prefix='nc-http-', dir='/tmp') as directory:
            env = dict(os.environ)
            env.pop('NEOCLR_DIAGNOSTICS_DIRECTORY', None)
            if enabled: env['NEOCLR_DIAGNOSTICS_DIRECTORY'] = directory
            process = subprocess.Popen(invocation, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            profiler = None
            errors, timings, completed, captures = [], [], [], []
            stop = threading.Event()
            traffic = None
            try:
                if not select.select([process.stdout], [], [], 60)[0]: raise RuntimeError('HTTP startup timeout')
                port_line = process.stdout.readline()
                if not port_line.strip().isdigit():
                    raise RuntimeError('HTTP failed to start: ' + port_line + process.stderr.read())
                endpoint = f'{directory}/neoclr-{process.pid}.sock'
                if not enabled and Path(endpoint).exists(): raise RuntimeError('disabled endpoint exists')

                def request():
                    begin = time.perf_counter()
                    connection = http.client.HTTPConnection('127.0.0.1', int(port_line), timeout=10)
                    try:
                        connection.request('GET', '/greeting')
                        response = connection.getresponse()
                        body = response.read()
                        if response.status != 200 or body != 'Café 🌍'.encode():
                            raise RuntimeError('unexpected HTTP response')
                    finally: connection.close()
                    timings.append(time.perf_counter() - begin)
                    completed.append(1)

                # The app has served requests before any profiler attaches.
                for _ in range(4): request()

                def requests():
                    try:
                        for index in range(124):
                            if proof and index == 120 and not stop.wait(15):
                                raise RuntimeError('profiler proof timeout')
                            request()
                            if proof: time.sleep(.002)
                    except Exception as error: errors.append(str(error))
                start = time.perf_counter()
                traffic = threading.Thread(target=requests, daemon=True); traffic.start()
                if proof:
                    # A stalled diagnostics reader must not stop HTTP progress.
                    with socket.socket(socket.AF_UNIX) as slow:
                        slow.connect(endpoint)
                        before = len(completed)
                        time.sleep(.1)
                        if len(completed) <= before: raise RuntimeError('HTTP stalled with diagnostics reader')
                    for session in range(2):
                        before = len(completed)
                        row = subprocess.run([sys.executable, str(CLIENT), '--socket', endpoint,
                                              '--count', '4', '--interval', '.01'],
                                             capture_output=True, text=True, timeout=5)
                        if row.returncode: raise RuntimeError(row.stderr)
                        rows = [json.loads(line) for line in row.stdout.splitlines()]
                        if len(rows) != 4: raise RuntimeError('incomplete standalone capture')
                        captures.extend(rows)
                        if len(completed) <= before: raise RuntimeError('HTTP did not progress during capture')
                        report['cases'].append(dict(mode='jit' if '--jit-run' in invocation else 'aot',
                                                    session=session, requestsBefore=before,
                                                    requestsAfter=len(completed), snapshots=rows))
                    if not any(row['fresh'] for row in captures): raise RuntimeError('no live snapshot')
                    if captures[-1]['allocations'] <= captures[0]['allocations']:
                        raise RuntimeError('GC allocation counters did not advance')
                    if len({row['instance'] for row in captures}) != 1: raise RuntimeError('instance changed')
                    for row in captures:
                        if row['pid'] != process.pid or row['gcFailures'] or sum(row['gcPauseBuckets']) != row['gcAttempts']:
                            raise RuntimeError('invalid GC snapshot')
                        if row['allocatedPayloadBytes'] - row['reclaimedPayloadBytes'] != row['livePayloadBytesAfterCollection']:
                            raise RuntimeError('incoherent live payload')
                    stop.set()
                elif enabled and measure:
                    # Continuous external client, stopped when the fixed workload ends.
                    profiler = subprocess.Popen([sys.executable, str(CLIENT), '--socket', endpoint,
                                                 '--count', '100000', '--interval', '.01'],
                                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                    # Drain stdout concurrently: the client must not block on its output pipe.
                    def drain():
                        for line in profiler.stdout:
                            captures.append(json.loads(line))
                    drain_thread = threading.Thread(target=drain, daemon=True); drain_thread.start()
                traffic.join(30)
                elapsed = time.perf_counter() - start
                if traffic.is_alive() or errors: raise RuntimeError('HTTP requests failed: ' + repr(errors))
                stdout, stderr = process.communicate(timeout=10)
                if process.returncode or stderr or stdout.strip() != 'PASS 128 HTTP requests while profiling':
                    raise RuntimeError('HTTP shutdown failed: ' + stdout + stderr)
                if len(completed) != 128: raise RuntimeError('missing requests')
                if enabled and Path(endpoint).exists(): raise RuntimeError('endpoint leaked after shutdown')
                if profiler:
                    profiler.wait(timeout=5); drain_thread.join(5)
                    # The application exits after its fixed request count; client then reports disconnect.
                    profiler.stderr.read()
                    if not captures: raise RuntimeError('no measurement snapshots')
                return dict(requests=len(completed), secondsForLast124=elapsed,
                            medianRequestSeconds=statistics.median(timings[4:]),
                            snapshots=len(captures), firstSnapshot=captures[0] if captures else None,
                            lastSnapshot=captures[-1] if captures else None)
            finally:
                stop.set()
                if process.poll() is None: process.kill(); process.communicate()
                if profiler and profiler.poll() is None: profiler.kill(); profiler.communicate()
                if traffic: traffic.join(12)

    try:
        source = command('language-source')
        source[source.index('-o') + 1] = str(out / 'server.dll')
        source[-2] = str(HERE / 'HttpProfile.rvn')
        run('source', source)
        native = command('server-native-compile')
        native[2], native[4] = str(out / 'server.dll'), str(out / 'server.o')
        run('native', native)
        link = command('server-link')
        link = [str(out / 'server.o') if value.endswith('/server.o') else value for value in link]
        link[link.index('-o') + 1] = str(out / 'baseline')
        run('baseline-link', link)
        instrumented = link + ['-DNEOCLR_GC_PROFILE', '-DNEOCLR_DIAGNOSTICS', str(HERE / 'attach.c'), '-pthread']
        instrumented[instrumented.index('-o') + 1] = str(out / 'server')
        run('diagnostic-link', instrumented)
        jit_link = command('jit-host-build') + ['-DNEOCLR_GC_PROFILE', '-DNEOCLR_DIAGNOSTICS', str(HERE / 'attach.c'), '-pthread']
        jit_link[jit_link.index('-o') + 1] = str(out / 'host.dylib')
        run('jit-host', jit_link)
        # Hash transitive sources/headers used by the matching C hosts.
        for folder in [HERE, HERE.parent / 'aot-console', HERE.parent / 'runtime-async', HERE.parent / 'aot-fault-details']:
            for path in folder.iterdir():
                if path.suffix in ('.h', '.c'): report['inputsSha256'][str(path)] = sha(path)
        for path in [*sorted((ROOT / 'runtime/native').rglob('*.[ch]')), Path(__file__).resolve(), CLIENT]: report['inputsSha256'][str(path)] = sha(path)
        aot = [str(out / 'server')]
        jit = [native[0], '--jit-run', *native[1:]]
        jit[5] = str(out / 'host.dylib')
        report['aot'] = serve(aot, True, proof=True)
        report['jit'] = serve(jit, True, proof=True)
        for round_number in range(3):
            modes = [('uninstrumented', [str(out / 'baseline')], False),
                     ('instrumented-unattached', aot, False), ('attached', aot, True)]
            if round_number % 2: modes.reverse()
            for name, invocation, enabled in modes:
                result = serve(invocation, enabled, measure=True)
                report['performance'].append(dict(mode=name, round=round_number, **result))
        report['performanceSummary'] = {
            name: statistics.median(row['secondsForLast124'] for row in report['performance'] if row['mode'] == name)
            for name in ('uninstrumented', 'instrumented-unattached', 'attached')}
        report['performanceCaveat'] = 'Three local alternating AOT rounds, fixed serial HTTP requests; exploratory overhead, not a throughput or tail-latency claim.'
        for path, expected in report['inputsSha256'].items():
            if sha(path) != expected: raise RuntimeError('input changed: ' + path)
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(out / 'validation.json')


if __name__ == '__main__': main()
