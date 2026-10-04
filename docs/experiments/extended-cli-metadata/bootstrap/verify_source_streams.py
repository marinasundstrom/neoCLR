#!/usr/bin/env python3
"""Compile unchanged streams as a separate native library, then import and execute."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'base-library', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--ownership', type=Path, default=HERE / 'offset-ownership.json')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or expected == 42 and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    sources = [ROOT / 'runtime/raven/src/System/IO' / (name + '.rvn')
               for name in ('InputStream', 'OutputStream', 'SeekableStream', 'StreamError', 'MemoryStream')]
    manifest = json.loads(args.ownership.read_text())
    manifest['unit'] = dict(assemblyName='NeoCLR.CoreProbe', typeName='System.Void')
    manifest['libraries'].append(dict(assemblyName='Streams', sources=[str(p.relative_to(ROOT)) for p in sources],
                                      types=['System.IO.' + p.stem for p in sources]))
    ownership = output / 'ownership.json'
    ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    # Before Streams exists, validate only the dependency owners plus the explicit unit contract.
    bootstrap = output / 'bootstrap.json'
    before = dict(manifest, libraries=manifest['libraries'][:-1])
    bootstrap.write_text(json.dumps(before, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--reference', args.base_library.resolve()]
    streams = output / 'Streams.dll'
    run(common + ['--bootstrap-ownership', bootstrap, '--library', '-o', streams] + sources)
    common += ['--bootstrap-ownership', ownership, '--reference', streams]
    for name, source in [('StreamsConsumer', HERE / 'stream-consumer.rvn')]:
        image = output / (name + '.dll')
        run(common + ['-o', image, source])
        dependencies = ['--module', args.base_library.resolve(), '--module', streams, '--system', args.seed.resolve()]
        run([args.runtime.resolve(), 'verify', image] + dependencies)
        run([args.runtime.resolve(), 'run', image] + dependencies, 42)
    contracts = output / 'UnitContracts.dll'
    run(common + ['--library', '-o', contracts, HERE / 'unit-contracts.rvn'])
    app = output / 'UnitConsumer.dll'
    run(common + ['--reference', contracts, '-o', app, HERE / 'unit-consumer.rvn'])
    dependencies += ['--module', contracts]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    def digest(path):
        return hashlib.sha256(path.read_bytes()).hexdigest()
    inputs = sources + [HERE / 'stream-consumer.rvn', HERE / 'unit-contracts.rvn', HERE / 'unit-consumer.rvn',
                        args.core.resolve(), args.seed.resolve(), args.base_library.resolve(), args.compiler.resolve(),
                        args.runtime.resolve(), ownership] + sorted(output.glob('*.dll'))
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    compiler_revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=args.compiler.resolve().parent, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision, compilerRepositoryRevision=compiler_revision,
                    scope='Separate native stream library and generic unit values; not full System completion.',
                    hashes={str(p): digest(p) for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
