#!/usr/bin/env python3
"""Real Serve error recovery and concurrent peers with shadow tracing enabled."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import sys
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[[str((a.build / 'app-paced').resolve())]])
os.environ['NEOCLR_GC_SHADOW_REPORT'] = str(out / 'shadow.json')
guest = Guest(report['commands'][0], timeout=30)
peers = []
def connect(port, request):
    peer = socket.create_connection(('127.0.0.1', port), timeout=10)
    peers.append(peer)
    peer.sendall(request)
    return peer
def read(peer):
    data = b''
    while True:
        part = peer.recv(4096)
        if not part: return data
        data += part
        assert len(data) < 4096

def valid(data, size):
    head, body = data.split(b'\r\n\r\n', 1)
    lines = head.split(b'\r\n')
    fields = dict(line.lower().split(b': ', 1) for line in lines[1:])
    assert lines[0] == b'HTTP/1.1 200 OK' and body == b'x' * size
    assert fields[b'content-length'] == str(size).encode() and fields[b'connection'] == b'close'
try:
    port = int(guest.line()[0]); control = int(guest.line()[0])
    malformed = connect(port, b'GET / HTTP/1.1\r\n\r\n')
    assert read(malformed) == b''
    batch = [connect(port, b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n') for _ in range(16)]
    for peer in batch: valid(read(peer), 10)
    valid(read(connect(port, b'GET / HTTP/1.1\r\nHost: localhost\r\n\r\n')), 10)
    valid(read(connect(port, b'GET /stop HTTP/1.1\r\nHost: localhost\r\n\r\n')), 0)
    with socket.create_connection(('127.0.0.1', control), timeout=10): pass
    guest.line(b'DONE')
    totals = [int(guest.line()[0]) for _ in range(3)]
    assert totals == [17, 0, 1], totals
    guest.finish()
    shadow = json.loads((out / 'shadow.json').read_text())
    assert shadow['started'] > 0 and shadow['applied'] == 0
    assert shadow['traceErrors'] == 0 and shadow['captureRejected'] == 0
    assert shadow['allocations'] == shadow['reclaimedAllocations']
    report.update(passed=True, serverTotals=totals, shadow=shadow)
finally:
    for peer in peers: peer.close()
    guest.close()
    report['inputs'] = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest()
                        for path in [Path(__file__), a.build / 'app-paced', a.build / 'build.json']}
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print('HTTP shadow recovery/concurrent peers: PASS')
