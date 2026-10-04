#!/usr/bin/env python3
"""Build unchanged JSON sources and run artifact-only DOM and internal-codec tests."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'base-library',
                 'encoding-library', 'streams-library', 'ownership', 'output'):
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

    sources = [ROOT / 'runtime/raven/src/System/Data/Json' / (name + '.rvn')
               for name in ('JsonDocument', 'JsonSyntax', 'JsonValue', 'JsonError')]
    sources.append(ROOT / 'runtime/raven/src/System/Runtime/Reflection/ReflectionError.rvn')
    dependencies = [args.base_library.resolve(), args.encoding_library.resolve(),
                    args.streams_library.resolve()]
    common = ['dotnet', args.compiler.resolve(), 'neoclr',
              '--core-reference', args.core.resolve(), '--runtime-seed', args.seed.resolve(),
              '--bootstrap-intrinsics', '--bootstrap-ownership', args.ownership.resolve()]
    for dependency in dependencies:
        common += ['--reference', dependency]
    runtime_dependencies = ['--system', args.seed.resolve()]
    for dependency in dependencies:
        runtime_dependencies += ['--module', dependency]

    fixtures = []
    ownership_files = []
    json_types = ['System.Data.Json.' + name for name in (
        'DocumentReader', 'DocumentWriter', 'JsonSyntax', 'MessageReader', 'JsonValue',
        'JsonObject', 'JsonArray', 'JsonString', 'JsonNumber', 'JsonBoolean', 'JsonNull', 'JsonError')]
    json_types.append('System.Runtime.Reflection.ReflectionError')
    for name, helper, consumer in (
        ('Json', None, HERE / 'json-library-consumer.rvn'),
        ('JsonContract', HERE / 'json-document-checks.rvn', HERE / 'json-document-consumer.rvn'),
    ):
        library = output / (name + '.dll')
        # The second library includes a test-only entry point for internal codecs.
        # Never link both copies of the JSON declarations into one runtime context.
        library_sources = sources + ([helper] if helper else [])
        fixtures += ([helper] if helper else []) + [consumer]
        run(common + ['--library', '-o', library] + library_sources)
        app = output / (name + 'Consumer.dll')
        manifest = json.loads(args.ownership.read_text())
        manifest['libraries'].append(dict(
            assemblyName=name, sources=[str(p.relative_to(ROOT)) for p in library_sources],
            types=json_types + (['NeoCLR.Tests.JsonDocumentChecks'] if helper else [])))
        owned = output / (name + '-ownership.json')
        owned.write_text(json.dumps(manifest, indent=2) + '\n')
        ownership_files.append(owned)
        consumer_common = common.copy()
        consumer_common[consumer_common.index('--bootstrap-ownership') + 1] = owned
        run(consumer_common + ['--reference', library, '-o', app, consumer])
        linked = runtime_dependencies + ['--module', library]
        run([args.runtime.resolve(), 'verify', app] + linked)
        run([args.runtime.resolve(), 'run', app] + linked, 42)

    compiler_files = [args.compiler.resolve()]
    for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll'):
        path = args.compiler.resolve().parent / name
        if path.exists():
            compiler_files.append(path)
    inputs = sources + fixtures + dependencies + compiler_files + ownership_files + [
        Path(__file__).resolve(), args.core.resolve(), args.seed.resolve(),
        args.runtime.resolve(), args.ownership.resolve()] + sorted(output.glob('*.dll'))
    evidence = dict(
        scope='Native public DOM and internal codec execution; public serializer/object mapping and .NET library parity remain outside this gate.',
        runtimeRepositoryRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        compilerRepositoryRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=args.compiler.resolve().parent, text=True).strip(),
        commands=commands,
        hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
