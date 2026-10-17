#!/usr/bin/env python3
"""Run signed range boundary cases through interpreter, JIT and AOT in a bundle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bundle, output = args.bundle.resolve(), args.output.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    for name, digest in manifest['files'].items():
        if hashlib.sha256((bundle / name).read_bytes()).hexdigest() != digest:
            raise ValueError('Bundle drift: ' + name)
    output.mkdir(parents=True, exist_ok=False)
    project = output / 'project'
    project.mkdir()
    source = ROOT / 'docs/validation/range-iteration/Main.rvn'
    shutil.copy2(source, project / 'Main.rvn')
    (project / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>RangeBoundaries</AssemblyName></PropertyGroup></Project>')
    report = dict(passed=False, platform=platform.platform(), commands=[], inputs={})

    def run(command, expected_output=None):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600)
        report['commands'].append(dict(command=command, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        if result.returncode or (expected_output is not None and result.stdout != expected_output):
            raise RuntimeError(result.stderr or result.stdout)

    try:
        windows = platform.system() == 'Windows'
        native = output / 'native'
        run([sys.executable, ROOT / 'scripts/build-native-project.py', '--bundle', bundle,
             '--project', project / 'App.rvnproj', '--output', native,
             '--aot', bundle / manifest['jit']['compiler'],
             '--profile', 'windows-console' if windows else 'console', '--command-timeout', '600'])
        report['inputs'].update(json.loads((native / 'build.json').read_text())['inputs'])
        report['inputs'].update({str(bundle / name): digest for name, digest in manifest['files'].items()})
        for path in [Path(__file__).resolve(), source, bundle / 'manifest.json', native / 'app.dll', native / ('app.exe' if windows else 'app')]:
            report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
        runtime = bundle / manifest['runtime']
        assembly = native / 'app.dll'
        context = ['--system', bundle / manifest['runtimeSeed'], '--object-root', bundle / manifest['objectRoot']]
        for module in manifest['modules']:
            context += ['--module', bundle / module]
        commands = [[runtime, 'run', assembly, '--execution', 'interpreter', '--instructions', '1000000', *context],
                    [runtime, 'run', assembly], [native / ('app.exe' if windows else 'app')]]
        for command in commands:
            run(command, 'PASS signed range boundaries\n')
        for name, digest in report['inputs'].items():
            if hashlib.sha256(Path(name).read_bytes()).hexdigest() != digest:
                raise ValueError('Validation input changed: ' + name)
        report['passed'] = True
    finally:
        (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(output / 'results.json')


if __name__ == '__main__':
    main()
