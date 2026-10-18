#!/usr/bin/env python3
"""Bounded open-loop HTTP load: retain offered arrivals, misses, failures and latency."""
import argparse
import asyncio
from collections import Counter
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import socket
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest
from host import load_average, process_snapshot


def percentiles(values):
    values = sorted(values)
    return {name: values[max(0, math.ceil(len(values) * q) - 1)] if values else None
            for name, q in [('p50', .5), ('p95', .95), ('p99', .99), ('max', 1)]}


def resource_sample(pid):
    output = subprocess.check_output(['ps', '-p', str(pid), '-o', 'time=,rss='], text=True).strip()
    cpu, rss = output.split()
    seconds = 0.0
    for part in cpu.split(':'):
        seconds = seconds * 60 + float(part)
    return dict(cpuSeconds=seconds, rssKiB=int(rss))


async def request(port, path, expected_size, timeout):
    writer = None
    try:
        async def exchange():
            nonlocal writer
            reader, writer = await asyncio.open_connection('127.0.0.1', port)
            writer.write(f'GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'.encode())
            await writer.drain()
            wire = bytearray()
            while True:
                chunk = await reader.read(16384)
                if not chunk:
                    break
                wire.extend(chunk)
                if len(wire) > expected_size + 32768:
                    return 'oversized_response'
            if b'\r\n\r\n' not in wire:
                return 'closed_without_headers'
            head, body = bytes(wire).split(b'\r\n\r\n', 1)
            lines = head.split(b'\r\n')
            fields = dict(line.lower().split(b': ', 1) for line in lines[1:])
            if lines[0] != b'HTTP/1.1 200 OK':
                return 'unexpected_status'
            if fields.get(b'content-length') != str(expected_size).encode() or fields.get(b'connection') != b'close':
                return 'invalid_framing'
            return 'ok' if body == b'x' * expected_size else 'invalid_body'
        return await asyncio.wait_for(exchange(), timeout)
    except asyncio.TimeoutError:
        return 'timeout'
    except (OSError, ValueError, asyncio.IncompleteReadError) as error:
        return type(error).__name__
    finally:
        if writer is not None:
            writer.close()
            try:
                await writer.wait_closed()
            except OSError:
                pass


async def window(port, path, size, rate, duration, concurrency, timeout, pid=None):
    loop = asyncio.get_running_loop()
    start = loop.time()
    count = math.floor(rate * duration)
    records = []
    tasks = set()
    resources = []
    sampling = True

    async def sample_resources():
        while sampling:
            try:
                sample = await asyncio.to_thread(resource_sample, pid)
                resources.append(dict(atSeconds=loop.time() - start, **sample))
            except (OSError, ValueError, subprocess.CalledProcessError) as error:
                resources.append(dict(atSeconds=loop.time() - start, error=str(error)))
            await asyncio.sleep(.5)

    sampler = asyncio.create_task(sample_resources()) if pid else None

    async def arrival(index, intended):
        actual = loop.time()
        result = await request(port, path, size, timeout)
        finished = loop.time()
        records.append(dict(index=index, intendedSeconds=intended-start,
                            startSeconds=actual-start, endSeconds=finished-start,
                            dispatchLagSeconds=actual-intended, result=result,
                            offeredLatencySeconds=finished-intended,
                            serviceLatencySeconds=finished-actual))

    peak = 0
    for index in range(count):
        intended = start + index / rate
        await asyncio.sleep(max(0, intended - loop.time()))
        # No queue and no replacement arrival: admission misses stay in the denominator.
        if len(tasks) >= concurrency:
            records.append(dict(index=index, intendedSeconds=intended-start,
                                dispatchLagSeconds=loop.time()-intended,
                                result='generator_inflight_limit'))
            continue
        task = asyncio.create_task(arrival(index, intended))
        tasks.add(task)
        task.add_done_callback(tasks.discard)
        peak = max(peak, len(tasks))
    await asyncio.sleep(max(0, start + duration - loop.time()))
    cpu_at_end = await asyncio.to_thread(resource_sample, pid) if pid else None
    if tasks:
        await asyncio.gather(*tasks)
    drained = loop.time() - start
    sampling = False
    if sampler:
        await sampler
    records.sort(key=lambda r: r['index'])
    counts = Counter(r['result'] for r in records)
    successes = [r for r in records if r['result'] == 'ok']
    within = [r for r in successes if r['endSeconds'] <= duration]
    usable = [r for r in resources if 'rssKiB' in r]
    return dict(rate=rate, durationSeconds=duration, clientInflightLimit=concurrency,
                offered=count, admitted=count-counts['generator_inflight_limit'], outcomes=dict(counts),
                completedWithinWindow=len(within), successfulCompletionsPerSecond=len(within)/duration,
                successfulOfferedFraction=len(successes)/count if count else None,
                drainSeconds=max(0, drained-duration), peakClientInflight=peak,
                offeredLatencySeconds=percentiles([r['offeredLatencySeconds'] for r in successes]),
                serviceLatencySeconds=percentiles([r['serviceLatencySeconds'] for r in successes]),
                dispatchLagSeconds=percentiles([r['dispatchLagSeconds'] for r in records]),
                sampledPeakRssKiB=max((r['rssKiB'] for r in usable), default=None),
                endOfWindowResource=cpu_at_end, resources=resources, requests=records)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--executable', default='app')
    p.add_argument('--server-source', type=Path, default=Path(__file__).with_name('Server.rvn'))
    p.add_argument('--control-stop', action='store_true', help='Read a second control port and signal shutdown after the stop response')
    p.add_argument('--workers', type=int, choices=[1, 4, 16], default=16)
    p.add_argument('--workload', choices=['plain', 'bulk'], default='plain')
    p.add_argument('--rates', nargs='+', type=float, required=True)
    p.add_argument('--concurrency', nargs='+', type=int, default=[1, 4, 16, 32])
    p.add_argument('--duration', type=float, default=15)
    p.add_argument('--rounds', type=int, default=2)
    p.add_argument('--timeout', type=float, default=3)
    p.add_argument('--warmup', type=float, default=2)
    p.add_argument('--p99-target-ms', type=float, default=250)
    a = p.parse_args()
    if min(a.rates) <= 0 or min(a.concurrency) < 1 or a.duration <= 0 or a.rounds < 1 or a.timeout <= 0 or a.warmup <= 0:
        p.error('Rates, durations, concurrency and rounds must be positive')
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    build = a.build.resolve()
    evidence = json.loads((build / 'build.json').read_text())
    if not evidence['passed']:
        p.error('Build must pass before measurement')
    inputs = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in [Path(__file__).resolve(), build/a.executable, build/'app.dll', build/'build.json',
                           a.server_source.resolve(), ROOT/'benchmarks/framework/process.py']}
    report = dict(completed=False, platform=platform.platform(), inputs=inputs, build=evidence,
                  loadBefore=load_average(), processesBefore=process_snapshot(), trials=[],
                  serverWorkers=a.workers, executable=a.executable, fixtureSource=str(a.server_source.resolve()),
                  contract='AOT loopback, configured server workers, HTTP/1.1 connection close, no TLS. Fixed offered schedule; client cap drops arrivals without queueing. Each response fully validated. Misses/failures remain in denominator. Graceful /stop after draining.',
                  qualification='Exploratory local load sweep, not production capacity. Same-machine generator and sampled RSS; no allocation profiler during timing.',
                  target=dict(p99Milliseconds=a.p99_target_ms, minimumSuccessFraction=1.0,
                              maximumDispatchLagP99Milliseconds=5, note='Provisional reporting target, not a product SLO. Memory stability needs longer runs.'))
    path = '/bulk' if a.workload == 'bulk' else '/plain'
    size = 8192 if a.workload == 'bulk' else 10
    try:
        points = [(c, rate) for c in a.concurrency for rate in a.rates]
        for round_index in range(a.rounds):
            for concurrency, rate in (points if round_index % 2 == 0 else reversed(points)):
                guest = Guest([str(build/a.executable), str(a.workers)])
                trial = dict(round=round_index, workload=a.workload, concurrency=concurrency, rate=rate)
                report['trials'].append(trial)
                try:
                    port, ready = guest.line()
                    control_port = int(guest.line()[0]) if a.control_stop else None
                    trial['startupSeconds'] = ready-guest.started
                    trial['warmup'] = asyncio.run(window(int(port), path, size, min(rate, 10), a.warmup, concurrency, a.timeout))
                    trial['resourceBefore'] = resource_sample(guest.process.pid)
                    trial['measurement'] = asyncio.run(window(int(port), path, size, rate, a.duration, concurrency, a.timeout, guest.process.pid))
                    m = trial['measurement']
                    cpu = m['endOfWindowResource']['cpuSeconds'] - trial['resourceBefore']['cpuSeconds']
                    m['serverCpuSecondsInWindow'] = cpu
                    m['serverCpuSecondsPerCompletion'] = cpu/m['completedWithinWindow'] if m['completedWithinWindow'] else None
                    m['meetsProvisionalTarget'] = (m['successfulOfferedFraction'] == 1 and
                        m['offeredLatencySeconds']['p99'] is not None and m['offeredLatencySeconds']['p99'] <= a.p99_target_ms/1000 and
                        m['dispatchLagSeconds']['p99'] <= .005)
                    stop = asyncio.run(request(int(port), '/stop', 0, 20))
                    if stop != 'ok':
                        raise RuntimeError('Shutdown request: '+stop)
                    if control_port is not None:
                        with socket.create_connection(('127.0.0.1', control_port), timeout=10):
                            pass
                    guest.line(b'DONE')
                    trial['serverTotals'] = dict(zip(['completed', 'limitErrors', 'otherErrors'],
                                                   [int(guest.line()[0]) for _ in range(3)]))
                    successful = trial['warmup']['outcomes'].get('ok', 0) + m['outcomes'].get('ok', 0)
                    trial['serverCompletedMinusClientSuccesses'] = trial['serverTotals']['completed'] - successful
                    if m['outcomes'].get('ok', 0) == m['admitted'] and trial['warmup']['outcomes'].get('ok', 0) == trial['warmup']['admitted']:
                        if trial['serverTotals']['completed'] != successful:
                            raise RuntimeError('Client/server completion counts disagree')
                    guest.finish()
                    trial['cleanShutdown'] = True
                    print(round_index+1, concurrency, rate, m['outcomes'], round(m['successfulCompletionsPerSecond'],2), flush=True)
                except Exception as error:
                    trial['error'] = str(error)
                    trial['stderr'] = guest.errors().decode(errors='replace')
                    raise
                finally:
                    guest.close()
                    (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
        for name, digest in inputs.items():
            if hashlib.sha256(Path(name).read_bytes()).hexdigest() != digest:
                raise RuntimeError('Input drift: '+name)
        report['completed'] = True
    finally:
        report['loadAfter'] = load_average()
        (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
