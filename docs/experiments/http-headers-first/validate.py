#!/usr/bin/env python3
"""Prove every server entry point dispatches before the body arrives."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest


def run(command, mode, case, startup_timeout=180):
    guest = Guest([*command, mode], timeout=startup_timeout)
    try:
        port = int(guest.line()[0])
        control_port = int(guest.line()[0])
        with socket.create_connection(('127.0.0.1', control_port), timeout=10) as control:
            with socket.create_connection(('127.0.0.1', port), timeout=10) as peer:
                body = b'123' if case == 'json' else 'Café'.encode()
                target = 'text' if case == 'cancel' else case
                peer.sendall(f'POST /{target} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {len(body)}\r\n\r\n'.encode())
                guest.line(b'HEADERS')  # Must arrive before sending any body bytes.
                if case in ('text', 'json', 'cancel'):
                    peer.settimeout(.1)
                    try:
                        data = peer.recv(1)
                        raise AssertionError(('response before body', data))
                    except socket.timeout:
                        pass
                    peer.settimeout(10)
                    if case == 'cancel':
                        control.sendall(b'x')
                    else:
                        peer.sendall(body[:2])
                        peer.sendall(body[2:])
                response = b''
                while True:
                    try:
                        part = peer.recv(4096)
                    except ConnectionResetError:
                        break
                    if not part:
                        break
                    response += part
                if case == 'cancel':
                    assert response == b'', response
                else:
                    expected = b'unread' if case == 'ignore' else body
                    assert response.startswith(b'HTTP/1.1 200 '), response
                    assert response.split(b'\r\n\r\n', 1)[1] == expected, response
                    control.sendall(b'x')
                guest.line(b'DONE')
                guest.finish()
                return dict(passed=True, headersBeforeBody=True, responseBytes=len(response))
    finally:
        guest.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--startup-timeout', type=int, default=180)
    parser.add_argument('--jit-compiler', type=Path)
    parser.add_argument('--engine', action='append', choices=['aot', 'jit', 'interpreter'])
    parser.add_argument('--entry-point', action='append', choices=['context', 'response', 'one', 'accept', 'headers'])
    args = parser.parse_args()
    bundle, build = args.bundle.resolve(), args.build.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    context = ['--system', str(bundle / manifest['runtimeSeed']), '--object-root', str(bundle / manifest['objectRoot'])]
    for module in manifest['modules']:
        context += ['--module', str(bundle / module)]
    commands = {
        'aot': [str(build / 'app')],
        'jit': [str(bundle / manifest['runtime']), 'run', str(build / 'app.dll'), '--'],
        'interpreter': [str(bundle / manifest['runtime']), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--instructions', '2000000000', *context, '--'],
    }
    if args.jit_compiler:
        commands['jit'][-1:-1] = ['--jit-compiler', str(args.jit_compiler.resolve())]
    report = dict(passed=False, commands=commands, cases={}, inputs={})
    for path in (Path(__file__), Path(__file__).with_name('Probe.rvn'), build / 'app', build / 'app.dll', bundle / 'manifest.json'):
        report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    if args.jit_compiler:
        report['inputs'][str(args.jit_compiler.resolve())] = hashlib.sha256(args.jit_compiler.read_bytes()).hexdigest()
    try:
        for engine in args.engine or commands:
            command = commands[engine]
            for mode in args.entry_point or ('context', 'response', 'one', 'accept', 'headers'):
                for case in ('ignore', 'text', 'json', 'cancel'):
                    key = f'{engine}-{mode}-{case}'
                    report['cases'][key] = run(command, mode, case, args.startup_timeout)
                    print('PASS', key, flush=True)
        report['passed'] = True
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
