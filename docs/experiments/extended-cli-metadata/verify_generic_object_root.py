"""Run a Raven-authored generic class over the explicit source Object test root."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'seed', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    parser.add_argument('--core-attributes', action='store_true', help='Exercise bootstrap attributes with a source Object owner')
    parser.add_argument('--source-attributes', action='store_true', help='Exercise source Attribute inheritance and union marker ownership')
    parser.add_argument('--closed-root', action='store_true', help='Exercise a closed family over source Object')
    args = parser.parse_args()
    if sum((args.core_attributes, args.source_attributes, args.closed_root)) > 1:
        parser.error('Select one root scenario')
    for name in ('compiler', 'core', 'seed', 'runtime', 'output'):
        setattr(args, name, getattr(args, name).resolve())
    args.output.mkdir(parents=True, exist_ok=False)
    sources = [Path(__file__).resolve().parent / ('bootstrap/' + name) for name in
               ('source-object-root.rvn', 'generic-object-root.rvn')]
    if args.core_attributes:
        sources = [sources[0], ROOT / 'runtime/raven/src/System/Introspection/BindingFlags.rvn',
                   sources[0].parent / 'core-attributes-source-root.rvn']
    if args.source_attributes:
        sources = [sources[0], ROOT / 'runtime/raven/src/System/Attribute.rvn',
                   ROOT / 'runtime/raven/src/System/Runtime/CompilerServices/UnionAttribute.rvn',
                   sources[0].parent / 'source-union-marker.rvn']
    if args.closed_root:
        sources = [sources[0], sources[0].parent / 'closed-object-root.rvn']
    library, app = args.output / 'Root.dll', args.output / 'App.pe'
    runtime_seed = args.output / 'System.neoil'
    runtime_seed.write_text('.module System\n.references ()\n')
    commands = []

    def run(command, expected=0):
        result = subprocess.run([str(p) for p in command], cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (args.output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    compile_seed = args.seed
    if args.source_attributes:
        seed_source = sources[0].parent / 'attribute-seed.neoil'
        compile_seed = args.output / 'AttributeSeed.neox'
        run([args.runtime, 'assemble', seed_source, compile_seed, '--format', 'neox'])
    run(['dotnet', args.compiler, 'neoclr', '--core-reference', args.core, '--runtime-seed', compile_seed,
         '--source-object-root', '--library', '-o', library] + sources)
    run(['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests', '--',
         '--generic-object-consumer', library, args.core, app])
    if args.source_attributes:
        run(['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests', '--',
             '--check-source-attributes', library])
    dependencies = ['--system', runtime_seed, '--module', library, '--object-root', library]
    if args.source_attributes:
        dependencies = ['--system', compile_seed, '--module', library, '--object-root', library]
    run([args.runtime, 'verify', app] + dependencies)
    result = run([args.runtime, 'run', app] + dependencies, 42)
    if result.stdout != ('source root attributes\n' if args.core_attributes else '') or result.stderr:
        raise RuntimeError('Unexpected runtime output')
    negative_inputs = []
    if args.core_attributes:
        for name, source, diagnostic in [
            ('flags', 'namespace System\npublic class FlagsAttribute : Attribute { init() { } }\n', 'configured core FlagsAttribute'),
            ('method', 'namespace System.Runtime.CompilerServices\npublic class MethodImplAttribute : System.Attribute { init(value: MethodImplOptions) { } }\n', 'core MethodImpl(InternalCall)')]:
            spoof = args.output / ('spoof-' + name + '.rvn')
            spoof.write_text(source)
            rejected = args.output / ('Rejected-' + name + '.dll')
            result = run(['dotnet', args.compiler, 'neoclr', '--core-reference', args.core, '--runtime-seed', compile_seed,
                          '--source-object-root', '--library', '-o', rejected] + sources + [spoof], 1)
            if rejected.exists() or diagnostic not in result.stderr + result.stdout:
                raise RuntimeError('Lookalike attribute must fail the exact bootstrap owner check: ' + name)
            negative_inputs.append(spoof)
    if args.source_attributes:
        invalid = args.output / 'invalid-marker.rvn'
        invalid.write_text('namespace System.Runtime.CompilerServices\npublic class UnionAttribute : System.Attribute { private init() { } }\n')
        use = args.output / 'marker-union.rvn'
        use.write_text('public union Choice {\n    case None\n}\n')
        rejected = args.output / 'Rejected-marker.dll'
        result = run(['dotnet', args.compiler, 'neoclr', '--core-reference', args.core, '--runtime-seed', compile_seed,
                      '--source-object-root', '--library', '-o', rejected] + sources[:2] + [invalid, use], 1)
        if rejected.exists() or 'source UnionAttribute requires a public parameterless constructor' not in result.stderr + result.stdout:
            raise RuntimeError('Invalid source marker must fail before publication')
        negative_inputs += [invalid, use]
        embedded_source = args.output / 'embedded-marker.rvn'
        embedded_source.write_text('public union Choice {\n    case None\n}\npublic func RunGeneric() -> int => 42\n')
        embedded = args.output / 'Embedded.dll'
        embedded_app = args.output / 'Embedded.pe'
        run(['dotnet', args.compiler, 'neoclr', '--core-reference', args.core, '--runtime-seed', compile_seed,
             '--source-object-root', '--library', '-o', embedded, sources[0], embedded_source])
        run(['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests', '--',
             '--generic-object-consumer', embedded, args.core, embedded_app])
        embedded_dependencies = ['--system', compile_seed, '--module', embedded, '--object-root', embedded]
        run([args.runtime, 'verify', embedded_app] + embedded_dependencies)
        run([args.runtime, 'run', embedded_app] + embedded_dependencies, 42)
        negative_inputs += [embedded_source, embedded, embedded_app]
    files = sources + negative_inputs + [Path(__file__), args.compiler, args.core, args.seed, args.runtime, runtime_seed, library, app]
    if args.source_attributes:
        files += [seed_source, compile_seed]
    files += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    evidence = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                    compilerRevision=args.compiler_revision, commands=commands,
                    scope=('Closed family over source Object' if args.closed_root else 'Source Attribute hierarchy and union marker' if args.source_attributes else 'Source Object fixture with bootstrap attributes' if args.core_attributes else 'Source Object fixture and generic class') + '; consumer generated by metadata API, not Raven imported-root acceptance.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in files})
    (args.output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(args.output / 'validation.json')


if __name__ == '__main__':
    main()
