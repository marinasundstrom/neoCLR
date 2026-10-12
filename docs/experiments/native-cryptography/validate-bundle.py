#!/usr/bin/env python3
"""Qualify private cryptography through an installed bundle's default JIT path."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
spec = importlib.util.spec_from_file_location('crypto_cases', HERE / 'validate.py')
cases = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cases)


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
    report = dict(passed=False, bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest(), cases=[], commands=[])

    def run(command, expected=0):
        result = subprocess.run(list(map(str, command)), cwd=output, capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=list(map(str, command)), exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode != expected:
            raise RuntimeError(f'Expected {expected}, got {result.returncode}: {result.stderr}')
        return result

    try:
        for name, body, fault in cases.cases():
            locals_ = [line for line in body.splitlines() if line.startswith('.local ')]
            instructions = [line for line in body.splitlines() if not line.startswith('.local ')]
            source, binary = output / (name + '.neoil'), output / (name + '.neox')
            # Explicit private declarations in the test assembly: no replacement
            # installed seed, public facade or compiler service mapping is implied.
            source.write_text('.module App\n.entry Main\n.function Main() -> Int32\n' + '\n'.join(locals_ + instructions) + '\nldc.i4 0\nret\n.end\n' + (ROOT / 'runtime/neoCLR/Runtime/Cryptography.neoil').read_text())
            run([runtime, 'assemble', source, binary, '--format', 'neox'])
            result = run([runtime, 'run', binary], 1 if fault else 0)
            if fault and 'RuntimeError' not in result.stderr:
                raise RuntimeError('Expected terminal RuntimeError from installed host')
            report['cases'].append(dict(name=name, defaultJit=True, exitCode=result.returncode))
        report['passed'] = True
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(passed=True, cases=len(report['cases']), report=str(output / 'validation.json'))))


if __name__ == '__main__':
    main()
