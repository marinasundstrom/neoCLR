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
    parser.add_argument("--closed-family", action="store_true", help="Use the closed root/protected constructor case")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = HERE / ('closed-family-consumer.rvn' if args.closed_family else 'class-base-consumer.rvn')
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
    if args.closed_family:
        declarations, separator, entry = source.read_text().partition('func Main()')
        if not separator:
            raise RuntimeError('missing consumer entry')
        library_source = output / 'Models.rvn'
        consumer_source = output / 'Consumer.rvn'
        invalid_source = output / 'Invalid.rvn'
        library_source.write_text(declarations)
        consumer_source.write_text(separator + entry)
        invalid_source.write_text('public class Invalid : Base { init(): base(1) {} }\n')
        for native in (False, True):
            prefix = 'Native' if native else 'DotNet'
            library = output / (prefix + 'Models.dll')
            consumer = output / (prefix + 'Consumer.dll')
            invalid = output / (prefix + 'Invalid.dll')
            flags = ['neoclr', '--core-reference', args.core.resolve(), '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics', '--bootstrap-ownership', args.ownership.resolve(), '--reference', args.base_library.resolve()] if native else ['--framework', 'net10.0', '--emit-core-types-only']
            library_flags = ['--library'] if native else ['--output-type', 'library']
            reference_flag = '--reference' if native else '--refs'
            run(['dotnet', args.compiler.resolve(), *flags, *library_flags, '-o', library, library_source], 0)
            # Only the emitted library is referenced; its source is absent here.
            run(['dotnet', args.compiler.resolve(), *flags, reference_flag, library, '-o', consumer, consumer_source], 0)
            if native:
                dependencies = ['--module', library, '--module', args.base_library.resolve(), '--system', args.seed.resolve()]
                run([args.runtime.resolve(), 'verify', consumer, *dependencies], 0)
                run([args.runtime.resolve(), 'run', consumer, *dependencies], 42)
            else:
                run(['dotnet', 'exec', '--runtimeconfig', args.compiler.resolve().with_suffix('.runtimeconfig.json'), consumer], 42)
            run(['dotnet', args.compiler.resolve(), *flags, reference_flag, library, *library_flags, '-o', invalid, invalid_source], 1)
            if invalid.exists():
                raise RuntimeError('invalid external closed-family child was published')
    inputs = [source, Path(__file__), args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(), args.base_library.resolve(), args.ownership.resolve(), *output.glob('*.dll'), *output.glob('*.rvn')]
    inputs += list(args.compiler.resolve().parent.glob('Raven.CodeAnalysis*.dll'))
    inputs += list(args.compiler.resolve().parent.glob('NeoCLR.Metadata*.dll'))
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(scope=('Closed-family root and protected constructor' if args.closed_family else 'Local nongeneric class bases') + ', constructor chaining, inherited field mutation and alias identity; both targets execute. External/virtual hierarchies and JSON remain open.',
                    runtimeRevision=revision(HERE), compilerRevision=revision(args.compiler.resolve().parent),
                    revisionsAreWorkingTreeBases=True,
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
