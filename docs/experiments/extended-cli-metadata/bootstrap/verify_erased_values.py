#!/usr/bin/env python3
"""Exercise native System.Value through a separately compiled generic API."""
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

    def run(command, expected=0, error=None):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or error and error not in result.stderr or expected == 42 and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--bootstrap-ownership', args.ownership.resolve(), '--reference', args.base_library.resolve()]
    library = output / 'ErasedContracts.dll'
    run(common + ['--library', '-o', library, HERE / 'erased-contracts.rvn'])
    dependencies = ['--module', args.base_library.resolve(), '--module', library, '--system', args.seed.resolve()]
    for name, source, expected, error in [('Consumer', 'erased-consumer.rvn', 42, None),
                                        ('WrongKind', 'erased-wrong-kind.rvn', 1, 'erased value contains')]:
        app = output / (name + '.dll')
        run(common + ['--reference', library, '-o', app, HERE / source])
        run([args.runtime.resolve(), 'verify', app] + dependencies)
        run([args.runtime.resolve(), 'run', app] + dependencies, expected, error)
    inputs = [HERE / name for name in ('erased-contracts.rvn', 'erased-consumer.rvn', 'erased-wrong-kind.rvn')]
    inputs += [args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
               args.base_library.resolve(), args.ownership.resolve()] + sorted(output.glob('*.dll'))
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Native erased carrier: Int64 success and Byte invalid-format/overflow statuses; wrong-kind failure. Not exhaustive payload support.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
