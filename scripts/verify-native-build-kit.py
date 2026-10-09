#!/usr/bin/env python3
"""Qualify an archived native build kit after extraction outside the source checkout."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import select
import socket
import subprocess
import sys
import tarfile
import tempfile
import time
import xml.etree.ElementTree as X

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def serve(command, directory, request, fragmented=False):
    process = subprocess.Popen(list(map(str, command)), cwd=directory, env={},
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    try:
        if not select.select([process.stdout], [], [], 30)[0]:
            raise RuntimeError('HTTP server did not report its port')
        line = process.stdout.readline()
        if not line.strip().isdigit():
            raise RuntimeError('Expected HTTP port: ' + repr(line))
        response = b''
        with socket.create_connection(('127.0.0.1', int(line)), timeout=10) as peer:
            if fragmented:
                for at in range(0, len(request), 3):
                    peer.sendall(request[at:at + 3])
                    time.sleep(0.002)
            else:
                peer.sendall(request)
            peer.shutdown(socket.SHUT_WR)
            try:
                while True:
                    part = peer.recv(4096)
                    if not part:
                        break
                    response += part
                    if len(response) > 8192:
                        raise RuntimeError('Unexpected HTTP response size')
            except ConnectionResetError:
                pass
        stdout, stderr = process.communicate(timeout=30)
        return dict(exitCode=process.returncode, stdout=stdout.decode(), stderr=stderr.decode(), responseHex=response.hex())
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--http', action='store_true', help='Also qualify the opt-in HTTP profile')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, archiveSha256=sha(args.archive), cases=[])
    try:
        with tempfile.TemporaryDirectory(prefix='native kit extracted with spaces ') as temporary:
            location = Path(temporary).resolve()
            with tarfile.open(args.archive) as archive:
                # Python 3.9-compatible extraction of this regular-file-only format.
                members = archive.getmembers()
                names = set()
                for member in members:
                    destination = (location / member.name).resolve()
                    if (Path(member.name).is_absolute() or location not in destination.parents
                            or not (member.isfile() or member.isdir()) or member.name in names):
                        raise ValueError('Invalid build kit archive member: ' + member.name)
                    names.add(member.name)
                archive.extractall(location, members=members)
            kit = location / 'neoclr-native-build-kit'
            manifest = json.loads((kit / 'native-build-kit.json').read_text())
            report['kitManifestSha256'] = sha(kit / 'native-build-kit.json')
            report['profile'] = manifest['profile']
            report['sourceRevision'] = manifest['revision']
            report['sourceDirty'] = manifest['dirty']
            report['sourceInputs'] = manifest['inputs']
            report['aotBuild'] = manifest['aotBuild']
            report['bundleManifestSha256'] = manifest['bundleManifestSha256']
            report['packagedFileCount'] = len(manifest['files'])
            # Keep OS build tools and dotnet available, but remove the Cargo toolchain.
            environment = {**os.environ, 'PATH': str(Path(shutil.which('dotnet')).parent) +
                           ':/usr/bin:/bin:/usr/sbin:/sbin'}
            assert shutil.which('cargo', path=environment['PATH']) is None

            def build(name, profile='console', project=None):
                destination = location / name
                command = [sys.executable, kit / 'scripts/build-native-project.py',
                           '--project', project or kit / 'samples/hello/App.rvnproj',
                           '--profile', profile, '--output', destination]
                result = subprocess.run(list(map(str, command)), cwd=location, env=environment,
                                        capture_output=True, text=True, timeout=240)
                evidence = json.loads((destination / 'build.json').read_text())
                row = dict(name=name, exitCode=result.returncode, stdout=result.stdout,
                           stderr=result.stderr, build=evidence)
                report['cases'].append(row)
                return result, evidence, destination

            result, evidence, destination = build('native hello output')
            assert result.returncode == 0 and evidence['passed'], result.stderr
            assert all(kit in Path(path).parents for path in evidence['inputs'])
            assert all(str(ROOT) not in str(c['command']) for c in evidence['commands'])
            standalone = location / 'standalone'
            standalone.mkdir()
            shutil.copy2(destination / 'app', standalone / 'app')
            native = subprocess.run([str(standalone / 'app')], cwd=standalone, env={}, capture_output=True, timeout=30)
            assert native.returncode == 0 and native.stdout == 'Hello, Café 🌍\n'.encode() and not native.stderr
            assert list(standalone.iterdir()) == [standalone / 'app']
            report['execution'] = dict(exitCode=0, stdout=native.stdout.decode(), executableOnlyDirectory=True,
                                       emptyEnvironment=True, buildWithoutCargo=True, checkoutPathsAbsent=True)
            for name, relative in [('adapter', 'tools/native/console-host.c'), ('backend', 'bin/neoclr-aot-poc'),
                                   ('compiler', 'bundle/sdk/tools/rvnc/rvnc.runtimeconfig.json')]:
                path = kit / relative
                size = path.stat().st_size
                with path.open('ab') as stream:
                    stream.write(b'\nmodified\n')
                try:
                    result, evidence, destination = build('reject changed ' + name)
                    assert result.returncode != 0 and not evidence['passed']
                    assert evidence['commands'] == [] and 'Package file mismatch' in evidence['error']
                    assert not (destination / 'app').exists() and not (destination / 'app.dll').exists()
                finally:
                    with path.open('r+b') as stream:
                        stream.truncate(size)
            if args.http:
                result, evidence, destination = build('native http output', 'http', kit / 'samples/http/App.rvnproj')
                assert result.returncode == 0 and evidence['passed'], result.stderr
                assert all(kit in Path(path).parents for path in evidence['inputs'])
                assert all(str(ROOT) not in str(c['command']) for c in evidence['commands'])
                http_dir = location / 'http standalone'
                http_dir.mkdir()
                shutil.copy2(destination / 'app', http_dir / 'app')
                lib = kit / 'bundle/lib'
                catalog = json.loads((lib / 'bundle.json').read_text())
                context = ['--system', lib / catalog['runtimeSeed'],
                           *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))],
                           '--object-root', lib / 'System.Runtime.dll']
                valid = b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
                expected = (b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n'
                            b'Content-Type: text/plain; charset=utf-8\r\n\r\n' + 'Café 🌍'.encode())
                cases = [('greeting', valid, False, None), ('fragmented', valid, True, None),
                         ('duplicate-length', valid.replace(b'Connection: close', b'Content-Length: 0\r\nContent-Length: 0'),
                          False, 'Duplicate request Content-Length'),
                         ('handler-error', valid.replace(b'/greeting', b'/other'), False, 'Unexpected request')]
                report['httpCases'] = {}
                for name, request, fragmented, error in cases:
                    vm = serve([kit / 'bundle/bin/neoclr', 'run', destination / 'app.dll', *context], http_dir, request, fragmented)
                    native = serve([http_dir / 'app'], http_dir, request, fragmented)
                    report['httpCases'][name] = dict(interpreter=vm, native=native)
                    assert native == vm
                    assert native['exitCode'] == 0 and not native['stderr']
                    assert bytes.fromhex(native['responseHex']) == (expected if error is None else b'')
                    assert native['stdout'] == ('Other work runs while HTTP accept is pending\n' +
                        ('Served greeting; server closed\n' if error is None else 'Server error: ' + error + '\n'))
                assert list(http_dir.iterdir()) == [http_dir / 'app']
                # Real guest failure from a handler exercises callback fault unwinding.
                fault_project = location / 'HTTP fault project'
                fault_project.mkdir()
                project = X.parse(kit / 'samples/http/App.rvnproj')
                project.getroot().find('Import').set('Project', str(lib / catalog['projectConfiguration']))
                project.write(fault_project / 'App.rvnproj', encoding='unicode')
                source = (kit / 'samples/http/Main.rvn').read_text()
                source = source.replace('    let source = Promise<Result<HttpResponse, HttpError>>()',
                    '    if request.Target == "/fault" {\n        System.Fail("HTTP handler failed")\n    }\n'
                    '    let source = Promise<Result<HttpResponse, HttpError>>()')
                (fault_project / 'Main.rvn').write_text(source)
                result, evidence, destination = build('native http fault output', 'http', fault_project / 'App.rvnproj')
                assert result.returncode == 0 and evidence['passed'], result.stderr
                shutil.copy2(destination / 'app', http_dir / 'app')
                request = valid.replace(b'/greeting', b'/fault')
                vm = serve([kit / 'bundle/bin/neoclr', 'run', destination / 'app.dll', *context], http_dir, request)
                native = serve([http_dir / 'app'], http_dir, request)
                report['httpCases']['guest-fault'] = dict(interpreter=vm, native=native)
                assert native == vm and native['exitCode'] == 1
                assert 'HTTP handler failed' in native['stderr'] and not native['responseHex']
                report['httpStandalone'] = dict(executableOnlyDirectory=True, emptyEnvironment=True,
                                                buildWithoutCargo=True, host='existing bounded HTTP correctness host')
            # Retain compact evidence; command streams remain byte-verifiable.
            for case in report['cases']:
                for command in case['build']['commands']:
                    for field in ('stdout', 'stderr'):
                        value = command[field]
                        if len(value) > 2500:
                            command[field + 'Sha256'] = hashlib.sha256(value.encode()).hexdigest()
                            command[field + 'Bytes'] = len(value.encode())
                            command[field] = value[:1000] + '\n[compacted]\n' + value[-1000:]
            report['passed'] = True
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed: extracted native kit, no checkout/Cargo build dependency, standalone UTF-8 execution and three pre-build integrity rejections')


if __name__ == '__main__':
    main()
