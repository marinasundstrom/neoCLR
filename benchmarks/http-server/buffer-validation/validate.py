#!/usr/bin/env python3
"""Check fragmented request growth, response block boundaries and snapshot ownership."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
import time
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
b = a.bundle.resolve()
manifest = json.loads((b / 'manifest.json').read_text())
context = ['--system', str(b / manifest['runtimeSeed']), '--object-root', str(b / manifest['objectRoot'])]
for module in manifest['modules']:
    context += ['--module', str(b / module)]
assembly = str(a.build.resolve() / 'app.dll')
commands = {
    'interpreter': [str(b / manifest['runtime']), 'run', assembly, '--execution', 'interpreter', '--instructions', '2000000000', '--array-elements', '2097152', '--array-bytes', '268435456', *context],
    'jit': [str(b / manifest['runtime']), 'run', assembly],
    'aot': [str(a.build.resolve() / 'app')],
}
report = dict(passed=False, commands=commands, cases=[], inputs={})
for path in [Path(__file__), Path(__file__).with_name('Server.rvn'), a.build / 'app.dll', a.build / 'app', b / 'manifest.json']:
    report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
try:
    for mode in ('aot', 'jit', 'interpreter'):
        command = commands[mode]
        guest = Guest(command)
        try:
            port, _ = guest.line()
            for size in [0, 255, 256, 257, 4096, 65410, 65536, 70000]:
                with socket.create_connection(('127.0.0.1', int(port)), timeout=60) as peer:
                    peer.sendall(b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 4097\r\n\r\n' + b'z' * 200)
                    time.sleep(.02)
                    peer.sendall(b'z' * 3897)
                    wire = bytearray()
                    while True:
                        data = peer.recv(997)
                        if not data:
                            break
                        wire.extend(data)
                        if len(wire) > size + 32768:
                            raise ValueError('Oversized response')
                if b'\r\n\r\n' not in wire:
                    guest.process.wait(timeout=10)
                    raise RuntimeError(f'Incomplete response: {mode}, {size}, {guest.errors()!r}')
                head, content = bytes(wire).split(b'\r\n\r\n', 1)
                lines = head.split(b'\r\n')
                fields = dict(line.lower().split(b': ', 1) for line in lines[1:])
                assert lines[0] == b'HTTP/1.1 200 OK', lines[0]
                assert fields[b'content-length'] == str(size).encode(), fields
                assert fields[b'connection'] == b'close', fields
                assert content == b'x' * size, (mode, size, len(content), content[-4:])
                report['cases'].append(dict(mode=mode, size=size, passed=True))
            guest.line(b'DONE')
            guest.finish()
        finally:
            guest.close()
    report['passed'] = True
finally:
    a.output.write_text(json.dumps(report, indent=2) + '\n')
print('PASS', len(report['cases']), 'buffer and snapshot cases')
