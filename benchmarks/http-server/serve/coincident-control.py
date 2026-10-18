#!/usr/bin/env python3
"""Observe a trivial peer alongside Serve in one generator; diagnostic, not capacity."""
import argparse
import asyncio
import hashlib
import importlib.util
import json
from pathlib import Path
import socket
import sys

spec = importlib.util.spec_from_file_location('load', Path(__file__).parents[1] / 'load/run.py')
load = importlib.util.module_from_spec(spec)
spec.loader.exec_module(load)
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--executable', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rate', type=float, default=150)
p.add_argument('--duration', type=float, default=30)
p.add_argument('--control-rate', type=float, default=100)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
peer = Path(__file__).with_name('generator-control.py')
commands = [[str(a.executable.resolve()), '16'], [sys.executable, str(peer)]]
report = dict(purpose='Two independent peers, shared generator loop; additional traffic is diagnostic only. Common lag does not prove a particular OS cause.',
              commands=commands, serveRate=a.rate, controlRate=a.control_rate, durationSeconds=a.duration,
              inputs={str(f.resolve()): hashlib.sha256(f.read_bytes()).hexdigest() for f in
                      [Path(__file__), Path(load.__file__), peer, a.executable]},
              loadBefore=load.load_average(), processesBefore=load.process_snapshot())
guests = []
ports = []
try:
    for command in commands:
        guest = load.Guest(command)
        guests.append(guest)
        ports.append((int(guest.line()[0]), int(guest.line()[0])))
    warmups = [asyncio.run(load.window(port, '/plain', 10, 10, 2, 64, 3)) for port, _ in ports]

    async def measure():
        async def run(index):
            start = asyncio.get_running_loop().time()
            rate = a.rate if index == 0 else a.control_rate
            m = await load.window(ports[index][0], '/plain', 10, rate, a.duration, 64, 3,
                                  guests[index].process.pid, diagnostics=True)
            return dict(startMonotonicSeconds=start, measurement=m)
        return await asyncio.gather(run(0), run(1))

    results = asyncio.run(measure())
    report['serve'], report['control'] = results
    for index, guest in enumerate(guests):
        port, control = ports[index]
        assert asyncio.run(load.request(port, '/stop', 0, 20)) == 'ok'
        with socket.create_connection(('127.0.0.1', control), timeout=10): pass
        guest.line(b'DONE')
        totals = [int(guest.line()[0]) for _ in range(3)]
        results[index]['totals'] = totals
        assert totals == [warmups[index]['outcomes'].get('ok', 0) + results[index]['measurement']['outcomes'].get('ok', 0), 0, 0]
        guest.finish()
    report['passed'] = True
finally:
    for guest in guests: guest.close()
    (a.output/'results.json').write_text(json.dumps(report, indent=2)+'\n')
for name in ('serve', 'control'):
    m = report[name]['measurement']
    print(name, m['outcomes'], m['offeredLatencySeconds'], m['dispatchLagSeconds'])
