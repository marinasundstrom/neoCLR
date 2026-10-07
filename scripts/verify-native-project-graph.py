#!/usr/bin/env python3
"""Build and execute a native project diamond; verify rejection preserves outputs."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'runtime-library-directory', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    library = args.runtime_library_directory.resolve()
    sources = {
        'Base': '''namespace Graph
public class Box<T> {
    public field Value: T
    public init(value: T) { Value = value }
}
''',
        'Left': '''import Graph.*
public func Create() -> Box<int> => Box<int>(41)
''',
        'Right': '''import Graph.*
public func Bump(box: Box<int>) { box.Value = box.Value + 1 }
''',
        'App': '''import System.*
func Main() -> int {
    let box = Create()
    let alias = box
    Bump(alias)
    if box.Value != 42 { return 1 }
    Console.WriteLine("Native project graph passed")
    return 0
}
'''}
    dependencies = {'Base': [], 'Left': ['Base'], 'Right': ['Base'], 'App': ['Left', 'Right']}
    projects = {}
    for name, source in sources.items():
        directory = output / name
        directory.mkdir()
        (directory / 'Main.rvn').write_text(source)
        project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
        properties = ET.SubElement(project, 'PropertyGroup')
        for key, value in dict(TargetFramework='net10.0', OutputType='Exe' if name == 'App' else 'Library',
                              AssemblyName=name, RavenTargetPlatform='NeoCLR', RavenMetadataFormat='NeoCLR',
                              RavenNeoClrCoreReference=str(args.core.resolve()),
                              RavenNeoClrRuntimeSeed=str(library / 'System.runtime.neox'),
                              RavenNeoClrBootstrapOwnership=str(library / 'ownership.json'),
                              RavenNeoClrObjectLibrary='System.Runtime').items():
            ET.SubElement(properties, key).text = value
        items = ET.SubElement(project, 'ItemGroup')
        if name == 'Base':
            reference = ET.SubElement(items, 'Reference', Include='System.Runtime')
            ET.SubElement(reference, 'HintPath').text = str(library / 'System.Runtime.dll')
        for dependency in dependencies[name]:
            ET.SubElement(items, 'ProjectReference', Include=f'../{dependency}/{dependency}.rvnproj')
        projects[name] = directory / f'{name}.rvnproj'
        ET.ElementTree(project).write(projects[name], encoding='unicode')
    report = dict(compilerRevision=args.compiler_revision, sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[])

    def run(prebuilt=False):
        command = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--project', str(projects['App']), '--run', str(args.runtime.resolve())]
        if prebuilt:
            command.append("--no-build-references")
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
        return result

    result = run()
    artifacts = [output / name / f'bin/neoclr/{name}.dll' for name in sources]
    expected = ''.join(f'Native build output: {path}\n' for path in artifacts) + 'Native project graph passed\n'
    if result.returncode or result.stdout != expected:
        raise RuntimeError(result.stdout + result.stderr)
    before = [path.read_bytes() for path in artifacts]
    base = projects['Base'].read_text()
    document = ET.parse(projects['Base'])
    ET.SubElement(document.getroot().find('ItemGroup'), 'ProjectReference', Include='../Left/Left.rvnproj')
    document.write(projects['Base'], encoding='unicode')
    result = run()
    if result.returncode == 0 or 'Cyclic project reference' not in result.stderr or before != [path.read_bytes() for path in artifacts]:
        raise RuntimeError('Cycle failed to reject before publication')
    projects['Base'].write_text(base)
    source_path = output / 'Base/Main.rvn'
    source = source_path.read_text()
    source_path.write_text('func Broken() -> int => MissingName\n')
    result = run()
    if result.returncode == 0 or before != [path.read_bytes() for path in artifacts]:
        raise RuntimeError('Dependency binding failure replaced outputs')
    result = run(prebuilt=True)
    if result.returncode != 0 or result.stdout != f'Native build output: {artifacts[-1]}\nNative project graph passed\n' or before[:-1] != [path.read_bytes() for path in artifacts[:-1]]:
        raise RuntimeError('Prebuilt dependency mode rebuilt sources or failed execution')
    saved = artifacts[0].read_bytes()
    artifacts[0].unlink()
    consumer_before = artifacts[-1].read_bytes()
    result = run(prebuilt=True)
    if result.returncode == 0 or artifacts[-1].read_bytes() != consumer_before:
        raise RuntimeError('Missing prebuilt dependency did not preserve consumer output')
    artifacts[0].write_bytes(saved)
    source_path.write_text(source)
    paths = [Path(__file__), args.compiler, args.runtime, args.core, *projects.values(), *artifacts]
    paths += list(output.glob('*/Main.rvn')) + [library / name for name in ('System.Runtime.dll', 'System.runtime.neox', 'ownership.json')]
    paths += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report['hashes'] = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Native project diamond, execution, cycle rejection and failed-output preservation passed')


if __name__ == '__main__':
    main()
