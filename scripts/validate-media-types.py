#!/usr/bin/env python3
"""Validate media type wire formatting, incoming parsing and content guards."""
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
    # Keep codegen/runtime matched to the tested metadata and compiler bundle.
    if args.runtime == parser.get_default('runtime'):
        args.runtime = bundle / 'bin' / ('neoclr.exe' if os.name == 'nt' else 'neoclr')
    if args.aot == parser.get_default('aot'):
        args.aot = bundle / 'bin' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen')
    out.mkdir(parents=True, exist_ok=False)
    fixture = ROOT / 'docs/experiments/media-types'
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
            interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--system', str(lib / 'System.runtime.neox'),
                           '--object-root', str(lib / 'System.Runtime.dll'), '--instructions', '100000000']
            for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
                interpreter += ['--module', str(lib / (name + '.dll'))]
            commands[project] = {'interpreter': interpreter, 'native': [str(build / ('app.exe' if os.name == 'nt' else 'app'))]}
        for mode in ('interpreter', 'native'):
            command = commands['Client'][mode]
            record = run_case('conflicting-charset', mode, command, ['charset'])
            if not record['exitCode'] or 'HttpStringContent requires charset=utf-8' not in record['stderr']:
                raise RuntimeError('Conflicting charset did not fault')
            for invalid in ('not-a-media-type', 'text/plain; x=a; X=b', 'text/*',
                            'text/plain; x =a', 'text/plain; x="unterminated',
                            'text/plain; x="a\r\nb"', 'text/plain; q=0.5'):
                record = run_case('invalid-before-dns', mode, command, ['invalid', invalid])
                record['input'] = invalid
                if record['exitCode'] or 'PASS invalid media type before DNS' not in record['stdout']:
                    raise RuntimeError('Invalid media type guard failed')
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
                            while len(body) < 2:
                                part = connection.recv(2 - len(body))
                                if not part: raise RuntimeError('Truncated body')
                                body += part
                            if not head.startswith(b'POST /items HTTP/1.1\r\n') or b'Content-Length: 2' not in head or b'Content-Type: application/json; profile="A; B"' not in head or body != b'{}':
                                raise RuntimeError('Media type wire mismatch')
                            peer['validated'] = True
                            connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Type: application/problem+json\r\nContent-Length: 2\r\n\r\n{}')
                    except Exception as error:
                        peer['error'] = str(error)
                worker = threading.Thread(target=serve, daemon=True)
                worker.start()
                record = run_case('media-type-client', mode, command, [url])
                worker.join(timeout=6)
                record['peer'] = peer
                if record['exitCode'] or 'PASS media type client' not in record['stdout'] or not peer.get('validated'):
                    raise RuntimeError('Media type client failed: ' + repr(record))
            process = http_json.start_process(commands['Server'][mode], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
            try:
                port = int(http_json.read_port(process.stdout, timeout=30))
                with socket.create_connection(('127.0.0.1', port), timeout=10) as connection:
                    connection.sendall(f'POST /items HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: multipart/form-data; boundary="AbC; 123"\r\nContent-Length: 0\r\n\r\n'.encode())
                    response = b''
                    while True:
                        part = connection.recv(4096)
                        if not part: break
                        response += part
                stdout, stderr = process.communicate(timeout=20)
                record = dict(case='media-type-server', mode=mode, exitCode=process.returncode,
                              stdout=stdout.decode(), stderr=stderr.decode(), responseHex=response.hex())
                report['cases'].append(record)
                if process.returncode or b'PASS media type server' not in stdout or b'Content-Type: application/problem+json\r\n' not in response or not response.endswith(b'\r\n\r\n{}'):
                    raise RuntimeError('Media type server failed: ' + repr(record))
            finally:
                http_json.stop_process(process)
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS media types and content guards in both modes')


if __name__ == '__main__':
    main()
