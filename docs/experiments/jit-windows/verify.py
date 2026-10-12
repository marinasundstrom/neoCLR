#!/usr/bin/env python3
"""Prepare the pinned Raven bundle, build a Windows host recipe and qualify JIT HTTP."""
import argparse
import importlib.util
import json
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[3]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
        raise ValueError('This gate requires Windows x64 with MSVC')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    prerequisite = load('windows_prerequisite', ROOT/'scripts/validate-windows-project.py')
    http = load('http_prerequisite', ROOT/'scripts/validate-native-http-project.py')
    report = dict(passed=False, commands=[])

    def run(command, name):
        print('START '+name, flush=True)
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=1200)
        (out/(name+'.stdout.log')).write_bytes(result.stdout)
        (out/(name+'.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(name+' failed; see retained logs')
        print('PASS '+name, flush=True)
        return result

    try:
        memory = ROOT/'tools/native'
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/O2', '/MT',
             '/Fo'+str(out)+'/', '/Fe:'+str(out/'guarded-heap.exe'),
             memory/'windows-host-memory.c', memory/'windows-host-memory-test.c'], 'guarded-heap-build')
        result = run([out/'guarded-heap.exe'], 'guarded-heap')
        if result.stdout != b'Windows guarded heap: 18 allocation lifecycles passed\r\n' or result.stderr:
            raise RuntimeError('Guarded heap acceptance differs')
        archive = out/prerequisite.ARCHIVE
        urllib.request.urlretrieve(prerequisite.URL, archive)
        if prerequisite.sha(archive) != prerequisite.BUNDLE_SHA:
            raise ValueError('Pinned bundle hash mismatch')
        with tarfile.open(archive) as source:
            source.extractall(out, filter='data')
        bundle = http.rebuild_libraries(run, out/'neoclr-native-poc', out/'development')
        run(['cargo', 'build', '--locked', '--release', '--manifest-path', ROOT/'tools/aot-poc/Cargo.toml'], 'jit-build')
        run(['cargo', 'build', '--locked', '--release', '--bin', 'neoclr'], 'runtime-build')
        run(['cargo', 'test', '--locked', '--release', '--manifest-path', ROOT/'tools/aot-poc/Cargo.toml',
             '--test', 'scalar', 'jit_'], 'scalar-jit')
        run(['cargo', 'test', '--locked', '--release', '--test', 'cli',
             'cli_execution_selection'], 'cli-selection')
        jit = ROOT/'tools/aot-poc/target/release/neoclr-aot-poc.exe'
        project = out/'server-source'
        project.mkdir()
        (project/'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>JitHttp</AssemblyName></PropertyGroup></Project>')
        (project/'Main.rvn').write_text((ROOT/'docs/experiments/http-server/Server.rvn').read_text(encoding='utf-8'), encoding='utf-8')
        run([sys.executable, ROOT/'scripts/build-native-project.py', '--profile', 'windows-http',
             '--project', project/'App.rvnproj', '--bundle', bundle, '--aot', jit,
             '--output', out/'native'], 'native-recipe')
        run([sys.executable, ROOT/'scripts/prepare-jit-bundle.py', '--bundle', bundle,
             '--runtime', ROOT/'target/release/neoclr.exe', '--jit-compiler', jit,
             '--host-recipe', out/'native/build.json', '--output', out/'jit-install'], 'prepare-default-jit')
        installed = out/'jit-install/bundle'
        run([sys.executable, ROOT/'docs/experiments/native-cryptography/validate-bundle.py',
             '--bundle', installed, '--output', out/'cryptography'], 'cryptography-default-jit')
        run([sys.executable, ROOT/'docs/experiments/native-cryptography/validate-project.py',
             '--bundle', installed, '--aot', jit, '--output', out/'cryptography-project'], 'cryptography-project')
        run([sys.executable, ROOT/'docs/experiments/shared-tls/validate-bundle.py',
             '--bundle', installed, '--output', out/'tls-ownership'], 'tls-default-jit')
        run([sys.executable, ROOT/'docs/experiments/shared-tls/validate-project.py',
             '--bundle', installed, '--aot', jit, '--output', out/'tls-project'], 'tls-project')
        run([sys.executable, ROOT/'docs/experiments/shared-tls/validate-streams.py',
             '--bundle', installed, '--aot', jit, '--output', out/'tls-streams'], 'tls-streams')
        run([sys.executable, ROOT/'docs/experiments/shared-tls/validate-streams.py',
             '--bundle', installed, '--aot', jit, '--output', out/'tls-http-failures',
             '--http-failures'], 'tls-http-failures')
        run([sys.executable, ROOT/'docs/experiments/shared-tls/validate-interop.py',
             '--bundle', installed, '--aot', jit, '--output', out/'tls-http-interop'], 'tls-http-interop')
        run([sys.executable, ROOT/'scripts/test-runtime-library.py', '--bundle', installed,
             '--suite', 'cryptography', '--mode', 'interpreter', '--mode', 'jit', '--mode', 'native',
             '--output', out/'public-cryptography'], 'public-cryptography')
        run([sys.executable, ROOT/'docs/experiments/jit-http/verify.py', '--bundle', installed,
             '--windows-build', out/'native/build.json', '--default-cli', '--mode', 'jit',
             '--output', out/'http'], 'http-default-jit')
        run([sys.executable, ROOT/'scripts/test-runtime-library.py', '--bundle', installed,
             '--suite', 'collections', '--suite', 'json-dom', '--suite', 'encoding-streams',
             '--suite', 'index-range', '--suite', 'zoned-time-parsing', '--suite', 'runner-contract', '--output', out/'framework'], 'framework-default-jit')
        libraries = json.loads((installed/'lib/bundle.json').read_text())
        interpreted_context = ['--execution', 'interpreter', '--instructions', '100000000',
                               '--system', installed/'lib'/libraries['runtimeSeed'],
                               '--object-root', installed/'lib'/(libraries['objectAssembly']+'.dll')]
        for assembly_name in libraries['assemblyNames']:
            interpreted_context += ['--module', installed/'lib'/(assembly_name+'.dll')]
        for name, fixture, expected in [
            ('async-use', 'RuntimeUse.rvn', 'PASS runtime async scoped cleanup'),
            ('generic-async-use', 'RuntimeGenericUse.rvn', 'PASS generic async methods owners and scoped cleanup'),
        ]:
            source = out/(name+'-source')
            source.mkdir()
            (source/'App.rvnproj').write_text((project/'App.rvnproj').read_text().replace('JitHttp', 'AsyncContract'))
            (source/'Main.rvn').write_text((ROOT/'docs/experiments/runtime-async'/fixture).read_text(encoding='utf-8'), encoding='utf-8')
            native = out/name
            run([sys.executable, ROOT/'scripts/build-native-project.py', '--profile', 'windows-http',
                 '--project', source/'App.rvnproj', '--bundle', installed, '--aot', jit,
                 '--output', native], name+'-build')
            for mode, command in [
                ('jit', [installed/'bin/neoclr.exe', 'run', native/'app.dll']),
                ('aot', [native/'app.exe']),
                ('interpreter', [installed/'bin/neoclr.exe', 'run', native/'app.dll', *interpreted_context]),
            ]:
                result = run(command, name+'-'+mode)
                if result.stdout.decode('utf-8').replace('\r\n', '\n') != expected+'\n' or result.stderr:
                    raise RuntimeError(name+' '+mode+' outcome differs')
        report['passed'] = True
    finally:
        (out/'validation.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
