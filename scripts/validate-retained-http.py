#!/usr/bin/env python3
"""Qualify a retained Raven HTTP session against an await-based interpreter consumer."""
import argparse
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
spec = importlib.util.spec_from_file_location('http_project', ROOT / 'scripts/validate-native-http-project.py')
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
sha = common.sha


def serve(command, targets, fragmented, cwd, env=None):
    process = subprocess.Popen(list(map(str, command)), cwd=cwd, env=env,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    responses = []
    try:
        port = common.read_port(process.stdout, timeout=30)
        if not port.isdigit() or not 0 < int(port) < 65536:
            raise ValueError('Missing listening port: ' + repr(port))
        for target in targets:
            request = ('GET ' + target + ' HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n').encode()
            response = b''
            with socket.create_connection(('127.0.0.1', int(port)), timeout=10) as peer:
                for at in range(0, len(request), 3 if fragmented else len(request)):
                    peer.sendall(request[at:at + (3 if fragmented else len(request))])
                    if fragmented:
                        time.sleep(.002)
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
            responses.append(response.hex())
        stdout, stderr = process.communicate(timeout=20)
        # Both normal and terminal-fault teardown must close the one listener.
        try:
            # Windows can report refusal after its initial SYN retry interval.
            connection = socket.create_connection(('127.0.0.1', int(port)), timeout=5)
        except ConnectionRefusedError:
            pass
        else:
            connection.close()
            raise ValueError('Listener remained open after process exit')
        return dict(exitCode=process.returncode, stdout=stdout.decode().replace('\r\n', '\n'),
                    stderr=stderr.decode().replace('\r\n', '\n'), responses=responses)
    except Exception as error:
        if process.poll() is None:
            process.kill()
        stdout, stderr = process.communicate()
        raise RuntimeError(f'HTTP sequence failed: {error}; exit={process.returncode}; '
                           f'stdout={stdout!r}; stderr={stderr!r}; responses={responses!r}') from error
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, help='Required on macOS; Windows defaults to rebuilt Preview 13 toolchain')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), cases=[], commands=[])

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
        if not windows and platform.system() != 'Darwin':
            raise ValueError('Requires macOS ARM64 or Windows x64')
        report['revision'] = run(['git', 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        inputs = [Path(__file__).resolve(), ROOT / 'scripts/validate-native-http-project.py',
                  ROOT / 'scripts/verify-native-http-json.py', ROOT / 'docs/experiments/retained-http/Server.rvn',
                  ROOT / 'docs/experiments/retained-http/Native.rvnproj', ROOT / 'scripts/build-native-project.py',
                  ROOT / 'benchmarks/native-web/http-session-host.c', ROOT / 'docs/experiments/aot-console/native-session.c']
        report['inputs'] = {p.relative_to(ROOT).as_posix(): sha(p) for p in inputs}
        if args.bundle:
            bundle = args.bundle.resolve()
        elif windows:
            prereq = out.parent / 'windows-project-prerequisites'
            prereq.mkdir(exist_ok=True)
            archive = prereq / common.windows_project.ARCHIVE
            if not archive.exists():
                urllib.request.urlretrieve(common.windows_project.URL, archive)
            if sha(archive) != common.windows_project.BUNDLE_SHA:
                raise ValueError('Bundle archive hash mismatch')
            bundle = prereq / 'neoclr-native-poc'
            if not bundle.exists():
                with tarfile.open(archive) as source:
                    source.extractall(prereq, filter='data')
            bundle = common.rebuild_libraries(run, bundle, out / 'development toolchain')
        else:
            raise ValueError('--bundle is required on macOS')
        run(['cargo', 'build', '--locked', '--manifest-path', ROOT / 'tools/aot-poc/Cargo.toml'], 'aot-build')
        aot = ROOT / ('tools/aot-poc/target/debug/neoclr-aot-poc' + ('.exe' if windows else ''))
        report['aotSha256'] = sha(aot)
        interpreter = common.build_interpreter(run, windows)
        report['interpreterSha256'] = sha(interpreter)
        project = out / 'project with spaces'
        project.mkdir()
        for name in ('Server.rvn', 'Native.rvnproj'):
            shutil.copy2(ROOT / 'docs/experiments/retained-http' / name, project / name)
        destination = out / 'native output'
        run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http' if windows else 'http',
             '--bootstrap-root', 'Bootstrap', '--project', project / 'Native.rvnproj', '--bundle', bundle,
             '--aot', aot, '--output', destination], 'project-build')
        build = json.loads((destination / 'build.json').read_text())
        if not build['passed']:
            raise ValueError('Native build failed')
        isolated = out / 'executable only'
        isolated.mkdir()
        executable = isolated / ('app.exe' if windows else 'app')
        shutil.copy2(destination / executable.name, executable)
        lib = bundle / 'lib'
        catalog = json.loads((lib / 'bundle.json').read_text())
        vm = [interpreter, 'run', destination / 'app.dll',
              '--system', lib / catalog['runtimeSeed'],
              *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))],
              '--object-root', lib / 'System.Runtime.dll', '--instructions', '100000000']
        listing = out / 'application.metadata.txt'
        run([vm[0], 'disassemble', destination / 'app.dll', listing], 'metadata')
        module, _ = json.JSONDecoder().raw_decode(listing.read_text().split('.module ', 1)[1])
        report['interpreterEntry'] = module['entry']
        env = {key: value for key, value in os.environ.items() if key.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
        for name, targets, fragmented in [('three-requests', ['/count'] * 3, False),
                                           ('fragmented', ['/count'] * 3, True),
                                           ('fault-after-success', ['/count', '/fault'], False)]:
            row = dict(name=name)
            report['cases'].append(row)
            row['interpreter'] = serve(vm, targets, fragmented, ROOT)
            row['native'] = serve([executable], targets, fragmented, isolated, env)
            expected = [(b'HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n'
                         b'Content-Type: text/plain; charset=utf-8\r\n\r\nRequest ' + str(n).encode()).hex()
                        for n in range(1, 4)]
            reference = dict(row['interpreter'])
            if name == 'fault-after-success':
                # The same guest frames fault under different outer lifetimes:
                # interpreter async Main versus a quiescent native host dispatch.
                tail = ('   at System.Runtime.CompilerServices.RuntimeServices.DrainEntryTasks [instruction 0]\n'
                        + '   at ' + module['entry'] + ' [instruction 1]\n')
                if not reference['stderr'].endswith(tail):
                    raise ValueError('Unexpected interpreter fault boundary')
                reference['stderr'] = reference['stderr'][:-len(tail)]
                row['expectedInterpreterBoundary'] = tail
            if row['native'] != reference:
                raise ValueError(name + ': native/interpreter mismatch')
            result = row['native']
            if name == 'fault-after-success':
                if result['exitCode'] != 1 or result['stdout'] != 'Completed 1\n' or result['responses'] != [expected[0], ''] or 'Retained HTTP handler fault' not in result['stderr']:
                    raise ValueError('Unexpected terminal fault behavior')
            elif result != dict(exitCode=0, stdout='Completed 1\nCompleted 2\nCompleted 3\n', stderr='', responses=expected):
                raise ValueError(name + ': unexpected response/count')
            row['passed'] = True
        if any(sha(Path(p)) != h for p, h in build['inputs'].items()) or any(sha(ROOT / p) != h for p, h in report['inputs'].items()):
            raise ValueError('Inputs changed during validation')
        if sha(interpreter) != report['interpreterSha256'] or sha(aot) != report['aotSha256'] or sorted(p.name for p in isolated.iterdir()) != [executable.name]:
            raise ValueError('AOT or standalone directory changed during validation')
        report.update(passed=True, dependencies=build['dependencies'], standalone=True)
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Retained HTTP: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
