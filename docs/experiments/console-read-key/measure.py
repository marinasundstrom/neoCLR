#!/usr/bin/env python3
"""Measure whole-process native managed allocation counts for 128 public ReadKey calls."""
import argparse
import json
import os
from pathlib import Path
import pty
import re
import select
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('build', type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
build = args.build.resolve()
report = json.loads((build / 'build.json').read_text())
command = next(row['command'] for row in report['commands'] if row.get('phase') == 'host-compile-link').copy()
command[command.index(str(root / 'tools/native/console-host.c'))] = str(Path(__file__).with_name('allocation-host.c').resolve())
command[command.index('-o') + 1] = str(build / 'allocation-probe')
subprocess.run(command, check=True, capture_output=True)
rows = []
for mode in ['baseline', 'allocation']:
    master, slave = pty.openpty()
    process = subprocess.Popen([build / 'allocation-probe', mode], stdin=slave, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        if mode == 'allocation':
            ready = b''
            deadline = time.monotonic() + 30
            while b'READY\n' not in ready:
                assert time.monotonic() < deadline
                if select.select([process.stdout], [], [], .1)[0]:
                    chunk = os.read(process.stdout.fileno(), 4096)
                    assert chunk, process.stderr.read()
                    ready += chunk
            os.write(master, b'a' * 128)
        output, error = process.communicate(timeout=60)
        assert process.returncode == 0 and output.endswith(b'PASS\n'), (output, error)
        counters = {key: int(value) for key, value in re.findall(r'(allocations|collections|reclaimed)=(\d+)', error.decode())}
        assert counters['allocations'] == counters['reclaimed'], counters
        rows.append(dict(mode=mode, **counters))
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        os.close(master)
        os.close(slave)
assert rows[0]['allocations'] == rows[1]['allocations'], rows
args.output.write_text(json.dumps(dict(iterations=128, measurements=rows, scope='Native managed allocations; not host allocator counts, interpreter allocations, latency or throughput', command=command), indent=2) + '\n')
print(json.dumps(rows, indent=2))
