#!/usr/bin/env python3
"""Check configured HTTP admission, immutable snapshots and read deadlines."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import platform
import socket
import subprocess

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('http_peer', ROOT / 'scripts/verify-native-http-json.py')
peer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(peer)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'build', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    lib = args.bundle.resolve() / 'lib'
    build = args.build.resolve()
    evidence = json.loads((build / 'build.json').read_text())
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    if not evidence['passed'] or any(sha(Path(p)) != digest for p, digest in evidence['inputs'].items()):
        raise RuntimeError('Build evidence is not current')
    interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--system', str(lib / 'System.runtime.neox'),
                   '--object-root', str(lib / 'System.Runtime.dll'), '--instructions', '200000000']
    for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
        interpreter += ['--module', str(lib / (name + '.dll'))]
    cases = [
        ('fixed-boundary-and-snapshot', 'normal', 'ok', b'Content-Length: 4\r\n\r\nABCD', b'ABCD'),
        ('chunked-boundary', 'normal', 'ok', b'Transfer-Encoding: chunked\r\n\r\n2\r\nAB\r\n2\r\nCD\r\n0\r\n\r\n', b'ABCD'),
        ('fixed-over-limit', 'normal', 'limit', b'Content-Length: 5\r\n\r\nABCDE', None),
        ('chunked-cumulative-limit', 'normal', 'limit', b'Transfer-Encoding: chunked\r\n\r\n2\r\nAB\r\n3\r\nCDE\r\n0\r\n\r\n', None),
        ('header-count', 'normal', 'limit', b'X: a\r\nY: b\r\nContent-Length: 0\r\n\r\n', None),
        ('header-bytes', 'normal', 'limit', b'X: ' + b'a' * 256 + b'\r\n\r\n', None),
        ('zero-empty', 'zero', 'ok', b'Content-Length: 0\r\n\r\n', b''),
        ('zero-body', 'zero', 'limit', b'Transfer-Encoding: chunked\r\n\r\n1\r\nA\r\n0\r\n\r\n', None),
        ('read-deadline', 'timeout', 'timeout', b'', None),
    ]
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  runtimeSha256=sha(args.runtime), bundleManifestSha256=sha(args.bundle / 'manifest.json'),
                  buildEvidenceSha256=sha(build / 'build.json'),
                  sources={str(p.relative_to(ROOT)): sha(p) for p in [Path(__file__), Path(__file__).with_name('Main.rvn')]})
    try:
        for mode, command in [('interpreter', interpreter + ['--']), ('native', [str(build / 'app')])]:
            for name, setting, expected, fields, body in cases:
                process = subprocess.Popen(command + [setting, expected], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
                try:
                    port = int(peer.read_port(process.stdout, timeout=30))
                    with socket.create_connection(('127.0.0.1', port), timeout=30) as connection:
                        wire = b'P' if setting == 'timeout' else f'POST / HTTP/1.1\r\nHost: localhost:{port}\r\n'.encode() + fields
                        connection.sendall(wire)
                        response = b''
                        while True:
                            try:
                                part = connection.recv(4096)
                            except ConnectionResetError:
                                break
                            if not part:
                                break
                            response += part
                    stdout, stderr = process.communicate(timeout=30)
                    valid = process.returncode == 0 and b'PASS server options' in stdout and not stderr
                    if body is None:
                        valid = valid and not response
                    else:
                        valid = valid and response.startswith(b'HTTP/1.1 200 OK\r\n') and response.split(b'\r\n\r\n', 1)[1] == body
                    report['cases'].append(dict(case=name, mode=mode, passed=valid, exitCode=process.returncode,
                                                stdout=stdout.decode(), stderr=stderr.decode(), responseBytes=len(response)))
                    print(mode, name, 'PASS' if valid else 'FAIL', flush=True)
                    if not valid:
                        raise RuntimeError(str(report['cases'][-1]))
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.communicate()
        report['passed'] = True
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
