#!/usr/bin/env python3
"""Build and run the Raven runtime-library suite and runner contract in both modes."""
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
TESTS = ROOT / 'runtime/raven/tests'
COLLECTION_NAMES = [
    'ArrayList copy has independent storage',
    'ArrayQueue preserves FIFO',
    'ArrayStack preserves LIFO',
    'HashSet applies its comparer',
    'Any does not read Current',
    'Predicate Any stops at its first match',
    'For break disposes its iterator',
]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True, help='Matching native development bundle')
    parser.add_argument('--output', type=Path, required=True, help='Fresh build/evidence directory')
    parser.add_argument('--aot', type=Path, default=ROOT / 'tools/aot-poc/target/debug' / ('neoclr-aot-poc.exe' if os.name == 'nt' else 'neoclr-aot-poc'))
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    inputs = [Path(__file__).resolve(), ROOT / 'scripts/build-native-project.py',
              *sorted(TESTS.rglob('*.rvn')), *sorted(TESTS.rglob('*.rvnproj'))]
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  inputs={str(p.relative_to(ROOT)): sha(p) for p in inputs},
                  bundleManifestSha256=sha(bundle / 'manifest.json'),
                  aotSha256=sha(args.aot.resolve()), interpreterSha256=sha(args.runtime.resolve()))
    expected = [
        ('collections', 0, ''.join('PASS ' + n + '\n' for n in COLLECTION_NAMES) + 'Tests: 7, passed: 7, failed: 0, skipped: 0\n'),
        ('runner-contract', 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nSKIP intentional skip: contract probe\nTests: 4, passed: 2, failed: 1, skipped: 1\n'),
    ]
    try:
        for name, exit_code, stdout in expected:
            build = out / (name + '-build')
            command = [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
                       'windows-console' if os.name == 'nt' else 'console', '--project', TESTS / name / 'Tests.rvnproj',
                       '--bundle', bundle, '--aot', args.aot.resolve(), '--output', build]
            result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=600)
            (out / (name + '-build.stdout.log')).write_bytes(result.stdout)
            (out / (name + '-build.stderr.log')).write_bytes(result.stderr)
            if result.returncode:
                raise RuntimeError(name + ' build failed; see retained logs')
            isolated = out / (name + '-isolated')
            isolated.mkdir()
            exe = isolated / ('app.exe' if os.name == 'nt' else 'app')
            shutil.copy2(build / exe.name, exe)
            lib = bundle / 'lib'
            catalog = json.loads((lib / 'bundle.json').read_text())
            interpreter = [args.runtime.resolve(), 'run', build / 'app.dll', '--system', lib / catalog['runtimeSeed'],
                           *[arg for a in catalog['assemblyNames'] for arg in ('--module', lib / (a + '.dll'))],
                           '--object-root', lib / 'System.Runtime.dll', '--instructions', '100000000']
            env = {k: v for k, v in os.environ.items() if k.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
            for mode, command, cwd, environment in [('native', [exe], isolated, env), ('interpreter', interpreter, ROOT, None)]:
                result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=180)
                record = dict(suite=name, mode=mode, exitCode=result.returncode,
                              stdout=result.stdout.decode('utf-8').replace('\r\n', '\n'),
                              stderr=result.stderr.decode('utf-8').replace('\r\n', '\n'))
                report['cases'].append(record)
                print(name + ' (' + mode + '):\n' + record['stdout'], end='', flush=True)
                if (record['exitCode'], record['stdout'], record['stderr']) != (exit_code, stdout, ''):
                    raise RuntimeError(name + ' ' + mode + ' did not match the runner contract')
        for name, digest in report['inputs'].items():
            if sha(ROOT / name) != digest:
                raise RuntimeError('Input changed during validation: ' + name)
        if sha(args.aot.resolve()) != report['aotSha256'] or sha(args.runtime.resolve()) != report['interpreterSha256']:
            raise RuntimeError('Execution tool changed during validation')
        report['passed'] = True
    finally:
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Runtime library tests: PASS')


if __name__ == '__main__':
    main()
