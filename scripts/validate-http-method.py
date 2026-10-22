#!/usr/bin/env python3
"""Validate custom method wire spelling, server admission and constructor guards."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import socket
import subprocess
import sys
import threading

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('http_json', ROOT / 'scripts/verify-native-http-json.py')
http_json = importlib.util.module_from_spec(spec)
spec.loader.exec_module(http_json)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    parser.add_argument('--aot', type=Path, default=ROOT / 'target/debug' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen'))
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    fixture = ROOT / 'docs/experiments/http-method'
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  sources={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in fixture.iterdir()
                           if p.suffix in ('.rvn', '.rvnproj')},
                  bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest())
    commands = {}
    def run_case(case, mode, command, parameters):
        result = subprocess.run([*command, *(['--'] if mode == 'interpreter' else []), *parameters],
                                cwd=ROOT, capture_output=True, text=True, timeout=30)
        record = dict(case=case, mode=mode, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
        report['cases'].append(record)
        return record
    try:
        for project in ('Client', 'Server'):
            build = out / project
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'), '--profile',
                                     'windows-http' if os.name == 'nt' else 'http', '--project', str(fixture / (project + '.rvnproj')),
                                     '--bundle', str(bundle), '--aot', str(args.aot.resolve()), '--output', str(build)],
                                    cwd=ROOT, capture_output=True, text=True, timeout=600)
            (out / (project + '.stdout.log')).write_text(result.stdout)
            (out / (project + '.stderr.log')).write_text(result.stderr)
            if result.returncode:
                raise RuntimeError(project + ' build failed; see logs')
            lib = bundle / 'lib'
            interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--system', str(lib / 'System.runtime.neox'),
                           '--object-root', str(lib / 'System.Runtime.dll'), '--instructions', '100000000']
            for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
                interpreter += ['--module', str(lib / (name + '.dll'))]
            commands[project] = {'interpreter': interpreter, 'native': [str(build / ('app.exe' if os.name == 'nt' else 'app'))]}
        for mode in ('interpreter', 'native'):
            command = commands['Client'][mode]
            for invalid in ('', 'GET ', 'GET\r\nX: y'):
                record = run_case('invalid-constructor', mode, command, ['invalid', invalid])
                if not record['exitCode'] or 'Invalid HTTP method' not in record['stderr'] or 'ERROR' in record['stdout']:
                    raise RuntimeError('Invalid constructor did not fault')
            record = run_case('connect-unsupported', mode, command, ['connect'])
            if record['exitCode'] or 'PASS CONNECT remains unsupported' not in record['stdout']:
                raise RuntimeError('CONNECT transport guard failed')
            peer = {}
            with socket.socket() as listener:
                listener.bind(('127.0.0.1', 0))
                listener.listen(1)
                listener.settimeout(20)
                url = 'http://127.0.0.1:' + str(listener.getsockname()[1]) + '/items'
                def serve():
                    try:
                        with listener.accept()[0] as connection:
                            connection.settimeout(5)
                            request = b''
                            while b'\r\n\r\n' not in request:
                                part = connection.recv(4096)
                                if not part: raise RuntimeError('Truncated headers')
                                request += part
                            head, body = request.split(b'\r\n\r\n', 1)
                            while len(body) < 4:
                                part = connection.recv(4 - len(body))
                                if not part: raise RuntimeError('Truncated body')
                                body += part
                            if not head.startswith(b'm-SEARCH /items HTTP/1.1\r\n') or b'Content-Length: 4' not in head or body != b'body':
                                raise RuntimeError('Custom method wire mismatch')
                            peer['validated'] = True
                            connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nbody')
                    except Exception as error:
                        peer['error'] = str(error)
                worker = threading.Thread(target=serve, daemon=True)
                worker.start()
                record = run_case('custom-client', mode, command, [url])
                worker.join(timeout=6)
                record['peer'] = peer
                if record['exitCode'] or 'PASS custom method client' not in record['stdout'] or not peer.get('validated'):
                    raise RuntimeError('Custom client request failed: ' + repr(record))
            process = http_json.start_process(commands['Server'][mode], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
            try:
                port = int(http_json.read_port(process.stdout, timeout=30))
                with socket.create_connection(('127.0.0.1', port), timeout=10) as connection:
                    connection.sendall(f'm-SEARCH /items HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 4\r\n\r\nbody'.encode())
                    response = b''
                    while True:
                        part = connection.recv(4096)
                        if not part: break
                        response += part
                stdout, stderr = process.communicate(timeout=20)
                record = dict(case='custom-server', mode=mode, exitCode=process.returncode,
                              stdout=stdout.decode(), stderr=stderr.decode(), responseHex=response.hex())
                report['cases'].append(record)
                if process.returncode or b'PASS custom method server' not in stdout or b'X-Method: m-SEARCH\r\n' not in response or not response.endswith(b'\r\n\r\nbody'):
                    raise RuntimeError('Custom server response failed: ' + repr(record))
            finally:
                http_json.stop_process(process)
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS custom HTTP methods and constructor/CONNECT guards in both modes')


if __name__ == '__main__':
    main()
