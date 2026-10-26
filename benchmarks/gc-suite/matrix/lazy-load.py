"""Paced complex HTTP load across one or more isolated server processes.

Reuses the checked open-loop arrival scheduler: misses remain offered arrivals,
there is no hidden client queue, and latency includes dispatch delay. Processes
share the host CPU/memory; handler concurrency is bounded, not a hard CPU quota.
"""
import argparse
import asyncio
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import queue
import socket
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
loader = ROOT/'benchmarks/http-server/load/run.py'
spec = importlib.util.spec_from_file_location('http_load', loader)
load = importlib.util.module_from_spec(spec)
spec.loader.exec_module(load)


async def exchange(port, path, size, timeout, timings=None, header_gap=0.02, dependency_port=None):
    writer = None
    try:
        async def request():
            nonlocal writer
            reader, writer = await asyncio.open_connection('127.0.0.1', port)
            request_line = f'GET {path} HTTP/1.1\r\n'.encode()
            headers = b'Host: localhost\r\nConnection: close\r\n'
            if dependency_port:
                headers += b'X-Dependency: '+str(dependency_port).encode()+b'\r\n'
            if header_gap > 0:
                writer.write(request_line)
                await writer.drain()
                await asyncio.sleep(header_gap)
                writer.write(headers+b'\r\n')
            else:
                writer.write(request_line+headers+b'\r\n')
            await writer.drain()
            header = await reader.readuntil(b'\r\n\r\n')
            lines = header[:-4].split(b'\r\n')
            if lines[0] != b'HTTP/1.1 200 OK':
                return 'unexpected_status'
            fields = {}
            for line in lines[1:]:
                key, value = line.split(b':', 1)
                key = key.lower()
                if key in fields:
                    return 'duplicate_header'
                fields[key] = value.strip()
            length = int(fields[b'content-length'])
            if not 0 < length <= 256 or fields.get(b'connection') != b'close':
                return 'invalid_framing'
            if fields.get(b'content-type', b'').split(b';')[0] != b'application/json':
                return 'invalid_content_type'
            body = await reader.readexactly(length)
            if await reader.read(1):
                return 'trailing_bytes'
            return 'ok' if json.loads(body) == {'id': 42, 'name': 'Café team', 'active': True} else 'invalid_body'
        return await asyncio.wait_for(request(), timeout)
    except asyncio.TimeoutError:
        return 'timeout'
    except (OSError, ValueError, KeyError, asyncio.IncompleteReadError, asyncio.LimitOverrunError) as error:
        return type(error).__name__
    finally:
        if writer is not None:
            writer.close()
            try:
                await writer.wait_closed()
            except OSError:
                pass


class Dependency:
    """An asynchronous delayed dependency; no sleeping worker per connection."""
    def __init__(self, delay):
        self.delay = delay
        self.ready = threading.Event()
        self.requests = 0
        self.active = 0
        self.peak = 0
        self.thread = threading.Thread(target=self.run)
        self.thread.start()
        assert self.ready.wait(10)

    def run(self):
        async def serve():
            self.loop = asyncio.get_running_loop()
            self.done = asyncio.Event()
            self.server = await asyncio.start_server(self.respond, '127.0.0.1', 0)
            self.port = self.server.sockets[0].getsockname()[1]
            self.ready.set()
            async with self.server:
                await self.done.wait()
        asyncio.run(serve())

    async def respond(self, reader, writer):
        self.active += 1
        self.peak = max(self.peak, self.active)
        try:
            request = await asyncio.wait_for(reader.readexactly(1), 10)
            assert request == bytes([42]), request
            self.requests += 1
            await asyncio.sleep(self.delay)
            writer.write(bytes([43]))
            await writer.drain()
        except (OSError, asyncio.TimeoutError, asyncio.IncompleteReadError):
            pass
        finally:
            writer.close()
            try:
                await writer.wait_closed()
            except OSError:
                pass
            self.active -= 1

    def close(self):
        self.loop.call_soon_threadsafe(self.done.set)
        self.thread.join(timeout=10)
        assert not self.thread.is_alive()


class Server:
    def __init__(self, recipe, extra_env):
        env = {k: v for k, v in os.environ.items() if not k.startswith('NEOCLR_')}
        env.update(recipe['env'])
        env.update(extra_env)
        self.started = time.perf_counter()
        self.process = subprocess.Popen(recipe['command'], env=env, cwd=ROOT,
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.stdout, self.stderr, self.lines, self.readers = [], [], queue.Queue(), []
        def drain(stream, output, notify):
            for line in stream:
                output.append(line)
                if notify:
                    self.lines.put((line, time.perf_counter()))
            if notify:
                self.lines.put((None, time.perf_counter()))
        for stream, output, notify in [(self.process.stdout, self.stdout, True),
                                       (self.process.stderr, self.stderr, False)]:
            thread = threading.Thread(target=drain, args=(stream, output, notify))
            thread.start()
            self.readers.append(thread)

    def ready(self):
        while True:
            line, at = self.lines.get(timeout=120)
            assert line is not None, ''.join(self.stderr)
            if line.strip() == 'READY':
                break
        self.port = int(self.lines.get(timeout=10)[0])
        self.control = int(self.lines.get(timeout=10)[0])
        return at-self.started

    def stop(self):
        with socket.create_connection(('127.0.0.1', self.control), timeout=10):
            pass
        self.process.wait(timeout=45)
        for reader in self.readers:
            reader.join(timeout=5)
        assert self.process.returncode == 0, ''.join(self.stderr)
        done = next(line for line in self.stdout if line.startswith('DONE '))
        return list(map(int, done.split()[1:]))

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait(timeout=10)
        for reader in self.readers:
            reader.join(timeout=5)
        self.process.stdout.close()
        self.process.stderr.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--rates', type=float, nargs='+', default=[50, 150, 400])
    p.add_argument('--apps', type=int, nargs='+', default=[1, 2])
    p.add_argument('--modes', nargs='+', default=['arena-eager-sync-jit', 'arena-control-sync-jit',
                                               'arena-eager-live-jit', 'arena-control-live-jit'])
    p.add_argument('--duration', type=float, default=15)
    p.add_argument('--rounds', type=int, default=2)
    p.add_argument('--concurrency', type=int, default=16)
    p.add_argument('--timeout', type=float, default=3)
    p.add_argument('--header-gap-ms', type=float, default=20)
    p.add_argument('--p99-target-ms', type=float, default=100)
    p.add_argument('--dependency-ms', type=float, default=0)
    p.add_argument('--warmup-requests', type=int, default=16, help='Use zero to expose first-call compilation to paced traffic')
    p.add_argument('--reuse-processes', action='store_true', help='Run successive rate windows in the same long-lived app; reverse rates and modes on alternate rounds')
    p.add_argument('--shares', type=float, nargs='+', help='Optional per-app offered-load shares; length must match every app count')
    a = p.parse_args()
    assert min(a.rates) > 0 and min(a.apps) > 0 and a.duration > 0 and a.rounds > 0
    assert a.concurrency > 0 and a.timeout > 0 and a.header_gap_ms >= 0
    assert a.dependency_ms >= 0
    assert a.warmup_requests >= 0
    if a.shares:
        assert min(a.shares) > 0 and all(apps == len(a.shares) for apps in a.apps)
    source = a.build.resolve()/'build.json'
    build = json.loads(source.read_text())
    assert build['passed'] and all(mode in build['recipes'] for mode in a.modes)
    for path, digest in build['inputs'].items():
        assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(completed=False, scope=__doc__, platform=platform.platform(),
                  logicalCpuCount=os.cpu_count(), workload=build.get('workload'),
                  backendOptimization=build.get('backendOptimization','explicit retained recipe'),
                  durationSeconds=a.duration, headerGapMs=a.header_gap_ms,
                  dependencyDelayMs=a.dependency_ms, loadShares=a.shares or 'equal',
                  warmupRequestsPerApp=a.warmup_requests, reusedProcessAcrossRates=a.reuse_processes,
                  perAppClientInflightLimit=a.concurrency, rounds=a.rounds,
                  target=dict(p99Milliseconds=a.p99_target_ms, successfulOfferedFraction=1,
                              dispatchLagP99Milliseconds=5), trials=[], inputs={})
    for path in [source, Path(__file__).resolve(), loader]:
        report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

    async def checked(*args):
        return await exchange(*args, header_gap=a.header_gap_ms/1000,
                              dependency_port=dependency.port if dependency else None)

    async def warm(servers):
        async def one(server):
            began = time.perf_counter()
            outcomes, latencies = [], []
            for _ in range(a.warmup_requests):
                started = time.perf_counter()
                outcomes.append(await checked(server.port, '/json', 256, 30))
                latencies.append(time.perf_counter()-started)
            assert outcomes == ['ok']*a.warmup_requests, outcomes
            return dict(totalSeconds=time.perf_counter()-began, requestLatenciesSeconds=latencies)
        return await asyncio.gather(*(one(server) for server in servers))

    async def windows(servers, rate):
        shares = a.shares or [1]*len(servers)
        return await asyncio.gather(*(load.window(server.port, '/json', 256, rate*share/sum(shares),
            a.duration, a.concurrency, a.timeout, request_fn=checked) for server, share in zip(servers, shares)))

    dependency = Dependency(a.dependency_ms/1000) if a.dependency_ms else None
    extra_env = {}
    try:
        groups = ([(apps, mode, a.rates) for apps in a.apps for mode in a.modes] if a.reuse_processes else
                  [(apps, mode, [rate]) for apps in a.apps for rate in a.rates for mode in a.modes])
        for repetition in range(a.rounds):
            for group_index, (apps, mode, rates) in enumerate(groups if repetition % 2 == 0 else reversed(groups)):
                servers, scenario_rows = [], []
                dependency_before = dependency.requests if dependency else 0
                group_id = f'{repetition}-{group_index}'
                try:
                    for _ in range(apps):
                        servers.append(Server(build['recipes'][mode], extra_env))
                    ready = [server.ready() for server in servers]
                    cold = asyncio.run(warm(servers))
                    for rate in rates if repetition % 2 == 0 else reversed(rates):
                        row = dict(round=repetition, apps=apps, aggregateOfferedRate=rate, mode=mode,
                                   processGroup=group_id, servers=[])
                        report['trials'].append(row)
                        scenario_rows.append(row)
                        before = [load.resource_sample(server.process.pid) for server in servers]
                        measurements = asyncio.run(windows(servers, rate))
                        after = [load.resource_sample(server.process.pid) for server in servers]
                        for index, measurement in enumerate(measurements):
                            successes = measurement['outcomes'].get('ok', 0)
                            measurement['meetsTarget'] = (successes == measurement['offered'] and
                                measurement['offeredLatencySeconds']['p99'] is not None and
                                measurement['offeredLatencySeconds']['p99'] <= a.p99_target_ms/1000 and
                                measurement['dispatchLagSeconds']['p99'] <= .005)
                            row['servers'].append(dict(readySeconds=ready[index], coldWarmup=cold[index],
                                measurement=measurement,
                                cpuSecondsDuringTrafficAndDrain=after[index]['cpuSeconds']-before[index]['cpuSeconds'],
                                rssBeforeKiB=before[index]['rssKiB'], rssAfterKiB=after[index]['rssKiB']))
                        print(repetition, apps, rate, mode, [m['outcomes'] for m in measurements], flush=True)
                        (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
                    all_successful = True
                    total_completed = 0
                    for index, server in enumerate(servers):
                        totals = server.stop()
                        per_server = [row['servers'][index]['measurement'] for row in scenario_rows]
                        successes = sum(m['outcomes'].get('ok', 0) for m in per_server)
                        admitted = sum(m['admitted'] for m in per_server)
                        all_successful = all_successful and successes == admitted
                        if successes == admitted:
                            assert totals == [a.warmup_requests+successes, 0], totals
                        total_completed += totals[0]
                        jit = json.loads(next(line[len('JIT_RESULT '):] for line in server.stderr if line.startswith('JIT_RESULT ')))
                        assert jit['guestExitCode'] == 0
                        if build['recipes'][mode]['collector'] == 'live':
                            counts = next(list(map(int, line.split()[1:])) for line in server.stderr if line.startswith('CANDIDATE '))
                            assert counts[0] == counts[1]+counts[2] and counts[5] == 0
                        for row in scenario_rows:
                            row['servers'][index].update(jit=jit, serverTotals=totals,
                                serverTotalsScope='entire process group, including all rate windows and warmup',
                                stdout=''.join(server.stdout), stderr=''.join(server.stderr))
                    if dependency:
                        calls = dependency.requests-dependency_before
                        assert calls > 0, 'handler did not execute the dependency'
                        if all_successful:
                            assert calls == total_completed, (calls, total_completed)
                        for row in scenario_rows:
                            row['dependencyRequestsForProcessGroup'] = calls
                    for row in scenario_rows:
                        row['cleanShutdown'] = True
                except Exception as error:
                    report.setdefault('failedScenarios', []).append(dict(round=repetition, apps=apps, mode=mode,
                        processGroup=group_id, error=repr(error),
                        outputs=[dict(stdout=''.join(s.stdout), stderr=''.join(s.stderr)) for s in servers]))
                    raise
                finally:
                    for server in servers:
                        server.close()
                    (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
        report['completed'] = True
    finally:
        if dependency:
            report['dependency'] = dict(requests=dependency.requests, peakInflight=dependency.peak)
            dependency.close()
        (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
