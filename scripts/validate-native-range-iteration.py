#!/usr/bin/env python3
"""Run discovered signed-range behavior tests in interpreter, JIT and AOT modes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bundle, output = args.bundle.resolve(), args.output.resolve()
    if output.exists():
        raise FileExistsError('Validation requires a fresh output directory: ' + str(output))
    manifest = json.loads((bundle / 'manifest.json').read_text())
    # Preserve the existing entry point's complete input-integrity check.
    for name, digest in manifest['files'].items():
        if hashlib.sha256((bundle / name).read_bytes()).hexdigest() != digest:
            raise ValueError('Bundle drift: ' + name)
    command = [sys.executable, str(ROOT / 'scripts/test-runtime-library.py'),
               '--bundle', str(bundle), '--output', str(output),
               '--runtime', str(bundle / manifest['runtime']),
               '--aot', str(bundle / manifest['jit']['compiler']),
               '--suite', 'range-iteration',
               '--mode', 'interpreter', '--mode', 'jit', '--mode', 'native']
    result = subprocess.run(command, cwd=ROOT)
    report = output / 'report.json'
    if report.exists():
        # Keep the historical report filename; its schema is now the common runner's.
        shutil.copy2(report, output / 'results.json')
    if result.returncode:
        raise SystemExit(result.returncode)
    for name, digest in manifest['files'].items():
        if hashlib.sha256((bundle / name).read_bytes()).hexdigest() != digest:
            raise ValueError('Validation input changed: ' + name)
    print(output / 'results.json')


if __name__ == '__main__':
    main()
