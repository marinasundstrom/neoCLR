#!/usr/bin/env python3
"""Execute explicit interface properties across three assemblies on both targets."""
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
    parser.add_argument("--scenario", choices=("explicit-property", "pattern-expression"), default="explicit-property")
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
    sources = [HERE / (args.scenario + '-' + part + '.rvn')
               for part in ('contracts', 'implementation', 'consumer')]
    artifacts = []
    for target in ('native', 'dotnet'):
        directory = output / target
        directory.mkdir()
        contracts, implementation, app = [directory / (name + '.dll')
                                          for name in ('Contracts', 'Implementation', 'Consumer')]
        if target == 'native':
            run(common + ['--library', '-o', contracts, sources[0]])
            run(common + ['--reference', contracts, '--library', '-o', implementation, sources[1]])
            run(common + ['--reference', contracts, '--reference', implementation, '-o', app, sources[2]])
            dependencies = ['--module', contracts, '--module', implementation,
                            '--module', args.base_library.resolve(), '--system', args.seed.resolve()]
            run([args.runtime.resolve(), 'verify', app] + dependencies)
            run([args.runtime.resolve(), 'run', app] + dependencies, 42)
        else:
            cli = ['dotnet', args.compiler.resolve(), '--framework', 'net10.0', '--emit-core-types-only']
            run(cli + ['--output-type', 'classlib', '-o', contracts, sources[0]])
            run(cli + ['--refs', contracts, '--output-type', 'classlib', '-o', implementation, sources[1]])
            run(cli + ['--refs', contracts, '--refs', implementation, '-o', app, sources[2]])
            app.with_suffix('.runtimeconfig.json').write_text(json.dumps({'runtimeOptions': {
                'tfm': 'net10.0', 'framework': {'name': 'Microsoft.NETCore.App', 'version': '10.0.0'}}}))
            run(['dotnet', app], 42)
        artifacts += [contracts, implementation, app]
    inputs = sources + artifacts + [args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(),
                                    args.seed.resolve(), args.base_library.resolve(), args.ownership.resolve()]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scenario=args.scenario, scope='Three separately compiled assemblies on each target; checked scenario source assertions. Revisions are base revisions; working tree implementation validated.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
