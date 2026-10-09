#!/usr/bin/env python3
"""Build a Raven HTTP project and compare standalone native/interpreter requests."""
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
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]

def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

windows_project = load('windows_project', ROOT / 'scripts/validate-windows-project.py')
read_port = load('http_json', ROOT / 'scripts/verify-native-http-json.py').read_port

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def rebuild_libraries(run, bundle, output):
    # Preview 13 supplies the pinned compiler and primitive bootstrap. Public
    # development APIs come from this checkout, not the published library payload.
    run([sys.executable, ROOT / 'scripts/prepare-native-development-bundle.py', '--bundle', bundle,
         '--output', output, '--compiler-revision', '71cafd353'], 'development-libraries')
    return output / 'bundle'

def serve(command, request, fragmented, cwd, env=None):
    process = subprocess.Popen(list(map(str, command)), cwd=cwd, env=env,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    response = b''
    try:
        port = read_port(process.stdout, timeout=30)
        if not port.isdigit() or not 0 < int(port) < 65536:
            raise ValueError('Missing valid listening port: ' + repr(port))
        with socket.create_connection(('127.0.0.1', int(port)), timeout=10) as peer:
            if fragmented:
                for at in range(0, len(request), 3):
                    peer.sendall(request[at:at + 3])
                    time.sleep(.002)
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
                        raise ValueError('Oversized response')
            except ConnectionResetError:
                pass
        stdout, stderr = process.communicate(timeout=20)
        return dict(exitCode=process.returncode, stdout=stdout.decode(), stderr=stderr.decode(), responseHex=response.hex())
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, help='Required on macOS; Windows defaults to pinned Preview 13 bundle')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), cases=[], commands=[])
    def run(command, name, expected=0):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=600)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode))
        if result.returncode != expected:
            raise RuntimeError(name + ' failed; see retained logs')
        return result
    try:
        windows = platform.system() == 'Windows'
        if not windows and platform.system() != 'Darwin':
            raise ValueError('Requires macOS ARM64 or Windows x64')
        report['revision'] = run(['git', 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        if args.bundle:
            bundle = args.bundle.resolve()
        elif windows:
            prereq = out.parent / 'windows-project-prerequisites'
            prereq.mkdir(exist_ok=True)
            archive = prereq / windows_project.ARCHIVE
            if not archive.exists():
                urllib.request.urlretrieve(windows_project.URL, archive)
            if sha(archive) != windows_project.BUNDLE_SHA:
                raise ValueError('Bundle archive hash mismatch')
            bundle = prereq / 'neoclr-native-poc'
            if not bundle.exists():
                with tarfile.open(archive) as source:
                    source.extractall(prereq, filter='data')
            report['bundle'] = dict(url=windows_project.URL, sha256=sha(archive))
        else:
            raise ValueError('--bundle is required on macOS')
        run(['cargo', 'build', '--locked', '--manifest-path', ROOT / 'tools/aot-poc/Cargo.toml'], 'aot-build')
        aot = ROOT / ('tools/aot-poc/target/debug/neoclr-aot-poc' + ('.exe' if windows else ''))
        report['aotSha256'] = sha(aot)
        if windows and not args.bundle:
            bundle = rebuild_libraries(run, bundle, out / 'development toolchain')
        lib = bundle / 'lib'
        catalog = json.loads((lib / 'bundle.json').read_text())
        context = ['--system', lib / catalog['runtimeSeed'],
                   *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))],
                   '--object-root', lib / 'System.Runtime.dll']
        source = (ROOT / 'docs/experiments/http-server/Server.rvn').read_text()
        valid = b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
        expected = (b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n'
                    b'Content-Type: text/plain; charset=utf-8\r\n\r\n' + 'Café 🌍'.encode())
        cases = [('greeting', valid, False, None), ('fragmented', valid, True, None),
                 ('duplicate-length', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n', False, 'Duplicate request Content-Length'),
                 ('handler-error', valid.replace(b'/greeting', b'/other'), False, 'Unexpected request')]
        for fault in (False, True):
            name = 'fault' if fault else 'server'
            project = out / (name + ' project with spaces')
            project.mkdir()
            (project / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>HttpShowcase</AssemblyName></PropertyGroup></Project>')
            code = source.replace('func Respond(request: HttpRequest) -> Task<Result<HttpResponse, HttpError>> {', 'func Respond(request: HttpRequest) -> Task<Result<HttpResponse, HttpError>> {\n    System.Fail("HTTP callback fault")') if fault else source
            (project / 'Main.rvn').write_text(code, encoding='utf-8')
            destination = out / (name + ' native output')
            command = [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http' if windows else 'http',
                       '--project', project / 'App.rvnproj', '--bundle', bundle, '--aot', aot, '--output', destination]
            run(command, name + '-build')
            build = json.loads((destination / 'build.json').read_text())
            if not build['passed']:
                raise ValueError('Build report did not pass')
            isolated = out / (name + ' executable only')
            isolated.mkdir()
            executable = isolated / ('app.exe' if windows else 'app')
            shutil.copy2(destination / executable.name, executable)
            env = {key: value for key, value in os.environ.items() if key.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
            vm = [bundle / ('bin/neoclr.exe' if windows else 'bin/neoclr'), 'run', destination / 'app.dll', *context, '--instructions', '100000000']
            for label, request, fragmented, error in ([('callback-fault', valid, False, None)] if fault else cases):
                row = dict(name=label, dependencies=build['dependencies'])
                report['cases'].append(row)
                row['interpreter'] = serve(vm, request, fragmented, ROOT)
                row['native'] = serve([executable], request, fragmented, isolated, env)
                if row['native'] != row['interpreter']:
                    raise ValueError(label + ': interpreter/native mismatch')
                result = row['native']
                if fault:
                    if result['exitCode'] != 1 or 'HTTP callback fault' not in result['stderr'] or result['responseHex']:
                        raise ValueError('Missing callback fault or unexpected response')
                else:
                    stdout = 'Other work runs while HTTP accept is pending\n' + ('Served greeting; server closed\n' if error is None else 'Server error: ' + error + '\n')
                    if result != dict(exitCode=0, stdout=stdout, stderr='', responseHex=(expected if error is None else b'').hex()):
                        raise ValueError(label + ': unexpected response/output')
                row['passed'] = True
            if sorted(p.name for p in isolated.iterdir()) != [executable.name]:
                raise ValueError('Standalone directory changed')
            if not fault:
                before = sha(executable)
                run(command, 'existing-output-rejected', expected=1)
                if sha(destination / executable.name) != before:
                    raise ValueError('Existing executable changed')
                (project / 'Main.rvn').write_text('func Main( { invalid')
                failed = out / 'failed rebuild'
                run([*command[:-1], failed], 'stale-output-rejected', expected=1)
                if (failed / executable.name).exists() or list(failed.glob('app.pending*')):
                    raise ValueError('Failed rebuild published executable')
                (project / 'Main.rvn').write_text(code, encoding='utf-8')
        report.update(passed=len(report['cases']) == 5, standalone=True, existingOutputPreserved=True, staleOutputRejected=True)
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Native HTTP project: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
