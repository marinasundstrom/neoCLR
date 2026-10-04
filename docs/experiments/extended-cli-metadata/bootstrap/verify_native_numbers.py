#!/usr/bin/env python3
"""Compile all ten native numeric sources and execute an artifact-only consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'output'):
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
        if result.returncode != expected or error and error not in result.stderr or expected in (42, 99) and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    ownership = json.loads(args.ownership.read_text())
    if len(ownership['libraries']) != 1:
        raise ValueError('Expected one cumulative source-library owner')
    source_library = ownership['libraries'][0]
    source_library['assemblyName'] = 'Numbers'
    numeric = ('SByte', 'Byte', 'Int16', 'UInt16', 'Int32', 'UInt32', 'Int64', 'UInt64', 'Single', 'Double')
    additions = numeric + ('Number', 'NumberParseError', 'IntegerDivisionError')
    source_library['sources'] += ['runtime/raven/src/System/' + name + '.rvn' for name in additions]
    source_library['types'] += ['System.' + name for name in additions]
    sources = source_library['sources']
    ownership['nativePrimitives'] = {'System.' + name: 'Numbers' for name in numeric}
    ownership['iteration']['assemblyName'] = 'Numbers'
    ownership['propagation']['assemblyName'] = 'Numbers'
    manifest = output / 'ownership.json'
    manifest.write_text(json.dumps(ownership, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--bootstrap-ownership', manifest]
    library = output / 'Numbers.dll'
    run(common + ['--library', '-o', library] + [ROOT / source for source in sources])
    app = output / 'Consumer.dll'
    consumer = HERE / 'native-number-consumer.rvn'
    run(common + ['--reference', library, '-o', app, consumer])
    dependencies = ['--module', library, '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 99)
    algorithms = output / 'NumberAlgorithms.dll'
    algorithm_source = HERE / 'native-number-algorithms.rvn'
    generic_source = HERE / 'native-number-generic-consumer.rvn'
    run(common + ['--reference', library, '--library', '-o', algorithms, algorithm_source])
    generic_app = output / 'GenericConsumer.dll'
    run(common + ['--reference', library, '--reference', algorithms, '-o', generic_app, generic_source])
    generic_dependencies = dependencies + ['--module', algorithms]
    run([args.runtime.resolve(), 'verify', generic_app] + generic_dependencies)
    run([args.runtime.resolve(), 'run', generic_app] + generic_dependencies, 42)
    invalid_source = output / 'InvalidBound.rvn'
    invalid_source.write_text('import NumberAlgorithms.*\nfunc Bad() -> string => Sum<string>("a", "b")\n')
    invalid_output = output / 'InvalidBound.dll'
    run(common + ['--reference', library, '--reference', algorithms, '--library', '-o', invalid_output, invalid_source], 1, 'RAV0320')
    if invalid_output.exists():
        raise RuntimeError('Incompatible generic argument published an assembly')
    # Missing native providers must never select bootstrap declarations or publish output.
    missing = output / 'Missing.dll'
    run(common + ['-o', missing, consumer], 1)
    if missing.exists():
        raise RuntimeError('Failed native binding published an assembly')
    conflicting_seed = output / 'ConflictingSystem.neox'
    run([args.runtime.resolve(), 'assemble', HERE / 'comparer-seed.neoil', conflicting_seed, '--format', 'neox'])
    conflicting = common.copy()
    conflicting[conflicting.index('--runtime-seed') + 1] = conflicting_seed
    rejected = output / 'DuplicateOwner.dll'
    run(conflicting + ['--reference', library, '-o', rejected, consumer], 1)
    if rejected.exists():
        raise RuntimeError('Duplicate primitive seed ownership published an assembly')
    inputs = [HERE / name for name in ('numeric-seed.neoil', 'collection-seed.neoil', 'union-seed.neoil', 'service-seed.neoil', 'service-catalog.json')] + [ROOT / source for source in sources] + [consumer, algorithm_source, generic_source, invalid_source, manifest]
    inputs += [args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
               args.ownership.resolve()] + sorted(output.rglob('*.dll'))
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='The cumulative source-library subset plus all ten unchanged numeric sources execute via direct native import: scalar receivers, identities, parsing boundaries/errors, NaN ordering, arrays, integer display and checked division. Int32/Int64 seed copies are absent. A separately compiled generic algorithms library and source-free consumer execute all ten types through Number arithmetic, Zero/One, inherited ComparableTo<Self> dispatch and generic forwarding.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
