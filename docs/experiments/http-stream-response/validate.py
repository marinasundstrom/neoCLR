#!/usr/bin/env python3
"""Validate bounded server response streaming against an independent wire client."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'benchmarks/framework'))
from process import Guest

def run_case(command):
    guest = Guest(command, timeout=180)
    results = []
    try:
        port, ready = guest.line()
        for mode in range(10):
            method = b'HEAD' if mode == 2 else b'GET'
            with socket.create_connection(('127.0.0.1', int(port)), timeout=90) as peer:
                peer.sendall(method+b' / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                wire = b''
                while True:
                    part = peer.recv(113)
                    if not part:
                        break
                    wire += part
                    assert len(wire) < 16384
            guest.line(b'CASE')
            if mode == 7:
                assert wire == b'', wire
            else:
                head, body = wire.split(b'\r\n\r\n', 1)
                assert head.startswith(b'HTTP/1.1 200 OK\r\n')
                length = 0 if mode == 3 else 8193 if mode == 1 else 1025
                assert b'Content-Length: '+str(length).encode() in head
                expected = bytes(i%251 for i in range(length)) if mode in (0,1) else b''
                assert body == expected, (mode,len(body))
            results.append(dict(case=mode,passed=True,wireBytes=len(wire)))
        guest.line(b'DONE')
        guest.finish()
        return dict(passed=True,cases=results)
    except Exception as error:
        raise RuntimeError(f'case {mode}: '+str(error)+'; guest errors: '+str(guest.errors())) from error
    finally:
        guest.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build', type=Path, required=True)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--mode', action='append', choices=['jit', 'aot', 'interpreter'])
    a = p.parse_args()
    b = a.bundle.resolve()
    manifest = json.loads((b/'manifest.json').read_text())
    context = ['--system', str(b/manifest['runtimeSeed']), '--object-root', str(b/manifest['objectRoot'])]
    for module in manifest['modules']:
        context += ['--module', str(b/module)]
    app = a.build.resolve()/'app.dll'
    commands = {'jit': [str(b/manifest['runtime']), 'run', str(app)],
                'aot': [str(a.build.resolve()/'app')],
                'interpreter': [str(b/manifest['runtime']), 'run', str(app), '--execution', 'interpreter', '--instructions', '2000000000', *context]}
    report = dict(passed=False, commands=commands, modes={}, inputs={},
                  scope='Known-length server response streams: deferred short reads, owned/borrowed sources, HEAD, empty body, premature EOF, source error, overread disposed content, caller cancellation and deadline cancellation of a waiting producer; not a throughput measurement')
    for path in [Path(__file__), Path(__file__).with_name('Server.rvn'), app, a.build/'app', a.build/'build.json', b/'manifest.json']:
        report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
    try:
        for mode in a.mode or ['jit', 'aot', 'interpreter']:
            report['modes'][mode] = run_case(commands[mode])
            print('PASS', mode, flush=True)
        report['passed'] = True
    finally:
        a.output.write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
