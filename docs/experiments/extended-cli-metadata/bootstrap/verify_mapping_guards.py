#!/usr/bin/env python3
"""Compile mapping-guards signatures and execute an artifact-only consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'ownership', 'base-library', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []
    inputs = [Path(__file__)] + [getattr(args, n).resolve() for n in ('compiler', 'runtime', 'core', 'seed', 'ownership', 'base_library')]

    def run(command, expected=0):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics', '--bootstrap-ownership', args.ownership.resolve(),
              '--reference', args.base_library.resolve()]
    provider = HERE / 'mapping-guards-provider.rvn'
    consumer = HERE / 'mapping-guards-consumer.rvn'
    library = output / 'MappingGuards.dll'
    app = output / 'Consumer.dll'
    run(common + ['--library', '-o', library, provider])
    run(common + ['--reference', library, '-o', app, consumer])
    dependencies = ['--module', library, '--module', args.base_library.resolve(), '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    inputs += [provider, consumer, library, app]
    inputs += [args.compiler.resolve().parent / n for n in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda cwd: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cwd, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=revision(args.compiler.resolve().parent),
                    scope='Native mapping-guards import: empty/expanded calls and an existing array return 42. JSON mapping remains open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
