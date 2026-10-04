#!/usr/bin/env python3
"""Execute native typeof through a separate test provider and real runtime services."""
import argparse
import hashlib
import json
import re
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

    def run(command, expected=0, error=None):
        command = [str(x) for x in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=command, exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or expected == 42 and result.stdout or error and error not in result.stderr:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    # Copy checked catalog wrappers, preserving services already owned by the seed.
    seed_text = args.seed_source.read_text()
    catalog = (HERE / 'service-seed.neoil').read_text()
    marker = '.type System.Runtime.CompilerServices.RuntimeServices\n'
    if seed_text.count(marker) != 1:
        raise ValueError('Expected exactly one service owner')
    wrappers = []
    for name in ('TypeName', 'TypeEquals', 'TypeArgumentCount', 'TypeArgument',
                 'TypeShape', 'TypeDisplayName', 'TypeMetadataToken', 'ObjectTypeHandle',
                 'ReflectionConstructionCheck', 'ReflectionConstruct'):
        if '.method static ' + name + '(' in seed_text:
            raise ValueError('Seed already owns wrapper: ' + name)
        match = re.search(r'\.method static ' + name + r'\(.*?\n.end', catalog, re.S)
        if match is None:
            raise ValueError('Missing checked catalog wrapper: ' + name)
        wrappers.append(match.group(0))
        if '.function neoCLR.Runtime.' + name + '(' not in seed_text:
            native = re.search(r'\.function neoCLR.Runtime.' + name + r'\(.*?\n.end', catalog, re.S)
            if native is None:
                raise ValueError('Missing retained native binding: ' + name)
            seed_text += '\n' + native.group(0) + '\n'
    seed_text = seed_text.replace(marker, marker + '\n'.join(wrappers) + '\n', 1)
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
                                     types=['HandleContract.Info', 'HandleContract.Descriptor', 'HandleContract.Context', 'HandleContract.Box`1', 'HandleContract.Constructed']))
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
    invalid_source = HERE / 'type-handle-invalid-argument.rvn'
    invalid_app = output / 'InvalidArgument.dll'
    run(common + ['--reference', provider, '-o', invalid_app, invalid_source])
    run([args.runtime.resolve(), 'run', invalid_app] + dependencies, 1, 'generic argument index out of range')
    inputs = [invalid_source, invalid_app, Path(__file__).resolve(), provider_source, consumer_source, args.core.resolve(),
              HERE / 'service-seed.neoil', args.seed_source.resolve(), args.base_library.resolve(), args.ownership.resolve(),
              args.runtime.resolve(), args.compiler.resolve(), seed_source, seed, provider, app, ownership]
    inputs += [args.compiler.resolve().parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda cwd: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cwd, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Handle identity, generic arguments, object-type lookup and real parameterless reflection construction through an artifact-only provider; invalid argument and unsupported construction checks. Production introspection and JSON mapping remain open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
