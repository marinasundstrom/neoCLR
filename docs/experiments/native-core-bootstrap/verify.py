#!/usr/bin/env python3
"""Verify the bounded native-only core consumer; not a full class-library bootstrap."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('raven', 'runtime', 'aot', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists():
        raise FileExistsError(output)
    output.mkdir(parents=True)
    artifacts = output / 'artifacts'
    report = {'scope': 'Native-only core integer consumer; explicit fixture Object and empty runtime seed.', 'commands': []}

    def run(command, expected=0, include_stderr=False):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=180)
        report['commands'].append({'command': command, 'exitCode': result.returncode,
                                   'stdout': result.stdout, 'stderr': result.stderr})
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode != expected:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout + (result.stderr if include_stderr else '')

    metadata = ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj'
    run(['dotnet', 'run', '--project', HERE / 'Probe.csproj', '-p:RavenRoot=' + str(args.raven.resolve()),
         '-p:NeoClrMetadataProject=' + str(metadata), '-p:WarningLevel=0', '--', artifacts])
    runtime = args.runtime.resolve()
    aot = args.aot.resolve()
    core = artifacts / 'NativeCore.dll'
    consumer = artifacts / 'Consumer.dll'
    seed = artifacts / 'System.neox'
    run([runtime, 'assemble', HERE / 'System.neoil', seed, '--format', 'neox'])
    dependencies = ['--module', core, '--system', seed, '--object-root', core]
    interpreted = run([runtime, 'run', consumer, *dependencies, '--show-result'], expected=42, include_stderr=True)
    if interpreted.strip() != '=> Int32(42)':
        raise AssertionError(interpreted)
    selection = run([aot, '--closed-world', consumer, '@entry', artifacts / 'consumer.o', *dependencies])
    (output / 'selection.json').write_text(selection)
    run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', HERE / 'host.c', artifacts / 'consumer.o', '-o', artifacts / 'consumer'])
    native = run([artifacts / 'consumer'])
    if native.strip() != '42':
        raise AssertionError(native)
    linked = run(['otool', '-L', artifacts / 'consumer'])
    dependencies_found = [line.strip().split(' (', 1)[0] for line in linked.splitlines()[1:]]
    if dependencies_found != ['/usr/lib/libSystem.B.dylib']:
        raise AssertionError(dependencies_found)
    report['result'] = {'interpreter': 42, 'native': 42, 'nativeLibraries': dependencies_found}
    report['revisions'] = {name: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
                           for name, path in [('neoclr', ROOT), ('raven', args.raven)]}
    inputs = [runtime, aot, *[p for p in HERE.iterdir() if p.suffix in ('.cs', '.csproj', '.rvn', '.neoil', '.c', '.py')]]
    inputs += list((HERE / 'bin/Debug/net10.0').glob('*.dll'))
    inputs += [core, consumer, seed, artifacts / 'consumer']
    report['sha256'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS native-only core consumer: interpreter=42, native=42, libSystem only')


if __name__ == '__main__':
    main()
