#!/usr/bin/env python3
"""Build actual Number contracts, an ordinary struct implementation and an artifact-only consumer."""
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

    manifest = json.loads(args.ownership.read_text())
    manifest['self'] = dict(assemblyName='NeoCLR.CoreProbe', typeName='System.Runtime.CompilerServices.Self')
    ownership = output / 'ownership.json'
    ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--reference', args.base_library.resolve()]
    configured = common + ['--bootstrap-ownership', ownership]
    contracts = output / 'NumberContracts.dll'
    source = ROOT / 'runtime/raven/src/System/Number.rvn'
    run(configured + ['--library', '-o', contracts, source])
    implementation = output / 'NumberImplementation.dll'
    run(configured + ['--reference', contracts, '--library', '-o', implementation, HERE / 'number-scalar.rvn'])
    app = output / 'Consumer.dll'
    run(configured + ['--reference', contracts, '--reference', implementation, '-o', app, HERE / 'number-scalar-consumer.rvn'])
    dependencies = ['--module', args.base_library.resolve(), '--module', contracts, '--module', implementation, '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    bad_source = output / 'wrong-implementation.rvn'
    bad_source.write_text((HERE / 'number-scalar.rvn').read_text().replace('static val One: Self', 'val One: Self'))
    bad = output / 'Rejected.dll'
    run(configured + ['--reference', contracts, '--library', '-o', bad, bad_source], 1, 'RAV0330')
    if bad.exists():
        raise RuntimeError('instance member satisfied static contract or failed output was published')
    inputs = [source, HERE / 'number-scalar.rvn', HERE / 'number-scalar-consumer.rvn', ownership,
              args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
              args.base_library.resolve(), args.ownership.resolve(), contracts, implementation, app]
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Actual Number source, separate ordinary struct implementation and source-free consumer execute direct operators, identities and ordering. This does not replace the numeric primitive family or establish generic callself emission.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
