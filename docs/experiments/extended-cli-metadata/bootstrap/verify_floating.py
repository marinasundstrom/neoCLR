#!/usr/bin/env python3
"""Exercise floating-point metadata through separate libraries and consumers on both targets."""
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
    library = output / 'FloatingContracts.dll'
    run(common + ['--library', '-o', library, HERE / 'floating-contracts.rvn'])
    dependencies = ['--module', args.base_library.resolve(), '--module', library, '--system', args.seed.resolve()]
    for name, source, expected, error in [('Consumer', 'floating-consumer.rvn', 42, None), ('ParsingConsumer', 'floating-parsing-consumer.rvn', 42, None)]:
        app = output / (name + '.dll')
        run(common + ['--reference', library, '-o', app, HERE / source])
        run([args.runtime.resolve(), 'verify', app] + dependencies)
        run([args.runtime.resolve(), 'run', app] + dependencies, expected, error)
    cli_output = output / 'dotnet'
    cli_output.mkdir()
    cli_library = cli_output / 'FloatingContracts.dll'
    cli_app = cli_output / 'Consumer.dll'
    cli = ['dotnet', args.compiler.resolve(), '--framework', 'net10.0', '--emit-core-types-only']
    run(cli + ['--output-type', 'classlib', '-o', cli_library, HERE / 'floating-contracts.rvn'])
    run(cli + ['--refs', cli_library, '-o', cli_app, HERE / 'floating-consumer.rvn'])
    (cli_output / 'Consumer.runtimeconfig.json').write_text(json.dumps({
        'runtimeOptions': {'tfm': 'net10.0', 'framework': {'name': 'Microsoft.NETCore.App', 'version': '10.0.0'}}}))
    run(['dotnet', cli_app], 42)
    inputs = [HERE / name for name in ('floating-contracts.rvn', 'floating-consumer.rvn', 'floating-parsing-consumer.rvn')]
    inputs += [args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
               args.base_library.resolve(), args.ownership.resolve()] + sorted(output.rglob('*.dll'))
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Separate floating library and artifact-only consumer: fields, properties, arrays, generic calls, numeric conversions, NaN comparisons and signed zero execute on both targets. Source Single/Double class-library declarations remain open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
