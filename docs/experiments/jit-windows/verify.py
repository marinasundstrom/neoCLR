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
        run([sys.executable, ROOT/'docs/experiments/jit-http/verify.py', '--bundle', installed,
             '--windows-build', out/'native/build.json', '--default-cli', '--mode', 'jit',
             '--output', out/'http'], 'http-default-jit')
        run([sys.executable, ROOT/'scripts/test-runtime-library.py', '--bundle', installed,
             '--suite', 'collections', '--suite', 'json-dom', '--suite', 'encoding-streams',
             '--suite', 'index-range', '--suite', 'runner-contract', '--output', out/'framework'], 'framework-default-jit')
        report['passed'] = True
    finally:
        (out/'validation.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
