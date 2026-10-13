#!/usr/bin/env python3
"""Fresh managed HTTP objects, identical stress/paced hosts, separate GC diagnostics."""
import argparse
import hashlib
import json
import os
import runpy
from pathlib import Path
import platform
import socket
import statistics
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest


def sample(executable, workload, diagnostic=False):
    guest = Guest([str(executable)])
    times = []
    try:
        port, _ = guest.line()
        for index in range(32):
            start = time.perf_counter()
            with socket.create_connection(('127.0.0.1', int(port)), timeout=60) as peer:
                peer.sendall(b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                response = b''
                while True:
                    part = peer.recv(4096)
                    if not part:
                        break
                    response += part
                    if len(response) > 8192:
                        raise ValueError('Oversized response')
            header, body = response.split(b'\r\n\r\n', 1)
            lines = header.split(b'\r\n')
            fields = dict(line.lower().split(b': ', 1) for line in lines[1:])
            assert lines[0] == b'HTTP/1.1 200 OK' and fields[b'connection'] == b'close'
            assert int(fields[b'content-length']) == len(body)
            if workload == 'plain':
                assert body == 'Café 🌍'.encode()
            else:
                assert fields[b'content-type'] == b'application/json; charset=utf-8'
                assert json.loads(body) == dict(id=42, name='Café team', active=True)
            if index >= 8:
                times.append(time.perf_counter() - start)
        guest.line(b'DONE')
        stats = json.loads(guest.line()[0]) if diagnostic else None
        guest.finish()
        if stats:
            assert stats['allocations'] == stats['reclaimed']
        return dict(validatedRequests=32, timedRequests=24, requestSeconds=times,
                    requestsPerSecond=24 / sum(times), gc=stats)
    except Exception as error:
        # Let a terminal fault finish writing before reading its shared stderr file.
        try:
            guest.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass
        raise RuntimeError(f'{error}; exit={guest.process.poll()}; stdout={guest.output!r}; stderr={guest.errors()!r}') from error
    finally:
        guest.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--aot', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=ROOT / ('target/release/neoclr.exe' if platform.system() == 'Windows' else 'target/release/neoclr'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=3)
    args = parser.parse_args()
    if args.rounds < 3:
        parser.error('At least three measured pairs are required')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    windows = platform.system() == 'Windows'
    report = dict(passed=False, promotionEligible=False, platform=platform.platform(),
                  scope='Managed state-machine HTTP, native AOT host policy comparison; not default promotion or sustained/concurrent load.',
                  commands=[], inputs={}, workloads={})

    def run(command):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600, env={**os.environ, 'NeoClrBundleRoot': str(args.bundle.resolve())})
        report['commands'].append(dict(command=command, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stderr or result.stdout)
        return result.stdout

    try:
        report['revision'] = run(['git', 'rev-parse', 'HEAD']).strip()
        for path in [Path(__file__), ROOT / 'benchmarks/framework/process.py', ROOT / 'benchmarks/jit-http/Server.rvn', ROOT / 'benchmarks/http-server/JsonServer.rvn', ROOT / 'benchmarks/http-server/Model.rvn', ROOT / 'scripts/run-native-sample.py', args.runtime.resolve()]:
            report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
        wrapper = out / 'diagnostic-host.c'
        wrapper.write_text('''#define main measured_main
#define wmain measured_wmain
#include "''' + str(ROOT / 'benchmarks/native-web/http-host.c').replace('\\', '/') + '''"
#undef main
#undef wmain
int NEOCLR_PROCESS_MAIN(int argc, neoclr_process_char **argv) {
#ifdef _WIN32
    int code=measured_wmain(argc,argv);
#else
    int code=measured_main(argc,argv);
#endif
    neoclr_gc_statistics s=neoclr_gc_statistics_v1();
    neoclr_gc_profile p=neoclr_gc_profile_v1();
    printf("{\\"allocations\\":%llu,\\"reclaimed\\":%llu,\\"collections\\":%llu,\\"peakArenaBytes\\":%llu}\\n",
        (unsigned long long)s.allocations,(unsigned long long)s.reclaimed_allocations,
        (unsigned long long)s.collections,(unsigned long long)p.peak_arena_used);
    return code ? code : s.allocations != s.reclaimed_allocations;
}
''')
        report['inputs'][str(wrapper)] = hashlib.sha256(wrapper.read_bytes()).hexdigest()
        for workload in ['plain', 'json']:
            project = out / workload / 'project'
            project.mkdir(parents=True)
            source = ROOT / ('benchmarks/jit-http/Server.rvn' if workload == 'plain' else 'benchmarks/http-server/JsonServer.rvn')
            (project / 'Main.rvn').write_bytes(source.read_bytes())
            if workload == 'json':
                (project / 'Model.rvn').write_bytes((ROOT / 'benchmarks/http-server/Model.rvn').read_bytes())
            (project / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>App</AssemblyName><RavenNeoClrRuntimeAsyncHelper></RavenNeoClrRuntimeAsyncHelper></PropertyGroup></Project>')
            roots = []
            if workload == 'json':
                run(['dotnet', args.bundle.resolve() / 'sdk/tools/rvnc/rvnc.dll', 'neoclr', '--project', project / 'App.rvnproj'])
                prepare = runpy.run_path(str(ROOT / 'scripts/run-native-sample.py'))['reflection_roots']
                roots = prepare(args.runtime.resolve(), project / 'bin/neoclr/App.dll', ['Person'], run)
            native = project.parent / 'native'
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http' if windows else 'http',
                 '--project', project / 'App.rvnproj', '--bundle', args.bundle.resolve(), '--aot', args.aot.resolve(), '--output', native, *roots])
            build = json.loads((native / 'build.json').read_text())
            assert build['passed']
            report['inputs'].update(build['inputs'])
            base = next(row['command'] for row in build['commands'] if row.get('phase') == 'host-compile-link')
            binaries = {}
            for mode in ['stress', 'paced']:
                for diagnostic in [False, True]:
                    name = mode + ('-diagnostic' if diagnostic else '')
                    exe = project.parent / (name + ('.exe' if windows else ''))
                    cmd = base.copy()
                    define = '/D' if windows else '-D'
                    if windows:
                        cmd = [('/Fe:' + str(exe)) if arg.startswith('/Fe:') else arg for arg in cmd]
                    else:
                        cmd[cmd.index('-o') + 1] = str(exe)
                    if mode == 'paced':
                        cmd.insert(1, define + 'NEOCLR_GC_PACED')
                    if diagnostic:
                        cmd.insert(1, define + 'NEOCLR_GC_PROFILE')
                        cmd = [str(wrapper) if Path(arg) == ROOT / 'benchmarks/native-web/http-host.c' else arg for arg in cmd]
                    run(cmd)
                    binaries[name] = exe
            rows = report['workloads'][workload] = dict(runs=[], diagnostics=[], summary={})
            for round_index in range(-1, args.rounds):
                for mode in ['stress', 'paced'][::1 if round_index % 2 == 0 else -1]:
                    rows['runs'].append(dict(round=round_index, mode=mode, **sample(binaries[mode], workload)))
                    print(workload, round_index, mode, flush=True)
            for mode in ['stress', 'paced']:
                rows['diagnostics'].append(dict(mode=mode, **sample(binaries[mode + '-diagnostic'], workload, True)))
            medians = {mode: statistics.median(row['requestsPerSecond'] for row in rows['runs'] if row['round'] >= 0 and row['mode'] == mode) for mode in ['stress', 'paced']}
            rows['summary'] = dict(requestsPerSecond=medians, speedup=medians['paced']/medians['stress'])
        for path, digest in report['inputs'].items():
            assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
