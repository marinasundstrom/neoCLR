#!/usr/bin/env python3
"""Compare native and interpreted host-culture snapshots on the same machine."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--aot', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--existing-build', type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    bundle = args.bundle.resolve()
    build = args.existing_build.resolve() if args.existing_build else out / 'build'
    report = dict(passed=False, platform=platform.platform(), commands=[])

    def run(command):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600)
        report['commands'].append(dict(command=command, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stderr)
        return result.stdout

    try:
        if not args.existing_build:
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
                 'windows-console' if os.name == 'nt' else 'console', '--project', HERE / 'Culture.rvnproj',
                 '--bundle', bundle, '--aot', args.aot.resolve(), '--output', build])
        built = json.loads((build / 'build.json').read_text())
        if not built['passed'] or Path(built['project']).resolve() != HERE / 'Culture.rvnproj':
            raise ValueError('Expected a successful culture-probe build')
        for path, digest in built['inputs'].items():
            if sha(Path(path)) != digest:
                raise ValueError('Build input changed: ' + path)
        for name, digest in built['artifacts'].items():
            if sha(build / name) != digest:
                raise ValueError('Build artifact changed: ' + name)
        lib = bundle / 'lib'
        catalog = json.loads((lib / 'bundle.json').read_text())
        inputs = [lib / catalog['runtimeSeed'], lib / 'bundle.json',
                  *[lib / (name + '.dll') for name in catalog['assemblyNames']],
                  args.runtime.resolve(), build / 'build.json', HERE / 'Culture.rvn', Path(__file__).resolve()]
        report['inputs'] = {str(p): sha(p) for p in inputs}
        native = run([build / ('app.exe' if os.name == 'nt' else 'app')])
        interpreted = run([args.runtime.resolve(), 'run', build / 'app.dll', '--system', lib / catalog['runtimeSeed'],
                           *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))],
                           '--object-root', lib / 'System.Runtime.dll'])
        if native != interpreted or len(native.splitlines()) != 3:
            raise AssertionError('Native and interpreted culture snapshots differ')
        reported, selected, current = native.splitlines()
        if selected != current or current not in ('', 'sv-SE', 'he-IL'):
            raise AssertionError('Unexpected bounded culture selection')
        if any(sha(Path(p)) != digest for p, digest in report['inputs'].items()):
            raise ValueError('Inputs changed during execution')
        report.update(passed=True, preferredCulture=reported, selectedCulture=selected)
        print('PASS native/interpreter host-culture parity')
    finally:
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
