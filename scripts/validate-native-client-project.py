#!/usr/bin/env python3
"""Qualify native HttpClient DNS/connect and a paired Raven server on both OSes."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import shutil
import socket
import subprocess
import sys
import tarfile
import threading
import time
import urllib.request
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('http_project', ROOT / 'scripts/validate-native-http-project.py')
http = importlib.util.module_from_spec(spec)
spec.loader.exec_module(http)
sha = http.sha

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--bundle', type=Path)
    parser.add_argument('--client-build', type=Path, help='Reuse an unchanged successful local build, checking all recorded inputs/artifacts')
    parser.add_argument('--server-build', type=Path, help='Reuse a matching successful local server build')
    args = parser.parse_args()
    out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), commands=[], cases=[])
    def run(command, name):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=600)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(name + ' failed; see retained logs')
        return result
    try:
        windows = platform.system() == 'Windows'
        if not windows and platform.system() != 'Darwin': raise ValueError('Requires macOS or Windows')
        report['revision'] = run(['git', 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        if args.bundle: bundle = args.bundle.resolve()
        elif windows:
            dependency = http.windows_project
            prereq = out.parent / 'windows-project-prerequisites'; prereq.mkdir(exist_ok=True)
            archive = prereq / dependency.ARCHIVE
            if not archive.exists(): urllib.request.urlretrieve(dependency.URL, archive)
            if sha(archive) != dependency.BUNDLE_SHA: raise ValueError('Bundle hash mismatch')
            bundle = prereq / 'neoclr-native-poc'
            if not bundle.exists():
                with tarfile.open(archive) as tar: tar.extractall(prereq, filter='data')
            report['bundle'] = dict(url=dependency.URL, sha256=sha(archive))
        else: raise ValueError('--bundle is required on macOS')
        if not args.client_build or not args.server_build:
            run(['cargo', 'build', '--locked', '--manifest-path', ROOT / 'tools/aot-poc/Cargo.toml'], 'aot-build')
        aot = ROOT / ('tools/aot-poc/target/debug/neoclr-aot-poc' + ('.exe' if windows else ''))
        report['aotSha256'] = sha(aot)
        if windows and not args.bundle:
            bundle = http.rebuild_libraries(run, bundle, out / 'development toolchain')
        lib = bundle / 'lib'; catalog = json.loads((lib / 'bundle.json').read_text())
        vm = bundle / ('bin/neoclr.exe' if windows else 'bin/neoclr')
        report['interpreterSha256'] = sha(vm)
        context = ['--system', lib / catalog['runtimeSeed'], *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))], '--object-root', lib / 'System.Runtime.dll', '--instructions', '100000000']
        environment = {k: v for k, v in os.environ.items() if k.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
        builds = {}; isolated = {}
        for name, directory, reuse in [('client', 'native-http-client', args.client_build), ('server', 'http-server', args.server_build), ('factories', 'task-factories', None)]:
            project = ROOT / 'docs/experiments' / directory / 'Native.rvnproj'
            if reuse:
                build = reuse.resolve(); b = json.loads((build / 'build.json').read_text())
                if not b['passed'] or any(sha(Path(p)) != digest for p, digest in b['inputs'].items()) or any(sha(build / p) != digest for p, digest in b['artifacts'].items()):
                    raise ValueError('Reused build inputs/artifacts changed')
                shutil.copy2(build / 'build.json', out / (name + '-reused-build.json'))
            else:
                build = out / (name + ' native output')
                run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http' if windows else 'http', '--project', project, '--bundle', bundle, '--aot', aot, '--output', build], name + '-build')
            builds[name] = build
            folder = out / (name + ' executable only'); folder.mkdir()
            binary = folder / ('app.exe' if windows else 'app'); shutil.copy2(build / binary.name, binary)
            isolated[name] = binary
        row = dict(name='task-factories')
        report['cases'].append(row)
        for mode, command in [('interpreter', [vm, 'run', builds['factories'] / 'app.dll', *context]), ('native', [isolated['factories']])]:
            result = run(command, 'factories-' + mode)
            row[mode] = dict(stdout=result.stdout.decode(), stderr=result.stderr.decode(), exitCode=result.returncode)
            if row[mode] != dict(stdout='42\nCafé 🌍\nFactories passed\n', stderr='', exitCode=0):
                raise ValueError('Completed task factory contract failed')
        row['passed'] = True
        report['sources'] = {p.relative_to(ROOT).as_posix(): sha(p) for p in [Path(__file__).resolve(), ROOT / 'docs/experiments/native-http-client/Client.rvn', ROOT / 'docs/experiments/native-http-client/Native.rvnproj', ROOT / 'docs/experiments/http-server/Server.rvn', ROOT / 'docs/experiments/http-server/Native.rvnproj']}
        commands = {'interpreter': [vm, 'run', builds['client'] / 'app.dll', *context], 'native': [isolated['client']]}
        def client(mode, url, action):
            result = subprocess.run(list(map(str, commands[mode])), input=(url + '\n' + action + '\n').encode(), capture_output=True, timeout=25,
                                    cwd=isolated['client'].parent if mode == 'native' else ROOT, env=environment if mode == 'native' else None)
            return dict(exitCode=result.returncode, stdout=result.stdout.decode(), stderr=result.stderr.decode())
        body = 'Café 🌍'.encode()
        response = b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\n' + body
        for name in ('greeting', 'numeric', 'fragmented', 'malformed', 'refused', 'timeout', 'pre-cancel', 'cancel', 'continuation-fault'):
            row = dict(name=name); report['cases'].append(row)
            for mode in commands:
                listener = socket.socket(); listener.bind(('127.0.0.1', 0)); port = listener.getsockname()[1]
                wire = {}; thread = None
                if name in ('refused', 'pre-cancel', 'cancel'): listener.close()
                else:
                    listener.listen(); listener.settimeout(15)
                    def fixture():
                        try:
                            with listener.accept()[0] as peer:
                                peer.settimeout(10); request = b''
                                while b'\r\n\r\n' not in request:
                                    part = peer.recv(4096)
                                    if not part or len(request) + len(part) > 4096: raise ValueError('Invalid client request')
                                    request += part
                                wire['requestHex'] = request.hex()
                                if name == 'timeout': peer.recv(1)
                                elif name == 'malformed': peer.sendall(b'not-http\r\n\r\n')
                                elif name == 'fragmented':
                                    for at in range(0, len(response), 3):
                                        peer.sendall(response[at:at+3]); time.sleep(.002)
                                else: peer.sendall(response)
                        except Exception as error: wire['error'] = str(error)
                        finally: listener.close()
                    thread = threading.Thread(target=fixture); thread.start()
                try:
                    host = '127.0.0.1' if name == 'numeric' else 'localhost'
                    action = name if name in ('cancel', 'pre-cancel') else 'fault' if name == 'continuation-fault' else 'normal'
                    row[mode] = client(mode, f'http://{host}:{port}/greeting', action)
                finally:
                    if thread: thread.join(timeout=16)
                if thread and (thread.is_alive() or 'error' in wire): raise ValueError(name + ': fixture failed ' + repr(wire))
                if thread:
                    expected = f'GET /greeting HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n'.encode()
                    if bytes.fromhex(wire['requestHex']) != expected: raise ValueError('Unexpected HTTP request')
                row[mode + 'Wire'] = wire
            if row['interpreter'] != row['native']: raise ValueError(name + ': interpreter/native mismatch')
            value = row['native']
            if name in ('greeting', 'numeric', 'fragmented'):
                if value != dict(exitCode=0, stdout='Café 🌍\n', stderr=''): raise ValueError(name + ': response mismatch')
            elif name in ('cancel', 'pre-cancel'):
                if value['exitCode'] != 1 or 'cancel' not in value['stderr'].lower(): raise ValueError(name + ': cancellation mismatch')
            elif name == 'continuation-fault':
                if value['exitCode'] != 1 or 'Client continuation fault' not in value['stderr']: raise ValueError('Missing client fault')
            elif value['exitCode'] or value['stderr'] or not value['stdout'].startswith('Error: '): raise ValueError(name + ': missing typed error')
            row['passed'] = True
            print(name + ': PASS', flush=True)
        row = dict(name='paired-raven-server'); report['cases'].append(row)
        for mode in commands:
            command = [isolated['server']] if mode == 'native' else [vm, 'run', builds['server'] / 'app.dll', *context]
            server = subprocess.Popen(list(map(str, command)), stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0,
                                      cwd=isolated['server'].parent if mode == 'native' else ROOT, env=environment if mode == 'native' else None)
            try:
                port = http.read_port(server.stdout, timeout=30)
                if not port.isdigit(): raise ValueError('Raven server missing port')
                row[mode] = client(mode, f'http://localhost:{int(port)}/greeting', 'normal')
                stdout, stderr = server.communicate(timeout=20)
                row[mode + 'Server'] = dict(exitCode=server.returncode, stdout=stdout.decode(), stderr=stderr.decode())
            finally:
                if server.poll() is None: server.kill(); server.communicate()
            if row[mode] != dict(exitCode=0, stdout='Café 🌍\n', stderr='') or row[mode+'Server'] != dict(exitCode=0, stdout='Other work runs while HTTP accept is pending\nServed greeting; server closed\n', stderr=''): raise ValueError('Paired Raven server/client mismatch')
        row['passed'] = True
        for binary in isolated.values():
            if sorted(p.name for p in binary.parent.iterdir()) != [binary.name]: raise ValueError('Executable-only directory changed')
        report.update(passed=len(report['cases']) == 11, standalone=True)
    except Exception as error: report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Native HTTP client: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']), flush=True)
    return 0 if report['passed'] else 1
if __name__ == '__main__': raise SystemExit(main())
