#!/usr/bin/env python3
"""Queued cancellation/admission recovery and existing headers-first body contracts."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import socket
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest

spec = importlib.util.spec_from_file_location('body_contract', ROOT / 'docs/experiments/http-body-stream/validate.py')
body_contract = importlib.util.module_from_spec(spec)
spec.loader.exec_module(body_contract)


def queue_case(command):
    guest = Guest(command, timeout=180)
    try:
        port, _ = guest.line()
        with socket.create_connection(('127.0.0.1', int(port)), timeout=30) as peer:
            # Fragmentation also exercises deferred parser creation and subsequent reads.
            peer.sendall(b'GET /recovered HTTP/1.1\r\nHost: localhost\r\n')
            peer.sendall(b'\r\n')
            wire = bytearray()
            while True:
                part = peer.recv(4096)
                if not part:
                    break
                wire.extend(part)
                assert len(wire) < 4096
        head, body = bytes(wire).split(b'\r\n\r\n', 1)
        assert head.startswith(b'HTTP/1.1 200 OK\r\n') and body == b'OK', wire
        guest.line(b'DONE')
        guest.finish()
        return dict(passed=True, command=command,
                    checks='16-slot limit; queued middle/tail/head cancellation; replacement admission; active accept and successor cancellation; pre-cancellation; listener recovery; 16 pending shutdown cancellations; closed-server rejection')
    finally:
        guest.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--queue', type=Path, required=True)
    p.add_argument('--body', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    bundle = a.bundle.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    context = ['--system', str(bundle / manifest['runtimeSeed']), '--object-root', str(bundle / manifest['objectRoot'])]
    for module in manifest['modules']:
        context += ['--module', str(bundle / module)]
    report = dict(passed=False, cases={}, inputs={})
    sources = [Path(__file__), Path(__file__).with_name('QueueProbe.rvn'),
               ROOT/'docs/experiments/http-body-stream/Common.rvn',
               ROOT/'docs/experiments/http-body-stream/server/Main.rvn',
               ROOT/'docs/experiments/http-body-stream/validate.py', bundle/'manifest.json']
    for build in [a.queue.resolve(), a.body.resolve()]:
        sources += [build/name for name in ['app', 'app.dll', 'build.json']]
    report['inputs'] = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources}
    try:
        for kind, build in [('queue', a.queue.resolve()), ('body', a.body.resolve())]:
            commands = {
                'aot': [str(build/'app')],
                'jit': [str(bundle/manifest['runtime']), 'run', str(build/'app.dll')],
                'interpreter': [str(bundle/manifest['runtime']), 'run', str(build/'app.dll'),
                                '--execution', 'interpreter', '--instructions', '2000000000', *context],
            }
            for mode, command in commands.items():
                report['cases'][kind+'-'+mode] = queue_case(command) if kind == 'queue' else body_contract.run_case(command, 'server')
                print('PASS', kind, mode, flush=True)
        report['passed'] = True
    finally:
        a.output.write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
