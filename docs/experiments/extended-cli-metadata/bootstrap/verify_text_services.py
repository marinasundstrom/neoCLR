#!/usr/bin/env python3
"""Execute the checked text-service boundary and source-built UnicodeScalar via native imports."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'base-library', 'ownership', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--incomplete-core', type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0, error=None):
        command = [str(x) for x in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=command, exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or error and error not in result.stderr or expected == 42 and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--bootstrap-ownership', args.ownership.resolve(), '--reference', args.base_library.resolve()]
    sources = [ROOT / 'runtime/raven/src/System/Text/UnicodeScalar.rvn', HERE / 'text-service-contracts.rvn']
    library = output / 'TextServices.dll'
    run(common + ['--library', '-o', library] + sources)
    app = output / 'Consumer.dll'
    run(common + ['--reference', library, '-o', app, HERE / 'text-service-consumer.rvn'])
    dependencies = ['--module', args.base_library.resolve(), '--module', library, '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    if args.incomplete_core:
        invalid = common.copy()
        invalid[invalid.index('--core-reference') + 1] = args.incomplete_core.resolve()
        rejected = output / 'MissingServices.dll'
        run(invalid + ['--library', '-o', rejected] + sources, 1, 'RAV0117')
        if rejected.exists():
            raise RuntimeError('Missing text services published an assembly')
    inputs = sources + [HERE / 'text-service-consumer.rvn', HERE / 'service-catalog.json',
                        HERE / 'service-seed.neoil', args.compiler.resolve(), args.runtime.resolve(),
                        args.core.resolve(), args.seed.resolve(), args.base_library.resolve(),
                        args.ownership.resolve(), library, app]
    if args.incomplete_core:
        inputs.append(args.incomplete_core.resolve())
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Existing native text services execute through generated explicit bindings and separate native library/consumer. Unchanged UnicodeScalar source executes. String/Char source ownership remains open; the bootstrap still owns those declarations. No claim of .NET grapheme parity.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
