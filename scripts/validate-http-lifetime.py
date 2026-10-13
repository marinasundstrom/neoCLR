#!/usr/bin/env python3
"""Qualify terminal HTTP lifetime/pipeline guards in interpreter and native hosts."""
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
    parser.add_argument('--aot', type=Path, default=ROOT / 'target/debug' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen'))
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    fixture = ROOT / 'docs/experiments/http-lifetime/Faults.rvnproj'
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest(),
                  fixtureSha256=hashlib.sha256(fixture.with_suffix('.rvn').read_bytes()).hexdigest())
    try:
        build = out / 'build'
        profile = 'windows-http' if os.name == 'nt' else 'http'
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'), '--profile', profile,
                                 '--project', str(fixture), '--bundle', str(bundle), '--aot', str(args.aot.resolve()),
                                 '--output', str(build)], cwd=ROOT, capture_output=True, text=True, timeout=600)
        (out / 'build.stdout.log').write_text(result.stdout)
        (out / 'build.stderr.log').write_text(result.stderr)
        if result.returncode:
            raise RuntimeError('HTTP lifetime fixture build failed; see retained logs')
        lib = bundle / 'lib'
        interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--system', str(lib / 'System.runtime.neox'),
                       '--object-root', str(lib / 'System.Runtime.dll')]
        for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
            interpreter += ['--module', str(lib / (name + '.dll'))]
        native = [str(build / ('app.exe' if os.name == 'nt' else 'app'))]
        cases = {'handler': 'HTTP handler is disposed', 'send': 'HTTP client is disposed',
                 'cancelled': 'HTTP client is disposed', 'text': 'HTTP client is disposed',
                 'unassigned': 'Inner handler has not been assigned',
                 'frozen': 'Handler pipeline is already started or disposed',
                 'disposed-chain': 'HTTP handler is disposed', 'null-inner': 'Inner handler cannot be null'}
        for case, message in cases.items():
            for mode, command in (('interpreter', [*interpreter, '--', case]), ('native', [*native, case])):
                result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=30)
                record = dict(case=case, mode=mode, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
                report['cases'].append(record)
                if result.returncode == 0 or message not in result.stderr or 'ERROR disposed send returned' in result.stdout:
                    raise RuntimeError(f'{case} {mode} did not fault at the expected boundary: {record}')
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS HTTP disposal and pipeline guards: eight cases in both execution modes')


if __name__ == '__main__':
    main()
