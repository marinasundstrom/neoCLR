#!/usr/bin/env python3
"""Execute native typeof through a separate test provider and real runtime services."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed-source', 'base-library', 'ownership', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0):
        command = [str(x) for x in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=command, exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or expected == 42 and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    # Explicitly extend the existing retained seed with the one exercised service.
    seed_text = args.seed_source.read_text()
    marker = '.type System.Runtime.CompilerServices.RuntimeServices\n'
    if seed_text.count(marker) != 1 or '.method static TypeName(' in seed_text:
        raise ValueError('Expected exactly one service owner without a TypeName wrapper')
    seed_text = seed_text.replace(marker, marker + '''.method static TypeName(System.RuntimeTypeHandle handle) -> String
ldarg handle
call neoCLR.Runtime.TypeName(System.RuntimeTypeHandle)
ret
.end
''', 1)
    if '.function neoCLR.Runtime.TypeName(' not in seed_text:
        seed_text += '''\n.function neoCLR.Runtime.TypeName(System.RuntimeTypeHandle handle) -> String
.methodimpl InternalCall
.end
'''
    seed_source = output / 'seed.neoil'
    seed_source.write_text(seed_text)
    seed = output / 'System.neox'
    run([args.runtime.resolve(), 'assemble', seed_source, seed, '--format', 'neox'])
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', seed, '--bootstrap-intrinsics', '--bootstrap-ownership', args.ownership.resolve(),
              '--reference', args.base_library.resolve()]
    provider_source = HERE / 'type-handle-provider.rvn'
    consumer_source = HERE / 'type-handle-consumer.rvn'
    provider = output / 'HandleProvider.dll'
    run(common + ['--library', '-o', provider, provider_source])
    manifest = json.loads(args.ownership.read_text())
    manifest['libraries'].append(dict(assemblyName='HandleProvider', sources=[str(provider_source.relative_to(ROOT))],
                                     types=['HandleContract.Info', 'HandleContract.Descriptor', 'HandleContract.Context']))
    manifest['typeOf'] = dict(assemblyName='HandleProvider', typeInfoTypeName='HandleContract.Info',
                              contextTypeName='HandleContract.Context')
    ownership = output / 'consumer-ownership.json'
    ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    common[common.index('--bootstrap-ownership') + 1] = ownership
    app = output / 'HandleConsumer.dll'
    run(common + ['--reference', provider, '-o', app, consumer_source])
    dependencies = ['--module', provider, '--module', args.base_library.resolve(), '--system', seed]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    inputs = [Path(__file__).resolve(), provider_source, consumer_source, args.core.resolve(),
              args.seed_source.resolve(), args.base_library.resolve(), args.ownership.resolve(),
              args.runtime.resolve(), args.compiler.resolve(), seed_source, seed, provider, app, ownership]
    inputs += [args.compiler.resolve().parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda cwd: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cwd, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Generic and external nominal typeof through an artifact-only test provider. Production introspection and JSON mapping remain open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
