#!/usr/bin/env python3
"""Qualify Raven cryptography through an installed bundle's default JIT path.

Private IL/provider boundary cases remain in validate.py. Use the installed Raven
library here so view and nominal array definitions come from one matched producer.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bundle, output = args.bundle.resolve(), args.output.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    if not manifest.get('jit', {}).get('cryptography'):
        parser.error('requires a bundle prepared with cryptography support')
    for relative, digest in manifest['files'].items():
        path = (bundle / relative).resolve()
        if not path.is_relative_to(bundle) or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise ValueError('bundle file mismatch: ' + relative)
    runtime = bundle / manifest['runtime']
    output.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest(), commands=[])

    def run(command, timeout=180):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=timeout)
        report['commands'].append(dict(command=list(map(str, command)), exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(f'Expected success, got {result.returncode}: {result.stderr}')
        return result

    try:
        suite = output / 'raven'
        run([sys.executable, ROOT / 'scripts/test-runtime-library.py', '--bundle', bundle,
             '--runtime', runtime, '--output', suite, '--suite', 'cryptography', '--mode', 'jit'], timeout=600)
        report['ravenSuite'] = json.loads((suite / 'report.json').read_text())
        result = run([runtime, 'run', suite / 'cryptography-build/app.dll'])
        if 'Tests: 3, passed: 3, failed: 0, skipped: 0' not in result.stdout:
            raise RuntimeError('default JIT did not execute the complete cryptography suite')
        report['defaultJit'] = True
        report['passed'] = True
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(passed=True, tests=3, report=str(output / 'validation.json'))))


if __name__ == '__main__':
    main()
