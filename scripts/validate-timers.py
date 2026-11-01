#!/usr/bin/env python3
"""Qualify sync/async Timer callbacks and guards in both execution modes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite', choices=('timers', 'cancel-after', 'http-timeout'), default='timers')
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    parser.add_argument('--aot', type=Path, default=ROOT / 'target/debug' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen'))
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    fixture = ROOT / 'docs/experiments' / args.suite
    report = dict(passed=False, platform=platform.platform(), instructionBudget=100000000, cases=[],
                  bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest(),
                  sources={p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in fixture.iterdir() if p.suffix in ('.rvn', '.rvnproj')})
    if args.suite in ('timers', 'cancel-after'):
        report['frameworkSources'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in (ROOT / 'tools/testing/NeoCLR.Testing/NeoClr/Testing').glob('*.rvn')}
        report['runtimeSha256'] = hashlib.sha256(args.runtime.resolve().read_bytes()).hexdigest()
        report['aotSha256'] = hashlib.sha256(args.aot.resolve().read_bytes()).hexdigest()
        report['validatorSha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    try:
        for project in ('Timers', 'Faults'):
            build = out / project
            profile = 'windows-http' if os.name == 'nt' else 'http'
            result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'), '--profile', profile,
                                     '--project', str(fixture / (project + '.rvnproj')), '--bundle', str(bundle),
                                     '--aot', str(args.aot.resolve()), '--output', str(build)],
                                    cwd=ROOT, capture_output=True, text=True, timeout=600)
            (out / (project + '.stdout.log')).write_text(result.stdout)
            (out / (project + '.stderr.log')).write_text(result.stderr)
            if result.returncode:
                raise RuntimeError('Runtime fixture build failed; see retained logs')
            lib = bundle / 'lib'
            interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--system', str(lib / 'System.runtime.neox'),
                           '--object-root', str(lib / 'System.Runtime.dll'), '--instructions', '100000000']
            for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
                interpreter += ['--module', str(lib / (name + '.dll'))]
            native = [str(build / ('app.exe' if os.name == 'nt' else 'app'))]
            cases = {'callbacks': None} if project == 'Timers' else {
                'negative': 'Timer duration is out of range', 'excessive': 'Timer duration is out of range',
                'zero-interval': 'Timer duration is out of range', 'negative-interval': 'Timer duration is out of range',
                'disposed': 'Timer is disposed'}
            if args.suite == 'cancel-after' and project == 'Faults':
                cases = {'negative': 'Timer duration is out of range', 'excessive': 'Timer duration is out of range',
                         'cancelled-negative': 'Timer duration is out of range', 'disposed': 'Cancellation source is disposed',
                         'disposed-disable': 'Cancellation source is disposed'}
            if args.suite == 'http-timeout' and project == 'Faults':
                cases = {'zero': 'HTTP timeout is out of range', 'negative': 'HTTP timeout is out of range',
                         'excessive': 'HTTP timeout is out of range', 'disposed': 'HTTP client is disposed',
                         'started': 'HTTP timeout cannot change after sending starts',
                         'cancelled-send': 'HTTP timeout cannot change after sending starts'}
            expected = {'timers': 10, 'cancel-after': 4, 'http-timeout': 9}[args.suite]
            for case, message in cases.items():
                for mode, command in (('interpreter', interpreter), ('native', native)):
                    command = [*command, *(['--'] if mode == 'interpreter' and message else []), *([case] if message else [])]
                    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=30)
                    record = dict(case=case, mode=mode, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
                    report['cases'].append(record)
                    valid = (result.returncode != 0 and message in result.stderr and 'ERROR' not in result.stdout) if message else (
                        result.returncode == 0 and result.stdout.count('PASS ') == expected and 'ERROR' not in result.stdout)
                    if args.suite == 'cancel-after' and message is None:
                        transcript = ('PASS reset countdown and deferred zero\n'
                                      'PASS disable and reenable countdown\n'
                                      'PASS disposal suppresses timed cancellation\n'
                                      'PASS manual cancellation removes countdown\n'
                                      'Tests: 4, passed: 4, failed: 0, skipped: 0\n')
                        valid = result.returncode == 0 and result.stdout == transcript and not result.stderr
                    if args.suite == 'timers' and message is None:
                        transcript = 'PASS Synchronous timers preserve one shot stop disposal repeat and reschedule lifecycles\nPASS Async repeating timers wait for callback completion\nPASS Restarting an async timer does not overlap callbacks\nPASS Disposal allows the active async callback to finish\nPASS Cancelled callbacks stop repeating timers\nPASS Pending callback cancellation stops repeating timers\nTests: 6, passed: 6, failed: 0, skipped: 0\n'
                        valid = result.returncode == 0 and result.stdout == transcript and not result.stderr
                    if not valid:
                        raise RuntimeError(f'{case} {mode} failed: {record}')
            if args.suite == 'http-timeout' and project == 'Timers':
                # This specifically guards the former native host's 15-second abort.
                result = subprocess.run([*native, 'long'], cwd=ROOT, capture_output=True, text=True, timeout=30)
                report['cases'].append(dict(case='long-request', mode='native', exitCode=result.returncode,
                                            stdout=result.stdout, stderr=result.stderr))
                if result.returncode or 'PASS default timeout permits requests beyond fifteen seconds' not in result.stdout:
                    raise RuntimeError('Long request was cut short by the native host')
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'PASS {args.suite} callbacks and guards in both execution modes')


if __name__ == '__main__':
    main()
