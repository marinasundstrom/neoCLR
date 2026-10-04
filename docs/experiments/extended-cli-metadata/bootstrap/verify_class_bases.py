#!/usr/bin/env python3
"""Compile the same forward-base constructor/mutation consumer for both runtimes."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'base-library', 'ownership', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = HERE / 'class-base-consumer.rvn'
    commands = []

    def run(command, expected):
        result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or expected == 42 and (result.stdout or result.stderr):
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    for native in (False, True):
        image = output / ('Native.dll' if native else 'DotNet.dll')
        flags = ['neoclr', '--core-reference', args.core.resolve(), '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics', '--bootstrap-ownership', args.ownership.resolve(), '--reference', args.base_library.resolve()] if native else ['--framework', 'net10.0', '--emit-core-types-only']
        run(['dotnet', args.compiler.resolve(), *flags, '-o', image, source], 0)
        if native:
            dependencies = ['--module', args.base_library.resolve(), '--system', args.seed.resolve()]
            run([args.runtime.resolve(), 'verify', image, *dependencies], 0)
            run([args.runtime.resolve(), 'run', image, *dependencies], 42)
        else:
            run(['dotnet', 'exec', '--runtimeconfig', args.compiler.resolve().with_suffix('.runtimeconfig.json'), image], 42)
    inputs = [source, Path(__file__), args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(), args.base_library.resolve(), args.ownership.resolve(), *output.glob('*.dll')]
    inputs += list(args.compiler.resolve().parent.glob('Raven.CodeAnalysis*.dll'))
    inputs += list(args.compiler.resolve().parent.glob('NeoCLR.Metadata*.dll'))
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(scope='Local nongeneric class bases, constructor chaining, inherited field mutation and alias identity; both targets execute. External/virtual/closed hierarchies and JSON remain open.',
                    runtimeRevision=revision(HERE), compilerRevision=revision(args.compiler.resolve().parent),
                    revisionsAreWorkingTreeBases=True,
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
