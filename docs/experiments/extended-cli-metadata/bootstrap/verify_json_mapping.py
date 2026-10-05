#!/usr/bin/env python3
"""Build production JSON/introspection sources; execute artifact-only mapping consumers."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCES = [
    'System/Data/Json/' + name + '.rvn' for name in
    ('JsonDocument', 'JsonSyntax', 'JsonValue', 'JsonError', 'JsonSerializer', 'ObjectMapper')
] + ['System/Introspection/' + name + '.rvn' for name in
     ('AssemblyInfo', 'BindingFlags', 'Descriptors', 'ModuleInfo', 'ParameterInfo')
] + ['System/Runtime/Reflection/ReflectionError.rvn', 'System/Runtime/Reflection/ReflectionExtensions.rvn',
     'System/Runtime/RuntimeContext.rvn', 'System/HashCode.rvn', 'System/Boolean.rvn', 'System/BooleanParseError.rvn']
SOURCES = [ROOT / 'runtime/raven/src' / name for name in SOURCES] + [
    ROOT / 'runtime/raven/native' / (name + '.rvn') for name in
    ('RuntimeIntrospectionServices', 'RuntimeIntrospectionCalls', 'ParameterSnapshot', 'ObjectIntrospection')]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed-source', 'ownership', 'base-library', 'encoding-library', 'streams-library', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0, stdout=None):
        command = [str(x) for x in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or stdout is not None and result.stdout != stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    # Extend only the explicit primitive/service seed. No descriptor or application stubs.
    seed_text = args.seed_source.read_text()
    catalog = (HERE / 'service-seed.neoil').read_text()
    marker = '.type System.Runtime.CompilerServices.RuntimeServices\n'
    if seed_text.count(marker) != 1:
        raise ValueError('Expected exactly one retained RuntimeServices owner')
    for name in ('TypeName', 'TypeEquals', 'TypeArgumentCount', 'TypeArgument', 'TypeShape',
                 'TypeDisplayName', 'TypeMetadataToken', 'ObjectTypeHandle',
                 'ReflectionConstructionCheck', 'ReflectionConstruct'):
        if '.method static ' + name + '(' not in seed_text:
            wrapper = re.search(r'\.method static ' + name + r'\(.*?\n.end', catalog, re.S)
            if wrapper is None:
                raise ValueError('Missing catalog wrapper: ' + name)
            seed_text = seed_text.replace(marker, marker + wrapper.group(0) + '\n', 1)
        if '.function neoCLR.Runtime.' + name + '(' not in seed_text:
            native = re.search(r'\.function neoCLR.Runtime.' + name + r'\(.*?\n.end', catalog, re.S)
            if native is None:
                raise ValueError('Missing catalog binding: ' + name)
            seed_text += '\n' + native.group(0) + '\n'
    if '.method instance virtual Equals(System.Object' not in seed_text:
        object_marker = '.type class abstract System.Object\n'
        if seed_text.count(object_marker) != 1:
            raise ValueError('Expected the explicit root Object owner')
        seed_text = seed_text.replace(object_marker, object_marker + '''.method instance virtual Equals(System.Object other) -> Boolean
ldarg 0
ldarg 1
call neoCLR.Runtime.ObjectReferenceEquals(System.Object,System.Object)
ret
.end
''')
    if '.type class System.ParamArrayAttribute\n' not in seed_text:
        seed_text += '\n.type class System.ParamArrayAttribute\n.method instance .ctor() -> noresult\nret\n.end\n.end\n'
    seed_source = output / 'System.neoil'
    seed_source.write_text(seed_text)
    seed = output / 'System.neox'
    run([args.runtime.resolve(), 'assemble', seed_source, seed, '--format', 'neox'])

    ownership = json.loads(args.ownership.read_text())
    if any(library['assemblyName'] == 'JsonIntrospection' for library in ownership['libraries']):
        raise ValueError('Input already owns JsonIntrospection')
    ownership['libraries'].append(dict(assemblyName='JsonIntrospection', sources=[str(p.relative_to(ROOT)) for p in SOURCES],
        types=['System.Boolean', 'System.BooleanParseError', 'System.Data.Json.JsonSerializer',
               'System.Introspection.TypeInfo', 'System.Runtime.RuntimeContext']))
    ownership['nativePrimitives']['System.Boolean'] = 'JsonIntrospection'
    ownership['typeOf'] = dict(assemblyName='JsonIntrospection', typeInfoTypeName='System.Introspection.TypeInfo', contextTypeName='System.Runtime.RuntimeContext')
    manifest = output / 'ownership.json'
    manifest.write_text(json.dumps(ownership, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
        '--runtime-seed', seed, '--bootstrap-intrinsics', '--bootstrap-ownership', manifest]
    libraries = [args.base_library.resolve(), args.encoding_library.resolve(), args.streams_library.resolve()]
    for library in libraries:
        common += ['--reference', library]
    provider = output / 'JsonIntrospection.dll'
    run(common + ['--library', '-o', provider] + SOURCES)
    common += ['--reference', provider]
    operators = output / 'ResultOperators.dll'
    operator_source = ROOT / 'runtime/raven/src/System/ResultOperators.rvn'
    run(common + ['--library', '-o', operators, operator_source])
    deps = ['--system', seed]
    for library in libraries + [provider, operators]:
        deps += ['--module', library]
    cases = [
        ('NativeMapping', [HERE / 'json-object-consumer.rvn'], 42,
         'Model constructed\nName assigned\nModel constructed\nInvalid input begins\nInvalid input ends\nNative JSON object mapping passed\n'),
        ('ExistingMapping', [ROOT / 'docs/experiments/json-object-mapping' / name for name in ('Mapping.rvn', 'Main.rvn')], 0,
         'JSON object mapping checks passed\n'),
    ]
    case_inputs = []
    for name, sources, status, expected in cases:
        app = output / (name + '.dll')
        run(common + ['--reference', operators, '-o', app] + sources)
        run([args.runtime.resolve(), 'verify', app] + deps)
        run([args.runtime.resolve(), 'run', app, '--instructions', '100000000'] + deps, status, expected)
        case_inputs += sources + [app]
    inputs = SOURCES + case_inputs + libraries + [provider, operators, operator_source, seed_source, seed, manifest,
        args.core.resolve(), args.compiler.resolve(), args.runtime.resolve(), args.ownership.resolve(), args.seed_source.resolve(),
        HERE / 'service-seed.neoil', Path(__file__)]
    inputs += [args.compiler.resolve().parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda cwd: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cwd, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=revision(args.compiler.resolve().parent),
        scope='Unchanged production mapper/serializer/introspection sources; separately emitted ResultOperators; artifact-only nested/array/mutation/validation consumer and unchanged earlier mapping sample.',
        instructionBudget=100000000, hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
