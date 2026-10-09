#!/usr/bin/env python3
"""Verify the bounded native-only core consumer; not a full class-library bootstrap."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('raven', 'runtime', 'aot', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--driver', type=Path, help='Optional native-enabled rvnc.dll; compile through the driver too.')
    parser.add_argument('--project', action='store_true', help='Validate native-only project selection; requires --driver.')
    parser.add_argument('--value-types', action='store_true', help='With --project, check copied values and the production union core frontier.')
    parser.add_argument('--text-services', action='store_true', help='With --project, verify native UTF-8 wrappers and production union frontier.')
    parser.add_argument('--unions', action='store_true', help='With --text-services, execute unchanged production Option/Result and pattern controls.')
    parser.add_argument('--string-boxing', action='store_true', help='With --unions, check generic String boxing/display and reference identity.')
    parser.add_argument('--union-display', action='store_true', help='With --unions, check String/Int32 union display and formatter binding; excludes --string-boxing.')
    args = parser.parse_args()
    if args.project and not args.driver:
        parser.error('--project requires --driver')
    if args.value_types and not args.project:
        parser.error('--value-types requires --project')
    if args.text_services and (not args.project or args.value_types):
        parser.error('--text-services requires --project and cannot combine with --value-types')
    if args.unions and not args.text_services:
        parser.error('--unions requires --text-services')
    if args.string_boxing and not args.unions:
        parser.error('--string-boxing requires --unions')
    if args.union_display and (not args.unions or args.string_boxing):
        parser.error('--union-display requires --unions and excludes --string-boxing')
    output = args.output.resolve()
    if output.exists():
        raise FileExistsError(output)
    output.mkdir(parents=True)
    artifacts = output / 'artifacts'
    report = {'scope': 'Native-only catalog core and separate-library integer consumer; explicit fixture Object and empty runtime seed.', 'commands': []}

    def run(command, expected=0, include_stderr=False):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=180)
        report['commands'].append({'command': command, 'exitCode': result.returncode,
                                   'stdout': result.stdout, 'stderr': result.stderr})
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode != expected:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout + (result.stderr if include_stderr else '')

    metadata = ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj'
    run(['dotnet', 'run', '--project', HERE / 'Probe.csproj', '-p:RavenRoot=' + str(args.raven.resolve()),
         '-p:NeoClrMetadataProject=' + str(metadata), '-p:WarningLevel=0', '--', artifacts, *(['--text-services'] if args.text_services else [])])
    runtime = args.runtime.resolve()
    aot = args.aot.resolve()
    core = artifacts / 'NativeCore.dll'
    library = artifacts / 'Input.dll'
    consumer = artifacts / 'Consumer.dll'
    production_sources = [ROOT / 'runtime/raven/src/System' / name for name in (
        'Propagatable.rvn', 'Option.rvn', 'Result.rvn', 'Runtime/CompilerServices/UnionAttribute.rvn')]
    union_library = artifacts / 'ProductionValues.dll'
    attribute_source = ROOT / 'runtime/raven/src/System/Attribute.rvn'
    if args.unions:
        run(['dotnet', args.driver.resolve(), 'neoclr', '--native-core-reference', core,
             '--library', '-o', union_library, *production_sources, attribute_source])
    if args.driver and not args.project:
        driver = args.driver.resolve()
        consumer = artifacts / 'DriverConsumer.dll'
        command = ['dotnet', driver, 'neoclr', '--native-core-reference', core, '--reference', library]
        run([*command, '-o', consumer, HERE / 'consumer.rvn'])
        for index, flags in enumerate([
            ['--core-reference', core], ['--native-core-reference', core],
            ['--runtime-seed', artifacts / 'System.neox'], ['--bootstrap-intrinsics'],
            ['--system-method', 'System.Math.Min/2'],
        ]):
            rejected = artifacts / f'rejected-{index}.dll'
            run([*command, *flags, '-o', rejected, HERE / 'consumer.rvn'], expected=1)
            if rejected.exists():
                raise AssertionError('Rejected driver invocation published output')
        before = consumer.read_bytes()
        run([*command, '-o', consumer, HERE / 'consumer.rvn'], expected=1)
        if consumer.read_bytes() != before:
            raise AssertionError('Driver overwrote existing output')
    seed = artifacts / 'System.neox'
    run([runtime, 'assemble', HERE / 'System.neoil', seed, '--format', 'neox'])
    if args.project:
        directory = artifacts / 'project'
        directory.mkdir()
        (directory / 'Main.rvn').write_text((HERE / 'display-consumer.rvn').read_text() if args.union_display
            else (HERE / 'boxing-consumer.rvn').read_text() if args.string_boxing
            else (HERE / 'union-consumer.rvn').read_text() if args.unions
            else (HERE / 'text-consumer.rvn').read_text() if args.text_services
            else (HERE / 'value-consumer.rvn').read_text() if args.value_types
            else 'module Example.App\n' + (HERE / 'consumer.rvn').read_text())
        project = directory / 'App.rvnproj'
        root = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
        group = ET.SubElement(root, 'PropertyGroup')
        for name, value in {
            'TargetFramework': 'net10.0', 'OutputType': 'Exe',
            'RavenTargetPlatform': 'NeoCLR', 'RavenMetadataFormat': 'NeoCLR',
            'RavenMetadataCoreAssemblyName': 'NativeCore',
            'RavenNeoClrNativeCoreReference': '../NativeCore.dll',
            'RavenNeoClrRuntimeSeed': '../System.neox',
        }.items():
            ET.SubElement(group, name).text = value
        reference = ET.SubElement(ET.SubElement(root, 'ItemGroup'), 'Reference', Include='Input')
        ET.SubElement(reference, 'HintPath').text = '../Input.dll'
        if args.unions:
            reference = ET.SubElement(ET.SubElement(root, 'ItemGroup'), 'Reference', Include='ProductionValues')
            ET.SubElement(reference, 'HintPath').text = '../ProductionValues.dll'
        ET.ElementTree(root).write(project, encoding='unicode')
        command = ['dotnet', args.driver.resolve(), 'neoclr', '--project', project]
        run(command)
        consumer = directory / 'bin/neoclr/App.dll'
        run([*command, '--run', runtime], expected=42)
        before = consumer.read_bytes()
        invalid = ET.SubElement(group, 'RavenNeoClrCoreReference')
        invalid.text = '../NativeCore.dll'
        ET.ElementTree(root).write(project, encoding='unicode')
        run(command, expected=1)
        if consumer.read_bytes() != before:
            raise AssertionError('Rejected mixed project changed its published output')
        group.remove(invalid)
        ET.ElementTree(root).write(project, encoding='unicode')
        report['project'] = {'nativeOnly': True, 'runExit': 42, 'mixedSelectionPreservedOutput': True}
    if args.value_types or args.text_services:
        rejected = artifacts / 'ProductionUnions.dll'
        diagnostic = run(['dotnet', args.driver.resolve(), 'neoclr', '--native-core-reference', core,
            '--library', '-o', rejected, *production_sources], expected=1, include_stderr=True)
        expected_diagnostic = ('NEOMETA001', 'class UnionAttribute') if args.text_services else ('RAV1501', 'String.Concat')
        if not all(part in diagnostic for part in expected_diagnostic) or rejected.exists():
            raise AssertionError('Union core frontier did not reject cleanly before publication: ' + diagnostic)
        report['importedBaseRejection' if args.unions else 'productionUnionFrontier'] = {'diagnostic': diagnostic.strip(),
            'outputPublished': False, 'sourcesUnchanged': True}
    dependencies = ['--module', core, '--module', library, '--system', seed, '--object-root', core]
    if args.unions:
        dependencies += ['--module', union_library]
    interpreted = run([runtime, 'run', consumer, *dependencies, '--show-result'], expected=42, include_stderr=True)
    if interpreted.strip() != '=> Int32(42)':
        raise AssertionError(interpreted)
    if args.union_display:
        rejected_object = artifacts / 'unbound-display.o'
        diagnostic = run([aot, '--closed-world', consumer, '@entry', rejected_object, *dependencies,
            '--compile-system', '--reference-arena', '--bind-utf8-text'], expected=1, include_stderr=True)
        if 'boxed Int32 display requires --bind-int32-to-string' not in diagnostic or rejected_object.exists():
            raise AssertionError('Missing formatter did not reject before publication: ' + diagnostic)
        report['missingIntegerFormatter'] = {'diagnostic': diagnostic.strip(), 'outputPublished': False}
    selection = run([aot, '--closed-world', consumer, '@entry', artifacts / 'consumer.o', *dependencies,
        *(['--compile-system', '--reference-arena', '--bind-utf8-text'] if args.text_services else []),
        *(['--bind-int32-to-string'] if args.union_display else [])])
    (output / 'selection.json').write_text(selection)
    adapters = [HERE / 'text-host.c', HERE.parent / 'aot-console/text-arena.c'] if args.text_services else [HERE / 'host.c']
    run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', *adapters, artifacts / 'consumer.o', '-o', artifacts / 'consumer'])
    native = run([artifacts / 'consumer'])
    if native.strip() != '42':
        raise AssertionError(native)
    linked = run(['otool', '-L', artifacts / 'consumer'])
    dependencies_found = [line.strip().split(' (', 1)[0] for line in linked.splitlines()[1:]]
    if dependencies_found != ['/usr/lib/libSystem.B.dylib']:
        raise AssertionError(dependencies_found)
    report['result'] = {'interpreter': 42, 'native': 42, 'nativeLibraries': dependencies_found}
    if args.text_services:
        report['scope'] = 'Native-only fixture UTF-8 static and instance wrappers; unchanged production union emission frontier. Not production Object or String completeness.'
        report['textChecks'] = ['Unicode concatenation', 'empty left/right', 'embedded NUL', 'ordinal inequality', 'instance UTF-8 byte count']
    if args.unions:
        report['scope'] = 'Unchanged production Propagatable/Option/Result/Attribute/UnionAttribute over native-only fixture core; imported-base rejection retained.'
        report.pop('textChecks', None)
        report['unionChecks'] = ['Some/None', 'Ok/Error', 'let-else success/failure', 'if-let match/mismatch/else', 'UTF-8 and NUL error payload', 'production TryGetOutput/TryGetResidual including mismatched output reset']
    if args.string_boxing:
        report['scope'] = 'Native-only generic String boxing, Object display/reference identity and unrelated unboxed production union construction.'
        report.pop('unionChecks', None)
        report['boxingChecks'] = ['generic String-to-Object', 'Unicode/NUL display', 'alias identity', 'distinct equal-content identity', 'unboxed value construction excluded from Object dispatch']
    if args.union_display:
        report['scope'] = 'Native-only production Option/Result String and Int32 display over fixture core; explicit integer formatter. No Char box producers, escaped strings or general Object display qualification.'
        report.pop('unionChecks', None)
        report['displayChecks'] = ['Some Unicode/NUL', 'None', 'Ok String', 'Error String', 'Some Int32 zero/negative/min/max', 'Ok Int32', 'Error Int32', 'missing formatter rejection']
    report['revisions'] = {name: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
                           for name, path in [('neoclr', ROOT), ('raven', args.raven)]}
    inputs = [runtime, aot, *[p for p in HERE.iterdir() if p.suffix in ('.cs', '.csproj', '.rvn', '.neoil', '.c', '.py')]]
    inputs += list((HERE / 'bin/Debug/net10.0').glob('*.dll'))
    inputs += [core, library, consumer, seed, artifacts / 'consumer', *production_sources, *adapters]
    if args.unions:
        inputs += [union_library, attribute_source]
    if args.driver:
        inputs += list(args.driver.resolve().parent.glob('*.dll'))
    if args.project:
        inputs += [project, directory / 'Main.rvn']
    report['sha256'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS native-only core consumer: interpreter=42, native=42, libSystem only')


if __name__ == '__main__':
    main()
