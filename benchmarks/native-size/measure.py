#!/usr/bin/env python3
"""Measure verified AOT outputs and catalogued development installation payloads."""
import argparse
from datetime import date
import hashlib
import json
from pathlib import Path
import platform
import subprocess


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest() if hasattr(hashlib, 'file_digest') else hashlib.sha256(stream.read()).hexdigest()


def artifact(path):
    return dict(bytes=path.stat().st_size, sha256=digest(path))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--backend-revision', required=True, help='Source revision used to build the measured backend')
    parser.add_argument('--compiler-revision', required=True, help='Source revision of the bundled Raven compiler')
    parser.add_argument('--sdk-archive', type=Path, required=True)
    parser.add_argument('--bundle-archive', type=Path, required=True)
    parser.add_argument('--case', action='append', required=True, help='Name=successful native build directory')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    root = args.bundle.resolve()
    manifest = json.loads((root / 'manifest.json').read_text())
    entries = []
    for relative, expected in manifest['files'].items():
        path = (root / relative).resolve()
        if root not in path.parents or digest(path) != expected:
            raise ValueError('Bundle catalogue mismatch: ' + relative)
        entries.append((relative, path.stat().st_size))
    entries.append(('manifest.json', (root / 'manifest.json').stat().st_size))
    def total(prefix=''):
        sizes = [size for path, size in entries if path.startswith(prefix)]
        return dict(bytes=sum(sizes), files=len(sizes))
    report = dict(schemaVersion=1, measuredOn=date.today().isoformat(), host=platform.platform(),
                  backendRevision=args.backend_revision, compilerRevision=args.compiler_revision,
                  method='Logical file bytes; no filesystem allocation rounding; catalogued bundle payload only.',
                  bundleManifest=artifact(root / 'manifest.json'),
                  installation=dict(ravenSdk=total('sdk/'), classLibraries=total('lib/'),
                                    jitToolchainBundle=total(),
                                    sdkArchive=artifact(args.sdk_archive),
                                    bundleArchive=artifact(args.bundle_archive)), cases=[])
    for entry in args.case:
        name, directory = entry.split('=', 1)
        folder = Path(directory)
        build = json.loads((folder / 'build.json').read_text())
        if not build['passed']:
            raise ValueError('A successful native build is required: ' + name)
        executable = 'app.exe' if (folder / 'app.exe').exists() else 'app'
        for filename in ('app.dll', executable):
            if digest(folder / filename) != build['artifacts'][filename]:
                raise ValueError('Native build output changed: ' + filename)
        result = subprocess.run([str((folder / executable).resolve())], capture_output=True, text=True, timeout=60, check=True)
        expected = {'hello': 'Hello, Café 🌍\n', 'json': 'JSON object mapping checks passed\n'}[name]
        if result.stdout != expected or result.stderr:
            raise ValueError('Unexpected application output: ' + name)
        dependencies = subprocess.check_output(['xcrun', 'otool', '-L', str(folder / executable)], text=True)
        report['cases'].append(dict(name=name, profile=build['profile'], assembly=artifact(folder / 'app.dll'),
                                   nativeExecutable=artifact(folder / executable),
                                   buildReport=artifact(folder / 'build.json'),
                                   toolchain=build['toolchain'], sourceInputs=build['inputs'],
                                   dependencies=dependencies.splitlines()[1:],
                                   machoSections=subprocess.check_output(['xcrun', 'size', '-m', str(folder / executable)], text=True),
                                   stdout=result.stdout,
                                   configuration='Default build-native-project.py release backend; clang -O2; no additional strip step'))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps(dict(installation=report['installation'], cases=[
        dict(name=c['name'], assembly=c['assembly']['bytes'], native=c['nativeExecutable']['bytes']) for c in report['cases']]), indent=2))


if __name__ == '__main__':
    main()
