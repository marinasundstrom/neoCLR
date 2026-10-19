#!/usr/bin/env python3
"""Copy incoming fixed, chunked and EOF-delimited HTTP bodies into a stream."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest


def run(command):
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        listener.listen(1)
        listener.settimeout(180)
        address = 'http://127.0.0.1:' + str(listener.getsockname()[1]) + '/'
        arguments = (['--'] if len(command) > 1 else []) + [address]
        guest = Guest(command + arguments, timeout=240)
        rows = []
        try:
            for case in range(5):
                peer, _ = listener.accept()
                with peer:
                    peer.settimeout(180)
                    request = b''
                    while b'\r\n\r\n' not in request:
                        part = peer.recv(4096)
                        assert part, 'early request EOF'
                        request += part
                    assert request.startswith(b'GET / HTTP/1.1\r\n')
                    framing = b'Content-Length: 2049\r\n' if case in (0, 3) else b'Transfer-Encoding: chunked\r\n' if case in (1, 4) else b''
                    peer.sendall(b'HTTP/1.1 200 OK\r\n' + framing + b'Connection: close\r\n\r\n')
                    guest.line(b'HEADERS')
                    if case in (0, 2):
                        peer.sendall(b'x' * 2049)
                    elif case == 1:
                        peer.sendall(b'400\r\n' + b'x' * 1024 + b'\r\n401\r\n' + b'x' * 1025 + b'\r\n0\r\n\r\n')
                    elif case == 3:
                        peer.sendall(b'xxx')
                    else:
                        peer.sendall(b'g\r\n')
                    if case in (2, 3):
                        peer.shutdown(socket.SHUT_WR)
                    while peer.recv(4096):
                        raise AssertionError('unexpected client output')
                guest.line(b'CASE')
                rows.append(dict(case=case, passed=True, headersBeforeBody=True))
            guest.line(b'DONE')
            guest.finish()
            return rows
        finally:
            guest.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bundle, build = args.bundle.resolve(), args.build.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    context = ['--system', str(bundle / manifest['runtimeSeed']), '--object-root', str(bundle / manifest['objectRoot'])]
    for module in manifest['modules']:
        context += ['--module', str(bundle / module)]
    commands = {
        'aot': [str(build / 'app')],
        'interpreter': [str(bundle / manifest['runtime']), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--instructions', '2000000000', *context],
    }
    report = dict(passed=False, scope='Loopback body-copy correctness; not throughput or TLS qualification.', commands=commands, cases={}, inputs={str(build / name): hashlib.sha256((build / name).read_bytes()).hexdigest() for name in ('app', 'app.dll', 'build.json')})
    try:
        for mode, command in commands.items():
            report['cases'][mode] = run(command)
            print('PASS', mode, flush=True)
        report['passed'] = True
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
