#!/usr/bin/env python3
"""Verify client status metadata with an independent HTTP/1.1 peer."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
import threading

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest


def run(command):
    ready = threading.Event()
    errors = []
    listener = socket.socket()
    listener.bind(('127.0.0.1', 0))
    listener.listen(4)
    listener.settimeout(180)
    cases = [('/custom', 201, b'Created\tHere  '), ('/empty', 204, b''),
             ('/ok', 200, b'OK'), ('/informational', 299, b'Nearly OK'),
             ('/error', 404, b'Different text'), ('/disposed', 204, b'Still readable'),
             ('/stream', 200, b'Body Later')]

    def serve():
        try:
            for path, code, phrase in cases:
                peer, _ = listener.accept()
                with peer:
                    peer.settimeout(10)
                    request = b''
                    while b'\r\n\r\n' not in request:
                        chunk = peer.recv(4096)
                        assert chunk, 'request closed before headers'
                        request += chunk
                    assert request.startswith(f'GET {path} HTTP/1.1\r\n'.encode()), request
                    if path == '/informational':
                        peer.sendall(b'HTTP/1.1 100 Interim phrase\r\n\r\n')
                    header = f'HTTP/1.1 {code} '.encode() + phrase + b'\r\n'
                    if code != 204:
                        header += b'Content-Length: 1\r\n'
                    peer.sendall(header + b'\r\n')
                    if path == '/stream':
                        assert ready.wait(10), 'metadata getter waited for body'
                    if code != 204:
                        peer.sendall(b'x')
        except Exception as error:
            errors.append(repr(error))

    worker = threading.Thread(target=serve, daemon=True)
    worker.start()
    guest = Guest([*command, f'http://127.0.0.1:{listener.getsockname()[1]}'], timeout=180)
    try:
        guest.line(b'HEADERS_READY')
        ready.set()
        guest.line(b'PASS HTTP response metadata')
        guest.finish()
        worker.join(10)
        assert not worker.is_alive(), 'peer did not finish'
        assert not errors, errors
        return dict(passed=True, responses=len(cases), metadataBeforeBody=True)
    finally:
        ready.set()
        listener.close()
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
    commands = {'aot': [str(build / 'app'), '--'],
                'jit': [str(bundle / manifest['runtime']), 'run', str(build / 'app.dll'), '--'],
                'interpreter': [str(bundle / manifest['runtime']), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--instructions', '1000000000', *context, '--']}
    # Native application arguments have no CLI delimiter.
    commands['aot'].pop()
    report = dict(passed=False, cases={}, commands=commands, inputs={})
    for path in (Path(__file__), Path(__file__).with_name('ReasonPhrase.rvn'), build / 'app', build / 'app.dll', bundle / 'manifest.json'):
        report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    try:
        for mode, command in commands.items():
            report['cases'][mode] = run(command)
            print('PASS', mode, flush=True)
        report['passed'] = True
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
