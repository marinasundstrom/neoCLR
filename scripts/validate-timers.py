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
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    parser.add_argument('--aot', type=Path, default=ROOT / 'tools/aot-poc/target/debug' / ('neoclr-aot-poc.exe' if os.name == 'nt' else 'neoclr-aot-poc'))
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    fixture = ROOT / 'docs/experiments/timers'
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest(),
                  sources={p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in fixture.iterdir() if p.suffix in ('.rvn', '.rvnproj')})
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
                raise RuntimeError('Timer fixture build failed; see retained logs')
            lib = bundle / 'lib'
            interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--system', str(lib / 'System.runtime.neox'),
                           '--object-root', str(lib / 'System.Runtime.dll')]
            for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
                interpreter += ['--module', str(lib / (name + '.dll'))]
            native = [str(build / ('app.exe' if os.name == 'nt' else 'app'))]
            cases = {'callbacks': None} if project == 'Timers' else {
                'negative': 'Timer duration is out of range', 'excessive': 'Timer duration is out of range',
                'zero-interval': 'Timer duration is out of range', 'negative-interval': 'Timer duration is out of range',
                'disposed': 'Timer is disposed'}
            for case, message in cases.items():
                for mode, command in (('interpreter', interpreter), ('native', native)):
                    command = [*command, *(['--'] if mode == 'interpreter' and message else []), *([case] if message else [])]
                    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=30)
                    record = dict(case=case, mode=mode, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
                    report['cases'].append(record)
                    valid = (result.returncode != 0 and message in result.stderr and 'ERROR' not in result.stdout) if message else (
                        result.returncode == 0 and result.stdout.count('PASS ') == 10 and 'ERROR' not in result.stdout)
                    if not valid:
                        raise RuntimeError(f'{case} {mode} failed: {record}')
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS ten Timer callback scenarios and five guards in both execution modes')


if __name__ == '__main__':
    main()
