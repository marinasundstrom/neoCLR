"""Compare existing Raven sample outputs through source, JSON and native binary loading.

This exercises the legacy Raven CLI bridge, not the new direct metadata emitter.
Use a fresh output directory; retain failures and tool hashes in the report.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from collection_library import build, ROOT

MATCH_OUTPUTS = {
    'Positional': '42\n-1\n', 'NominalDeconstruction': '42\n',
    'CaseImports': '42\n-1\n', 'OptionPositional': '42\n-1\n',
    'PositionalSingleEvaluation': '7\n42\n',
    'Forms': '42\n-1\nPresent\nAbsent\n42\nOverflow\n',
    'Guard': '1\n2\n3\n', 'StatementTail': '42\n-1\n',
    'SingleEvaluation': '7\n42\n', 'VoidOutput': '42\nSaved\nCompleted\nOverflow\n',
    'ExpressionBlockReturn': '42\n',
}
# Expectations from verify_project.py, covering ordinary library calls and floating operands.
MATCH_REJECTIONS = {
    'PositionalWrongArity': 'RAV2106', 'StatementReturn': 'RAV1503',
    'MissingExpression': 'RAV2100', 'MissingStatement': 'RAV2100',
    'UnreachableArm': 'RAV2101', 'WrongCase': 'RAV2102',
}
SAMPLES = {
    'Basics': ('library-basics.rvn', '42\n1\n0\nLibrary calls from Raven\n'),
    'Strings': ('library-strings.rvn', 'Hello, värld!\n14\nyes\nno\nyes\nno\nyes\nyes\nyes\nno\n-1\n0\n1\n'),
    'FloatingMath': ('library-floating-math.rvn', '0\n' * 19 + '-1\n'),
    'ValueCopy': ('library-value-copy.rvn', '42\n7\n'),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bridge', required=True, type=Path)
    parser.add_argument('--translator', required=True, type=Path)
    parser.add_argument('--runtime', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--only', nargs='+', choices=[*MATCH_OUTPUTS, *SAMPLES], help='Run selected positive cases; still check all compiler rejections')
    parser.add_argument('--system-json', type=Path, help='Reuse an assembled matching Raven collection profile')
    args = parser.parse_args()
    bridge, translator, runtime = (p.resolve() for p in (args.bridge, args.translator, args.runtime))
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {'pipeline': 'Raven source -> legacy CLI bridge -> neoil -> JSON -> NEOX',
              'directNativeCompilerEmission': False, 'cases': {}, 'tools': {}}
    for name, path in [('bridge', bridge), ('compiler', bridge.parent / 'Raven.CodeAnalysis.dll'),
                       ('translator', translator), ('metadata', translator.parent / 'NeoCLR.Metadata.Experimental.dll'),
                       ('runtime', runtime)]:
        report['tools'][name] = hashlib.sha256(path.read_bytes()).hexdigest()

    def run(*command):
        try:
            process = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120, cwd=ROOT)
        except subprocess.TimeoutExpired as error:
            raise RuntimeError('process exceeded 120 second limit') from error
        if process.returncode:
            raise RuntimeError(f'{process.returncode}: {process.stdout}{process.stderr}')
        return process.stdout

    def translate(source, target):
        run('dotnet', translator, source, target)

    def check(name, source, expected):
        result = report['cases'][name] = {}
        phase = 'source'
        try:
            run(runtime, 'verify', source, '--system', system_json)
            actual = run(runtime, 'run', source, '--system', system_json)
            if actual != expected:
                raise RuntimeError(f'expected {expected!r}, got {actual!r}')
            result['source'] = True
            phase = 'json'
            native = output / (name + '.json')
            run(runtime, 'assemble', source, native, '--system', system_json)
            run(runtime, 'verify', native, '--system', system_json)
            if run(runtime, 'run', native, '--system', system_json) != expected:
                raise RuntimeError('JSON output differs')
            result['json'] = True
            phase = 'translation'
            binary = output / (name + '.neox')
            translate(native, binary)
            result['translation'] = True
            phase = 'binary'
            run(runtime, 'verify', binary, '--system', system_binary)
            if run(runtime, 'run', binary, '--system', system_binary) != expected:
                raise RuntimeError('binary output differs')
            result.update(binary=True, stdout=expected, jsonBytes=native.stat().st_size,
                          binaryBytes=binary.stat().st_size,
                          binarySha256=hashlib.sha256(binary.read_bytes()).hexdigest())
        except RuntimeError as error:
            result.update(failedPhase=phase, error=str(error))
        print(name + ': ' + ('PASS' if result.get('binary') else result['failedPhase']), flush=True)

    try:
        system_json, system_binary = output / 'System.json', output / 'System.neox'
        if args.system_json:
            shutil.copyfile(args.system_json.resolve(), system_json)
        else:
            system_source = output / 'System.neoil'
            system_source.write_text(build(ROOT / 'runtime/System.neoil'))
            run(runtime, 'assemble', system_source, system_json)
        system_model = json.loads(system_json.read_text())
        report['system'] = {'profile': 'raven-collections', 'types': len(system_model['types']),
                            'functions': len(system_model['functions']),
                            'compactJsonBytes': len(json.dumps(system_model, separators=(',', ':'), ensure_ascii=False).encode()),
                            'jsonBytes': system_json.stat().st_size,
                            'jsonSha256': hashlib.sha256(system_json.read_bytes()).hexdigest()}
        try:
            translate(system_json, system_binary)
            run(runtime, 'verify', system_binary)
            report['system'].update(binaryBytes=system_binary.stat().st_size,
                                    binarySha256=hashlib.sha256(system_binary.read_bytes()).hexdigest())
        except RuntimeError as error:
            report['system'].update(failedPhase='translation/verification', error=str(error),
                                    applicationTestsUseJsonSystem=True)
            system_binary = system_json
        matches = output / 'matches'
        run('dotnet', bridge, '--matches', matches)
        compiled = json.loads((matches / 'match-results.json').read_text())
        report['matchCompilation'] = compiled
        if set(compiled) != set(MATCH_OUTPUTS) | set(MATCH_REJECTIONS):
            raise RuntimeError('review expected match matrix after a probe change')
        for name, diagnostic in MATCH_REJECTIONS.items():
            result = compiled[name]
            if (result['Stage'] != 'compile-rejected' or
                    diagnostic not in [item['Id'] for item in result['Diagnostics']] or
                    (matches / (name + '.neoil')).exists()):
                raise RuntimeError('unexpected compile rejection result: ' + name)
        report['expectedMatchRejections'] = len(MATCH_REJECTIONS)
        for name, expected in MATCH_OUTPUTS.items():
            if args.only and name not in args.only:
                continue
            if compiled[name]['Stage'] != 'imported':
                report['cases'][name] = {'failedPhase': 'compiler/importer', 'diagnostics': compiled[name]}
                continue
            check(name, matches / (name + '.neoil'), expected)
        for name, (filename, expected) in SAMPLES.items():
            if args.only and name not in args.only:
                continue
            project = output / name
            project.mkdir()
            shutil.copyfile(matches / 'NeoCLR.CoreProbe.dll', project / 'NeoCLR.CoreProbe.dll')
            shutil.copyfile(Path(__file__).parent / 'samples' / filename, project / 'Main.rvn')
            (project / 'Demo.rvnproj').write_text('''<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net11.0</TargetFramework><OutputType>Exe</OutputType>
    <RavenMetadataCoreAssemblyName>NeoCLR.CoreProbe</RavenMetadataCoreAssemblyName>
    <RavenTargetCoreAssemblyName>NeoCLR.CoreProbe</RavenTargetCoreAssemblyName>
    <RavenUseHostFrameworkReferences>false</RavenUseHostFrameworkReferences>
    <RavenPropagationAssemblyName>NeoCLR.CoreProbe</RavenPropagationAssemblyName>
    <RavenPropagationInterfaceType>System.Propagatable`3</RavenPropagationInterfaceType>
    <RavenIterationAssemblyName>NeoCLR.CoreProbe</RavenIterationAssemblyName>
    <RavenIterationArrayShapeType>System.Array`1</RavenIterationArrayShapeType>
    <RavenAllowArrayCovariance>false</RavenAllowArrayCovariance>
    <RavenIterationIterableType>System.Collections.Iterable`1</RavenIterationIterableType>
    <RavenIterationIteratorType>System.Collections.Iterator`1</RavenIterationIteratorType>
    <ImplicitImports>disable</ImplicitImports><RavenFrameworkProjections>None</RavenFrameworkProjections>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup><Compile Include="Main.rvn" />
    <Reference Include="NeoCLR.CoreProbe"><HintPath>NeoCLR.CoreProbe.dll</HintPath></Reference>
  </ItemGroup>
</Project>''')
            try:
                run('dotnet', bridge, '--project', project / 'Demo.rvnproj', project / 'compiled')
                mapping = json.loads((project / 'compiled/App.neoil.map.json').read_text())
                if mapping['RequiredLibraryProfile'] != 'raven-collections':
                    raise RuntimeError('project Runtime Contract selected an unexpected library profile')
            except RuntimeError as error:
                report['cases'][name] = {'failedPhase': 'compiler/importer', 'error': str(error)}
                continue
            check(name, project / 'compiled/App.neoil', expected)
    finally:
        report['passed'] = sum(bool(case.get('binary')) for case in report['cases'].values())
        report['failed'] = sum('failedPhase' in case for case in report['cases'].values())
        (output / 'translation-results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f"{report['passed']} passed; {report['failed']} failures (see translation-results.json)")
    return 1 if report['failed'] or 'failedPhase' in report.get('system', {}) else 0


if __name__ == '__main__':
    raise SystemExit(main())
