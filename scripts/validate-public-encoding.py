#!/usr/bin/env python3
"""Compile the public UTF-8 sample and reject access to its internal implementation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bundle, output = args.bundle.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
    sample = ROOT / 'docs/experiments/raven-target/samples/library-utf8.rvn'
    expected = sample.with_suffix('.expected.txt')
    report = dict(passed=False, platform=platform.platform(), commands=[], inputs={
        str(p): sha(p) for p in (Path(__file__), sample, expected, compiler,
                                args.runtime.resolve(), bundle / 'manifest.json')})
    environment = dict(os.environ, NeoClrBundleRoot=str(bundle))

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=environment,
                                capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        return result

    try:
        for name, source in [('public', sample.read_text()),
                             ('internal', 'import System.Text.*\nfunc Main() {\n    _ = Utf8.Encode("private")\n}\n')]:
            work = output / name
            work.mkdir()
            (work / 'Main.rvn').write_text(source)
            project = work / 'App.rvnproj'
            project.write_text('''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
    <AssemblyName>EncodingSurfaceConsumer</AssemblyName>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>
''')
            result = run(['dotnet', compiler, 'neoclr', '--project', project])
            images = list(work.rglob('*.dll'))
            if name == 'internal':
                # Older native importers omit internal types entirely; current
                # importers retain them for an explicit visibility diagnostic.
                diagnostics = result.stdout + result.stderr
                denied = 'RAV0500' in diagnostics or 'RAV0103' in diagnostics
                if result.returncode == 0 or images or not denied:
                    raise RuntimeError('Internal Utf8 must report inaccessible and emit no image')
                continue
            if result.returncode or len(images) != 1:
                raise RuntimeError('Public sample did not produce exactly one image')
            lib = bundle / 'lib'
            catalog = json.loads((lib / 'bundle.json').read_text())
            command = [args.runtime.resolve(), 'run', images[0], '--system', lib / catalog['runtimeSeed'],
                       *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))],
                       '--object-root', lib / 'System.Runtime.dll', '--instructions', '100000000']
            result = run(command)
            if result.returncode or result.stderr or result.stdout != expected.read_text():
                raise RuntimeError('Public sample output differs from its published fixture')
        report['passed'] = True
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS public UTF-8 sample and inaccessible internal helper')


if __name__ == '__main__':
    main()
