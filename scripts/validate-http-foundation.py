#!/usr/bin/env python3
"""Exercise bounded HTTP/1.1 framing and binary payloads with independent peers."""
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
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('http_json', ROOT / 'scripts/verify-native-http-json.py')
http_json = importlib.util.module_from_spec(spec)
spec.loader.exec_module(http_json)


def payload(size):
    return bytes(range(256)) * (size // 256) + bytes(range(size % 256))


def chunks(body):
    return b''.join(f'{len(part):X}\r\n'.encode() + part + b'\r\n'
                    for start in range(0, len(body), 4093)
                    for part in [body[start:start + 4093]]) + b'0\r\n\r\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--aot', type=Path, required=True)
    parser.add_argument('--mode', action='append', choices=('interpreter', 'native'), default=None)
    parser.add_argument('--case', action='append', help='Select named cases, e.g. server-fixed-0')
    parser.add_argument('--large-bodies', action='store_true', help='Also attempt 1 MiB native transfers; currently limited by stress-GC/deadline interaction')
    parser.add_argument('--part', choices=('all', 'transport', 'json', 'client', 'server'), default='all')
    parser.add_argument('--reuse-builds', type=Path, help='Reuse source/hash-verified builds from a prior validation directory')
    args = parser.parse_args()
    if args.case and args.part != 'server':
        parser.error('--case currently requires --part server')
    modes = args.mode or ['interpreter', 'native']
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    fixture = ROOT / 'docs/experiments/http-foundation'
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    report = dict(passed=False, platform=platform.platform(), cases=[], skips=[],
                  sources={str(p.relative_to(ROOT)): sha(p) for p in [Path(__file__), *fixture.glob('*.rvn'), *fixture.glob('*.rvnproj'), *fixture.glob('*.json'),
                           * (ROOT / 'runtime/raven/src/System/Web/Http').rglob('*.rvn')]},
                  runtimeSha256=sha(args.runtime), aotSha256=sha(args.aot),
                  bundleManifestSha256=sha(bundle / 'manifest.json'))
    commands = {}

    def command(project, mode, parameters):
        return commands[project][mode] + (['--'] if mode == 'interpreter' else []) + parameters

    def record(name, mode, result, **extra):
        item = dict(case=name, mode=mode, exitCode=result.returncode,
                    stdout=result.stdout, stderr=result.stderr, **extra)
        report['cases'].append(item)
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
        print(mode + ': ' + name + (' PASS' if result.returncode == 0 else ' FAIL'), flush=True)
        if result.returncode or 'PASS' not in result.stdout:
            raise RuntimeError(str(item))

    try:
        projects = ([args.part.title()] if args.part in ('client', 'server') else ['Client', 'Server'] if args.part == 'transport' else ['Json', 'JsonServer'] if args.part == 'json' else ['Client', 'Server', 'Json', 'JsonServer'])
        for project in projects:
            build = out / project
            if args.reuse_builds:
                build = args.reuse_builds.resolve() / project
                previous = json.loads((args.reuse_builds / 'validation.json').read_text())
                evidence = json.loads((build / 'build.json').read_text())
                if not evidence['passed']:
                    raise RuntimeError('Cannot reuse failed build: ' + project)
                for path, digest in evidence['inputs'].items():
                    if sha(Path(path)) != digest:
                        raise RuntimeError('Reused build input changed: ' + path)
                for item in ET.parse(fixture / (project + '.rvnproj')).getroot().iter('Compile'):
                    source = fixture / item.attrib['Include']
                    if previous['sources'][str(source.relative_to(ROOT))] != sha(source):
                        raise RuntimeError('Reused source changed: ' + str(source))
                if previous['bundleManifestSha256'] != sha(bundle / 'manifest.json'):
                    raise RuntimeError('Reused bundle changed')
            else:
                result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'), '--profile',
                                         'windows-http' if os.name == 'nt' else 'http', '--project', str(fixture / (project + '.rvnproj')),
                                         '--bundle', str(bundle), '--aot', str(args.aot.resolve()), '--output', str(build),
                                         '--command-timeout', '600', '--byte-array-limit', '4194304', '--http-heap-bytes', '33554432',
                                         *(['--reflection-roots', str(fixture / (project + '.reflection-roots.json'))] if project.startswith('Json') else [])],
                                        cwd=ROOT, capture_output=True, text=True, timeout=900)
                (out / (project + '.stdout.log')).write_text(result.stdout)
                (out / (project + '.stderr.log')).write_text(result.stderr)
                if result.returncode:
                    raise RuntimeError(project + ' build failed; see logs')
            lib = bundle / 'lib'
            interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--system', str(lib / 'System.runtime.neox'),
                           '--object-root', str(lib / 'System.Runtime.dll'), '--instructions', '2000000000',
                           '--array-elements', '16777216', '--array-bytes', '33554432']
            for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
                interpreter += ['--module', str(lib / (name + '.dll'))]
            commands[project] = {'interpreter': interpreter, 'native': [str(build / ('app.exe' if os.name == 'nt' else 'app'))]}

        for mode in modes:
            if args.part in ('all', 'json'):
                result = subprocess.run(command('Json', mode, []), cwd=ROOT, capture_output=True, text=True, timeout=60)
                record('json-content', mode, result)
                process = http_json.start_process(command('JsonServer', mode, []), cwd=ROOT,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
                try:
                    port = int(http_json.read_port(process.stdout, timeout=120))
                    with socket.create_connection(('127.0.0.1', port), timeout=30) as connection:
                        body = b'{"message":"hello"}'
                        connection.sendall(f'POST / HTTP/1.1\r\nHost: localhost:{port}\r\nContent-Length: {len(body)}\r\n\r\n'.encode() + body)
                        response = b''
                        while True:
                            part = connection.recv(4096)
                            if not part: break
                            response += part
                    stdout, stderr = process.communicate(timeout=30)
                    record('json-server', mode, subprocess.CompletedProcess(process.args, process.returncode, stdout.decode(), stderr.decode()))
                    if not response.startswith(b'HTTP/1.1 201 ') or b'application/json; charset=utf-8' not in response or json.loads(response.split(b'\r\n\r\n', 1)[1]) != {"message": "hello"}:
                        raise RuntimeError('JSON response wire mismatch')
                finally:
                    http_json.stop_process(process)
            if args.part == 'json':
                continue
            # Interpreter validates the same parser at useful small boundaries;
            # native mode additionally validates the whole-body allocation boundary.
            sizes = [0, 2049] if mode == 'interpreter' else [0, 8193, 32769]
            if mode == 'native' and args.large_bodies:
                sizes.append(1048576)
            client_cases = []
            for size in sizes:
                body = payload(size)
                client_cases.extend([
                    (f'fixed-{size}', size, b'HTTP/1.1 200 OK\r\nContent-Length: ' + str(size).encode() + b'\r\n\r\n' + body, 'ok'),
                    (f'chunked-{size}', size, b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n' + chunks(body), 'ok')])
            client_cases += [
                ('close-delimited', 8193, b'HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n' + payload(8193), 'ok'),
                ('large-headers', 0, b'HTTP/1.1 200 OK\r\n' + b'X: value\r\n' * 98 + b'Y: ' + b'a' * 4096 + b'\r\nContent-Length: 0\r\n\r\n', 'ok'),
                ('header-count-limit', 0, b'HTTP/1.1 200 OK\r\n' + b'X: a\r\n' * 101 + b'\r\n', 'limit'),
                ('informational', 8193, b'HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 103 Early Hints\r\nLink: </app.js>\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 8193\r\n\r\n' + payload(8193), 'ok'),
                ('too-many-informational', 0, b'HTTP/1.1 103 Early Hints\r\n\r\n' * 17, 'limit'),
                ('informational-framing', 0, b'HTTP/1.1 100 Continue\r\nContent-Length: 0\r\n\r\n', 'protocol'),
                ('upgrade', 0, b'HTTP/1.1 101 Switching Protocols\r\n\r\n', 'unsupported'),
                ('ambiguous', 0, b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n', 'protocol'),
                ('length-limit', 0, b'HTTP/1.1 200 OK\r\nContent-Length: 1048577\r\n\r\n', 'limit'),
                ('chunk-limit', 0, b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n100001\r\n', 'limit'),
                ('truncated', 0, b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nab', 'protocol'),
                ('invalid-chunk-crlf', 0, b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1\r\naX', 'protocol'),
                ('header-limit', 0, b'HTTP/1.1 200 OK\r\nX: ' + b'a' * 32768, 'limit'),
            ]
            for name, size, response, expected in ([] if args.part == 'server' else client_cases):
                if mode == 'interpreter' and name == 'header-limit':
                    report['skips'].append(dict(case='client-' + name, mode=mode, reason='32 KiB parsing exceeds fixed transport deadline; covered in native mode'))
                    continue
                peer = {}
                with socket.socket() as listener:
                    listener.bind(('127.0.0.1', 0))
                    listener.listen(1)
                    listener.settimeout(120)
                    url = 'http://127.0.0.1:' + str(listener.getsockname()[1]) + '/echo'
                    def serve():
                        try:
                            with listener.accept()[0] as connection:
                                connection.settimeout(120)
                                request = b''
                                while b'\r\n\r\n' not in request:
                                    part = connection.recv(4096)
                                    if not part: raise RuntimeError('Truncated headers')
                                    request += part
                                head, body = request.split(b'\r\n\r\n', 1)
                                while len(body) < size:
                                    part = connection.recv(min(65536, size - len(body)))
                                    if not part: raise RuntimeError('Truncated body')
                                    body += part
                                if not head.startswith(b'POST /echo HTTP/1.1\r\n') or body != payload(size):
                                    raise RuntimeError('Client request mismatch')
                                peer['validated'] = True
                                try:
                                    connection.sendall(response)
                                except (BrokenPipeError, ConnectionResetError):
                                    if expected == 'ok': raise
                        except Exception as error:
                            peer['error'] = str(error)
                    worker = threading.Thread(target=serve, daemon=True)
                    worker.start()
                    result = subprocess.run(command('Client', mode, [url, str(size), expected]), cwd=ROOT,
                                            capture_output=True, text=True, timeout=240)
                    worker.join(timeout=122)
                    record('client-' + name, mode, result, peer=peer)
                    if not peer.get('validated') or 'error' in peer:
                        raise RuntimeError(str(peer))

            server_cases = []
            for size in sizes:
                body = payload(size)
                server_cases += [
                    (f'fixed-{size}', b'Content-Length: ' + str(size).encode() + b'\r\n\r\n' + body, body, 'ok'),
                    (f'chunked-{size}', b'Transfer-Encoding: ChUnKeD\r\n\r\n' + chunks(body), body, 'ok')]
            server_cases += [
                ('bare-lf', b'X: value\n\r\n', b'', 'protocol'),
                ('bare-cr', b'X: value\rX\r\n\r\n', b'', 'protocol'),
                ('invalid-header-name', b'Bad Name: value\r\n\r\n', b'', 'protocol'),
                ('non-ascii-header', b'X: \xff\r\n\r\n', b'', 'protocol'),
                ('many-headers', b''.join(f'X-{i}: value\r\n'.encode() for i in range(30)) + b'Content-Length: 0\r\n\r\n', b'', 'ok'),
                ('large-header', b'X-Padding: ' + b'a' * 4096 + b'\r\n\r\n', b'', 'ok'),
                ('ambiguous', b'Content-Length: 0\r\nTransfer-Encoding: chunked\r\n\r\n', b'', 'protocol'),
                ('ambiguous-reverse', b'Transfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n', b'', 'protocol'),
                ('duplicate-transfer', b'Transfer-Encoding: chunked\r\nTransfer-Encoding: chunked\r\n\r\n', b'', 'protocol'),
                ('duplicate-length', b'Content-Length: 0\r\nContent-Length: 0\r\n\r\n', b'', 'protocol'),
                ('length-limit', b'Content-Length: 1048577\r\n\r\n', b'', 'limit'),
                ('chunk-limit', b'Transfer-Encoding: chunked\r\n\r\n100001\r\n', b'', 'limit'),
                ('chunk-overflow', b'Transfer-Encoding: chunked\r\n\r\nFFFFFFFFFFFFFFFF\r\n', b'', 'limit'),
                ('truncated', b'Transfer-Encoding: chunked\r\n\r\n3\r\nab', b'', 'protocol'),
                ('invalid-crlf', b'Transfer-Encoding: chunked\r\n\r\n1\r\naX', b'', 'protocol'),
                ('empty-size', b'Transfer-Encoding: chunked\r\n\r\n\r\n', b'', 'protocol'),
                ('unsupported-coding', b'Transfer-Encoding: gzip, chunked\r\n\r\n', b'', 'unsupported'),
                ('unsupported-trailer', b'Transfer-Encoding: chunked\r\n\r\n0\r\nX: y\r\n\r\n', b'', 'unsupported'),
                ('header-count-limit', b'X: a\r\n' * 100 + b'\r\n', b'', 'limit'),
                ('header-byte-limit', b'X: ' + b'a' * 32768, b'', 'limit'),
            ]
            if args.case and set(args.case) - {'server-' + c[0] for c in server_cases}:
                raise ValueError('Unknown or unavailable server case')
            for name, message, body, expected in ([] if args.part == 'client' else server_cases):
                if args.case and 'server-' + name not in args.case:
                    continue
                if mode == 'interpreter' and name == 'header-byte-limit':
                    report['skips'].append(dict(case='server-' + name, mode=mode, reason='32 KiB parsing exceeds fixed transport deadline; covered in native mode'))
                    continue
                process = http_json.start_process(command('Server', mode, [expected]), cwd=ROOT,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
                try:
                    port = int(http_json.read_port(process.stdout, timeout=120))
                    with socket.create_connection(('127.0.0.1', port), timeout=30) as connection:
                        wire = f'POST /echo HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n'.encode() + message
                        try:
                            # Deliberately fragment small requests across header/chunk boundaries.
                            if len(wire) < 10000:
                                for offset in range(0, len(wire), 7):
                                    connection.sendall(wire[offset:offset + 7])
                            else:
                                connection.sendall(wire)
                            connection.shutdown(socket.SHUT_WR)
                        except (BrokenPipeError, ConnectionResetError):
                            if expected == 'ok': raise
                        response = b''
                        while True:
                            try: part = connection.recv(65536)
                            except ConnectionResetError: break
                            if not part: break
                            response += part
                    stdout, stderr = process.communicate(timeout=60)
                    result = subprocess.CompletedProcess(process.args, process.returncode, stdout.decode(), stderr.decode())
                    record('server-' + name, mode, result, responseBytes=len(response), responseSha256=hashlib.sha256(response).hexdigest())
                    if expected == 'ok':
                        if not response.startswith(b'HTTP/1.1 200 OK\r\n') or response.split(b'\r\n\r\n', 1)[1] != body:
                            raise RuntimeError('Server response mismatch: ' + name)
                    elif response:
                        raise RuntimeError('Rejected request unexpectedly produced a response')
                finally:
                    http_json.stop_process(process)
            print('PASS HTTP foundation in ' + mode, flush=True)
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
