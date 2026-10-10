#!/usr/bin/env python3
"""Build and run the Raven runtime-library suite and runner contract in both modes."""
import argparse
import hashlib
import importlib.util
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
    'Any does not read Current',
    'For break disposes its iterator',
    'ArrayList copy has independent storage',
    'Predicate Any stops at its first match',
    'ArrayQueue preserves FIFO',
    'HashSet applies its comparer',
    'ArrayStack preserves LIFO',
]
DISCOVERY_SPEC = importlib.util.spec_from_file_location('test_discovery', ROOT / 'scripts/discover-runtime-tests.py')
DISCOVERY = importlib.util.module_from_spec(DISCOVERY_SPEC)
DISCOVERY_SPEC.loader.exec_module(DISCOVERY)


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
    inputs = [Path(__file__).resolve(), ROOT / 'scripts/build-native-project.py', ROOT / 'scripts/discover-runtime-tests.py', ROOT / 'scripts/validate-test-discovery.py',
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.cs')),
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.csproj')),
              *sorted(TESTS.rglob('*.rvn')), *sorted(TESTS.rglob('*.rvnproj'))]
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  inputs={str(p.relative_to(ROOT)): sha(p) for p in inputs},
                  bundleManifestSha256=sha(bundle / 'manifest.json'),
                  aotSha256=sha(args.aot.resolve()), interpreterSha256=sha(args.runtime.resolve()))
    validation_spec = importlib.util.spec_from_file_location('discovery_validation', ROOT / 'scripts/validate-test-discovery.py')
    validation = importlib.util.module_from_spec(validation_spec)
    validation_spec.loader.exec_module(validation)
    expected = [
        ('collections', 0, ''.join('PASS ' + n + '\n' for n in COLLECTION_NAMES) + 'Tests: 7, passed: 7, failed: 0, skipped: 0\n'),
        ('discovery-contract', 1, 'PASS first discovered test\nFAIL discovered failure: Expected 1, actual 2\nPASS after discovered failure\nPASS NeoClr.DiscoveryTests.DWithoutDescription\nPASS manually registered companion\nTests: 5, passed: 4, failed: 1, skipped: 0\n'),
        ('runner-contract', 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nSKIP intentional skip: contract probe\nTests: 4, passed: 2, failed: 1, skipped: 1\n'),
    ]
    try:
        validation.validate(bundle, out / 'discovery-signatures')
        for name, exit_code, stdout in expected:
            build = out / (name + '-build')
            environment = dict(os.environ)
            if name in ('collections', 'discovery-contract'):
                registry = DISCOVERY.discover(TESTS / name / 'Tests.rvnproj', bundle, out / (name + '-discovery'))
                environment['NeoClrTestRegistry'] = str(registry)
            command = [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
                       'windows-console' if os.name == 'nt' else 'console', '--project', TESTS / name / 'Tests.rvnproj',
                       '--bundle', bundle, '--aot', args.aot.resolve(), '--output', build]
            result = subprocess.run(list(map(str, command)), cwd=ROOT, env=environment, capture_output=True, timeout=600)
            (out / (name + '-build.stdout.log')).write_bytes(result.stdout)
            (out / (name + '-build.stderr.log')).write_bytes(result.stderr)
            if result.returncode:
                raise RuntimeError(name + ' build failed; see retained logs')
            if name in ('collections', 'discovery-contract'):
                DISCOVERY.verify_registration(build / 'app.dll', bundle, out / (name + '-discovery'))
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
