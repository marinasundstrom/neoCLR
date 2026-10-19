#!/usr/bin/env python3
"""Independent peers verify bounded dispatch, error recovery and joined shutdown."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'benchmarks/framework'))
from process import Guest


def connect(port, target):
    peer = socket.create_connection(('127.0.0.1', port), timeout=30)
    peer.sendall(f'GET {target} HTTP/1.1\r\nHost: localhost\r\n\r\n'.encode())
    return peer


def read(peer):
    result = b''
    while True:
        try:
            part = peer.recv(4096)
        except ConnectionResetError:
            return result  # Aborted connection; successful-response assertions still apply.
        if not part:
            return result
        result += part
        assert len(result) < 4096


def run(command, kind):
    guest = Guest(command, timeout=180)
    peers = []
    try:
        port = int(guest.line()[0]); control_port = int(guest.line()[0])
        with socket.create_connection(('127.0.0.1', control_port), timeout=30) as control:
            if kind == 'inline':
                peer = connect(port, '/inline'); peers.append(peer); guest.line(b'HOLD')
                assert read(peer) == b''
                control.sendall(bytes([4]))
                expected = (0, 0, 1)
            else:
                for target in ['/error', '/cancel', '/invalid']:
                    with connect(port, target) as peer: assert read(peer) == b''
                with socket.create_connection(('127.0.0.1', port), timeout=30) as peer:
                    peer.sendall(b'GET / HTTP/1.1\r\n\r\n'); assert read(peer) == b''
                with connect(port, '/ok') as peer: assert read(peer).endswith(b'\r\n\r\nx')
                for _ in range(16):
                    peers.append(connect(port, '/hold')); guest.line(b'HOLD')
                waiting = connect(port, '/ok'); peers.append(waiting)
                waiting.settimeout(.15)
                try:
                    unexpected = waiting.recv(1)
                    raise AssertionError(('17th request bypassed bound', unexpected))
                except socket.timeout:
                    pass
                waiting.settimeout(30)
                control.sendall(bytes([1]))
                assert read(peers[0]).endswith(b'\r\n\r\nx')
                assert read(waiting).endswith(b'\r\n\r\nx')
                control.sendall(bytes([3 if kind == 'cancel' else 2]))
                guest.line(b'CLOSING')
                for peer in peers[1:16]: assert read(peer) == b''
                control.sendall(bytes([4]))
                expected = (3, 4, 15)
            guest.line(b'DONE')
            actual = tuple(int(guest.line()[0]) for _ in range(3))
            assert actual == expected, (actual, expected)
            guest.finish()
            return dict(passed=True, completed=actual[0], failed=actual[1], lateSourcesDisposed=actual[2])
    except Exception as error:
        raise RuntimeError(f'{kind}: {error}; {guest.errors()}') from error
    finally:
        for peer in peers: peer.close()
        guest.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--build', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--source', type=Path, default=Path(__file__).with_name('Probe.rvn'))
    a = p.parse_args(); b=a.bundle.resolve(); build=a.build.resolve()
    manifest=json.loads((b/'manifest.json').read_text())
    context=['--system',str(b/manifest['runtimeSeed']),'--object-root',str(b/manifest['objectRoot'])]
    for module in manifest['modules']: context += ['--module',str(b/module)]
    commands={'aot':[str(build/'app')], 'jit':[str(b/manifest['runtime']),'run',str(build/'app.dll')],
              'interpreter':[str(b/manifest['runtime']),'run',str(build/'app.dll'),'--execution','interpreter','--instructions','2000000000',*context]}
    report=dict(passed=False,commands=commands,cases={},inputs={})
    for path in [Path(__file__),a.source,build/'app',build/'app.dll',build/'build.json',b/'manifest.json']:
        report['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
    try:
        for mode,command in commands.items():
            for kind in ['close','cancel','inline']:
                report['cases'][mode+'-'+kind]=run(command,kind)
                print('PASS',mode,kind,flush=True)
        report['passed']=True
    finally: a.output.write_text(json.dumps(report,indent=2)+'\n')


if __name__ == '__main__': main()
