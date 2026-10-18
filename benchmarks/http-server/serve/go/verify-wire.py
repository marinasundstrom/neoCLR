#!/usr/bin/env python3
"""Verify both compiled fixtures emit the same header fields and body, then join."""
import argparse
import asyncio
import hashlib
import importlib.util
import json
from pathlib import Path
import socket

spec = importlib.util.spec_from_file_location('load', Path(__file__).parents[2]/'load/run.py')
load = importlib.util.module_from_spec(spec)
spec.loader.exec_module(load)
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--neoclr', type=Path, required=True)
p.add_argument('--go', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
report = dict(passed=False, runs=[])
try:
    for name, executable in [('neoclr', a.neoclr), ('go', a.go)]:
        guest = load.Guest([str(executable.resolve()), '16'])
        try:
            port, control = int(guest.line()[0]), int(guest.line()[0])
            with socket.create_connection(('127.0.0.1', port), timeout=10) as conn:
                conn.sendall(b'GET /plain HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                wire = bytearray()
                while True:
                    part = conn.recv(16384)
                    if not part: break
                    wire.extend(part)
            head, body = bytes(wire).split(b'\r\n\r\n', 1)
            lines = head.split(b'\r\n')
            headers = dict(line.lower().split(b': ', 1) for line in lines[1:])
            assert lines[0] == b'HTTP/1.1 200 OK'
            assert headers == {b'content-length': b'10', b'connection': b'close'}
            assert body == b'x'*10
            assert asyncio.run(load.request(port, '/stop', 0, 20)) == 'ok'
            with socket.create_connection(('127.0.0.1', control), timeout=10): pass
            guest.line(b'DONE')
            totals = [int(guest.line()[0]) for _ in range(3)]
            assert totals == [1, 0, 0]
            guest.finish()
            report['runs'].append(dict(runtime=name, executableSha256=hashlib.sha256(executable.read_bytes()).hexdigest(),
                                       wire=wire.decode(), totals=totals))
        finally: guest.close()
    assert report['runs'][0]['wire'] == report['runs'][1]['wire']
    report['passed'] = True
finally:
    a.output.parent.mkdir(parents=True, exist_ok=True)
    a.output.write_text(json.dumps(report, indent=2)+'\n')
