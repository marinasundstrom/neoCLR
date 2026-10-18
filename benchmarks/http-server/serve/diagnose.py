#!/usr/bin/env python3
"""Native sampling or independent trivial-peer control; keep separate from capacity runs."""
import argparse
import asyncio
import hashlib
import importlib.util
import json
from pathlib import Path
import socket
import subprocess
import sys

spec = importlib.util.spec_from_file_location('load', Path(__file__).parents[1] / 'load/run.py')
load = importlib.util.module_from_spec(spec)
spec.loader.exec_module(load)
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--executable', type=Path)
p.add_argument('--sample', action='store_true')
p.add_argument('--rate', type=float, default=300)
p.add_argument('--duration', type=float, default=20)
p.add_argument('--concurrency', type=int, default=64)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
peer = Path(__file__).with_name('generator-control.py')
command = [str(a.executable.resolve()), '16'] if a.executable else [sys.executable, str(peer)]
report = dict(command=command, rate=a.rate, duration=a.duration, concurrency=a.concurrency,
              purpose='Diagnostic only; sampling changes timing. Trivial Python peer tests generator headroom, not comparative server performance.',
              inputs={str(f.resolve()): hashlib.sha256(f.read_bytes()).hexdigest() for f in
                      [Path(__file__), Path(load.__file__), a.executable or peer]},
              loadBefore=load.load_average(), processesBefore=load.process_snapshot())
guest = load.Guest(command)
try:
    port = int(guest.line()[0]); control = int(guest.line()[0])
    report['warmup'] = asyncio.run(load.window(port, '/plain', 10, 10, 2, a.concurrency, 3))
    before = load.resource_sample(guest.process.pid)

    async def measure():
        async def sample():
            await asyncio.sleep(3)
            command = ['sample', str(guest.process.pid), '5', '1', '-file', str(a.output/'sample.txt')]
            result = await asyncio.to_thread(subprocess.run, command, capture_output=True, text=True)
            report['sample'] = dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
        tasks = [load.window(port, '/plain', 10, a.rate, a.duration, a.concurrency, 3,
                             guest.process.pid, diagnostics=True)]
        if a.sample: tasks.append(sample())
        return (await asyncio.gather(*tasks))[0]

    report['measurement'] = asyncio.run(measure())
    m = report['measurement']
    m['serverCpuSecondsInWindow'] = m['endOfWindowResource']['cpuSeconds'] - before['cpuSeconds']
    assert asyncio.run(load.request(port, '/stop', 0, 20)) == 'ok'
    with socket.create_connection(('127.0.0.1', control), timeout=10): pass
    guest.line(b'DONE')
    totals = [int(guest.line()[0]) for _ in range(3)]
    report['totals'] = totals
    assert totals == [report['warmup']['outcomes'].get('ok', 0) + m['outcomes'].get('ok', 0), 0, 0]
    guest.finish()
    report['passed'] = True
finally:
    guest.close()
    report['loadAfter'] = load.load_average()
    (a.output/'results.json').write_text(json.dumps(report, indent=2)+'\n')
print(m['outcomes'], m['offeredLatencySeconds'], m['dispatchLagSeconds'])
