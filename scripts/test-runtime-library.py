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
    parser.add_argument('--suite', action='append', choices=('collections', 'discovery-contract', 'runner-contract'),
                        help='Run only this suite (repeatable); default runs all suites')
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
        selected = [case for case in expected if not args.suite or case[0] in args.suite]
        report['suites'] = [case[0] for case in selected]
        if any(case[0] != 'runner-contract' for case in selected):
            validation.validate(bundle, out / 'discovery-signatures')
        for name, exit_code, stdout in selected:
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
            selections = {
                'collections': [(['--filter', 'ArrayQueue'], 0, 'PASS ArrayQueue preserves FIFO\nTests: 1, passed: 1, failed: 0, skipped: 0\n')],
                'discovery-contract': [(['--id', 'manual'], 0, 'PASS manually registered companion\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                                       (['--filter', 'after discovered'], 0, 'PASS after discovered failure\nTests: 1, passed: 1, failed: 0, skipped: 0\n')],
                'runner-contract': [
                    (['--entry-probe', '', 'two words', 'Räven ☕ 😀', 'quote"slash\\', '--id'], 0, 'PASS entry arguments preserve text\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                    (['--id', 'runner.after'], 0, 'PASS after failure\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                    (['--id', 'runner.failure'], 1, 'FAIL intentional assertion failure: Expected 1, actual 2\nTests: 1, passed: 0, failed: 1, skipped: 0\n'),
                    (['--id', 'runner.skipped'], 0, 'SKIP intentional skip: contract probe\nTests: 1, passed: 0, failed: 0, skipped: 1\n'),
                    (['--filter', 'failure'], 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nTests: 3, passed: 2, failed: 1, skipped: 0\n'),
                    (['--filter', 'RUNNER'], 2, 'ERROR No tests matched the selection\n'),
                    (['--filter'], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                    (['--filter', ''], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                    (['--unknown', 'after'], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                    (['--id', 'runner.after', '--filter', 'after'], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                ],
            }[name]
            for arguments, selected_exit, selected_stdout in selections:
                for mode, command, cwd, environment in [('native', [exe, *arguments], isolated, env),
                                                        ('interpreter', [*interpreter, '--', *arguments], ROOT, None)]:
                    result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=180)
                    record = dict(suite=name, mode=mode, arguments=arguments, exitCode=result.returncode,
                                  stdout=result.stdout.decode('utf-8').replace('\r\n', '\n'),
                                  stderr=result.stderr.decode('utf-8').replace('\r\n', '\n'))
                    report['cases'].append(record)
                    if (record['exitCode'], record['stdout'], record['stderr']) != (selected_exit, selected_stdout, ''):
                        raise RuntimeError(name + ' ' + mode + ' selection did not match: ' + repr(record))
            print(name + ': filtering checks passed', flush=True)
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
