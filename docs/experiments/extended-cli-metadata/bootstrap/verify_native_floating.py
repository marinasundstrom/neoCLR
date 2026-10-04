#!/usr/bin/env python3
"""Compile unchanged native Single/Double declarations and execute an artifact-only consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'base-library', 'number-contracts', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--ownership', type=Path, default=HERE / 'offset-ownership.json')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0, error=None):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or error and error not in result.stderr or expected == 42 and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    ownership = json.loads(args.ownership.read_text())
    sources = ['runtime/raven/src/System/' + name + '.rvn' for name in ('Single', 'Double', 'NumberParseError')]
    ownership['libraries'].append(dict(assemblyName='FloatingNumbers', sources=sources,
                                      types=['System.Single', 'System.Double', 'System.NumberParseError']))
    ownership['nativePrimitives'] = {'System.Single': 'FloatingNumbers', 'System.Double': 'FloatingNumbers'}
    manifest = output / 'ownership.json'
    manifest.write_text(json.dumps(ownership, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--bootstrap-ownership', manifest, '--reference', args.base_library.resolve(),
              '--reference', args.number_contracts.resolve()]
    library = output / 'FloatingNumbers.dll'
    run(common + ['--library', '-o', library] + [ROOT / source for source in sources])
    app = output / 'Consumer.dll'
    consumer = HERE / 'native-floating-consumer.rvn'
    run(common + ['--reference', library, '-o', app, consumer])
    dependencies = ['--module', args.base_library.resolve(), '--module', args.number_contracts.resolve(),
                    '--module', library, '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    # Missing native providers must never select bootstrap declarations or publish output.
    missing = output / 'Missing.dll'
    run(common + ['-o', missing, consumer], 1)
    if missing.exists():
        raise RuntimeError('Failed native binding published an assembly')
    inputs = [ROOT / source for source in sources] + [consumer, manifest, args.number_contracts.resolve()]
    inputs += [args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
               args.base_library.resolve(), args.ownership.resolve()] + sorted(output.rglob('*.dll'))
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Unchanged source Single/Double declarations execute via direct native import: scalar receivers, properties, parsing Result payloads/errors, NaN ordering and arrays. Generic Number-constrained calls remain open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
