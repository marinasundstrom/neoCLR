#!/usr/bin/env python3
"""Compile native runtime-service sources and execute an artifact-only consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'ownership', 'base-library', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0, diagnostic=None):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or (diagnostic and diagnostic not in result.stdout + result.stderr):
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        if expected == 42 and result.stdout:
            raise RuntimeError('Unexpected stdout')

    # Exact executable dependency for this test: Object and the opaque handle only.
    # There are no duplicate internal calls or source-library types in this seed.
    seed_source = output / 'System.neoil'
    seed_source.write_text('''.module System
.type class abstract System.Object
.method instance .ctor() -> noresult
ret
.end
.end
.type System.RuntimeTypeHandle
.sealed
.end
''')
    seed = output / 'System.neox'
    run([args.runtime.resolve(), 'assemble', seed_source, seed, '--format', 'neox'])
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', seed, '--bootstrap-intrinsics', '--bootstrap-ownership', args.ownership.resolve(), '--reference', args.base_library.resolve()]
    services = ROOT / 'runtime/raven/native/RuntimeHandleServices.rvn'
    provider_source = HERE / 'internal-call-provider.rvn'
    consumer_source = HERE / 'internal-call-consumer.rvn'
    library = output / 'HandleServices.dll'
    run(common + ['--library', '-o', library, services, provider_source])
    app = output / 'Consumer.dll'
    run(common + ['--reference', library, '-o', app, consumer_source])
    dependencies = ['--module', library, '--system', seed]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)

    original = services.read_text()
    invalid = {
        'unmarked': original.replace('[MethodImpl(MethodImplOptions.InternalCall)]', ''),
        'public': original.replace('internal extern', 'public extern'),
        'wrong-namespace': original.replace('namespace neoCLR.Runtime', 'namespace Other'),
        'generic': original.replace('TypeArgumentCount(', 'TypeArgumentCount<T>('),
        'wrong-flags': original.replace('MethodImplOptions.InternalCall', 'MethodImplOptions.NoInlining'),
        'body': original.replace('-> int;', '-> int { return 0 }'),
    }
    for name, source in invalid.items():
        path = output / (name + '.rvn')
        path.write_text(source)
        destination = output / (name + '.dll')
        run(common + ['--library', '-o', destination, path], 1, 'RAV1916' if name == 'body' else 'NEOMETA001')
        if destination.exists():
            raise RuntimeError('Rejected declaration published output: ' + name)

    unknown = output / 'unknown.rvn'
    unknown.write_text(original.split('[MethodImpl(MethodImplOptions.InternalCall)]')[0] + '[MethodImpl(MethodImplOptions.InternalCall)]\ninternal extern func MissingService() -> int;\n')
    unknown_library = output / 'Unknown.dll'
    run(common + ['--library', '-o', unknown_library, unknown])
    run([args.runtime.resolve(), 'verify', app, '--module', library, '--module', unknown_library,
         '--system', seed], 1, 'no runtime binding')

    inputs = [Path(__file__), services, provider_source, consumer_source, args.core.resolve(),
              args.ownership.resolve(), args.base_library.resolve(), args.runtime.resolve(), args.compiler.resolve(), seed_source, seed, library, app]
    inputs += [args.compiler.resolve().parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda cwd: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cwd, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Source-declared internal calls, separate native consumer, failed publication and unknown binding. Production descriptors/JSON remain open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
