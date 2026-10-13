#!/usr/bin/env python3
"""Matched Windows JIT/AOT HTTP comparison; only root-probe.c differs."""
import argparse
import json
import os
from pathlib import Path
import platform
import runpy
import statistics
import subprocess
import sys
from xml.sax.saxutils import escape
import benchmark

ROOT = Path(__file__).resolve().parents[2]


def replace_option(command, prefix, value):
    positions = [i for i, arg in enumerate(command) if arg.lower().startswith(prefix.lower())]
    if len(positions) != 1:
        raise ValueError('Expected exactly one compiler option: ' + prefix)
    result = command[:]
    result[positions[0]] = prefix + str(value)
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--host-recipe', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--baseline', default='9cd74b374')
    p.add_argument('--rounds', type=int, default=5)
    a = p.parse_args()
    if platform.system() != 'Windows' or a.rounds < 5:
        p.error('Requires Windows and at least five alternating rounds')
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    bundle = a.bundle.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    host = json.loads(a.host_recipe.read_text())
    assert host['passed'] and manifest['jit']['default']
    observer = ROOT / 'docs/experiments/aot-console/root-probe.c'
    old = out / 'baseline-root-probe.c'
    old.write_bytes(subprocess.check_output(['git', 'show', a.baseline + ':docs/experiments/aot-console/root-probe.c'], cwd=ROOT))
    report = dict(passed=False, revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        baselineRevision=subprocess.check_output(['git', 'rev-parse', a.baseline], cwd=ROOT, text=True).strip(),
        platform=platform.platform(), machine=platform.machine(), processor=os.environ.get('PROCESSOR_IDENTIFIER'),
        rounds=a.rounds, commands=[], inputs={}, warmups=[], samples=[], summary={},
        scope='Windows x64 hosted runner. Same guest assemblies/AOT objects; only root-probe.c differs. HTTP/1.1 loopback, connection close, concurrency one. 32 validated requests/process, eight discarded; build/JIT startup excluded. No profiling during timings. Short local regression comparison, not saturation or cross-platform ranking.')

    def remember(path):
        path = Path(path).resolve()
        if path.is_file():
            digest = benchmark.sha(path)
            if str(path) in report['inputs'] and report['inputs'][str(path)] != digest:
                raise ValueError('Input changed: ' + str(path))
            report['inputs'][str(path)] = digest

    def run(command):
        command = list(map(str, command))
        print('BUILD', command[0], flush=True)
        result = subprocess.run(command, cwd=out, capture_output=True, text=True, timeout=600)
        report['commands'].append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    try:
        for name, digest in manifest['files'].items():
            if benchmark.sha(bundle / name) != digest:
                raise ValueError('Bundle drift: ' + name)
            remember(bundle / name)
        for path in [__file__, benchmark.__file__, benchmark.workbench.__file__, ROOT / 'benchmarks/framework/process.py',
                     bundle / 'manifest.json', a.host_recipe, observer, old, ROOT / 'scripts/build-native-project.py']:
            remember(path)
        for folder in ['aot-console', 'aot-fault-details', 'runtime-async', 'jit-managed']:
            for path in (ROOT / 'docs/experiments' / folder).glob('*.h'):
                remember(path)
        host_compile = next(c['command'] for c in host['commands'] if str(observer) in c['command'])
        host_link = next(c['command'] for c in host['commands'] if '/DLL' in c['command'])
        original_object = next(v[3:] for v in host_compile if v.startswith('/Fo'))
        hosts = {}
        for variant in ['baseline', 'candidate']:
            directory = out / variant
            directory.mkdir()
            obj = directory / 'root-probe.obj'
            cmd = [str(old) if v == str(observer) and variant == 'baseline' else v for v in host_compile]
            cmd = replace_option(cmd, '/Fo', obj)
            run(cmd)
            dll = directory / 'host.dll'
            cmd = [str(obj) if v == original_object else v for v in host_link]
            assert str(obj) in cmd
            cmd = replace_option(cmd, '/OUT:', dll)
            for value in cmd:
                remember(value)
                if value.startswith('/DEF:'): remember(value[5:])
            run(cmd)
            hosts[variant] = dll
            remember(dll)
        invocations = {}
        for case in ['plain', 'json']:
            project = out / (case + '-project')
            project.mkdir()
            source = ROOT / ('benchmarks/jit-http/Server.rvn' if case == 'plain' else 'benchmarks/http-server/JsonServer.rvn')
            (project / 'Main.rvn').write_bytes(source.read_bytes())
            if case == 'json':
                (project / 'Model.rvn').write_bytes((ROOT / 'benchmarks/http-server/Model.rvn').read_bytes())
            props = escape(str(bundle / 'lib/NeoCLR.ClassLibrary.props'), {'"': '&quot;'})
            (project / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="' + props + '"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>App</AssemblyName></PropertyGroup></Project>')
            for path in project.iterdir(): remember(path)
            roots = []
            if case == 'json':
                run(['dotnet', bundle / 'sdk/tools/rvnc/rvnc.dll', 'neoclr', '--project', project / 'App.rvnproj'])
                reflection = runpy.run_path(str(bundle / 'tools/run-native-sample.py'))['reflection_roots']
                roots = list(map(str, reflection(bundle / manifest['runtime'], project / 'bin/neoclr/App.dll', ['Person'], run)))
            native = out / (case + '-native')
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http',
                 '--project', project / 'App.rvnproj', '--bundle', bundle, '--aot', bundle / manifest['jit']['compiler'],
                 '--output', native, '--http-heap-bytes', str(manifest['jit']['heapBytes']),
                 '--byte-array-limit', str(manifest['jit']['byteArrayLimit']), '--command-timeout', '600', *roots])
            build = json.loads((native / 'build.json').read_text())
            assert build['passed']
            remember(native / 'build.json')
            link = next(c['command'] for c in build['commands'] if c['phase'] == 'host-compile-link')
            assert str(observer) in link
            for variant in ['baseline', 'candidate']:
                directory = out / variant / case
                directory.mkdir()
                app = directory / 'app.exe'
                cmd = [str(old) if v == str(observer) and variant == 'baseline' else v for v in link]
                cmd = replace_option(replace_option(cmd, '/Fe:', app), '/Fo', str(directory) + '/')
                for value in cmd: remember(value)
                run(cmd)
                invocations[(case, 'aot', variant)] = [str(app)]
                invocations[(case, 'jit', variant)] = [str(bundle / manifest['runtime']), 'run', str(native / 'app.dll'),
                    '--jit-host', str(hosts[variant]), *roots]
                for command in [invocations[(case, 'aot', variant)], invocations[(case, 'jit', variant)]]:
                    for value in command: remember(value)
        # All compilation is complete before warmups/timing. Never rebuild between pairs.
        for (case, mode, variant), command in invocations.items():
            report['warmups'].append(dict(case=case, mode=mode, variant=variant, **benchmark.server_sample(command, case)))
        for n in range(a.rounds):
            for case in ['plain', 'json']:
                for mode in ['aot', 'jit']:
                    for variant in (['baseline', 'candidate'] if n % 2 == 0 else ['candidate', 'baseline']):
                        report['samples'].append(dict(round=n, case=case, mode=mode, variant=variant,
                            **benchmark.server_sample(invocations[(case, mode, variant)], case)))
            print('ROUND', n + 1, flush=True)
        for case in ['plain', 'json']:
            for mode in ['aot', 'jit']:
                row = {}
                for variant in ['baseline', 'candidate']:
                    values = [r['requestsPerSecond'] for r in report['samples'] if (r['case'], r['mode'], r['variant']) == (case, mode, variant)]
                    row[variant] = dict(median=statistics.median(values), min=min(values), max=max(values))
                row['throughputRatio'] = row['candidate']['median'] / row['baseline']['median']
                report['summary'][case + '-' + mode] = row
        for path, digest in report['inputs'].items():
            if benchmark.sha(path) != digest: raise ValueError('Input changed: ' + path)
        report['passed'] = True
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report['summary'], indent=2))


if __name__ == '__main__':
    main()
