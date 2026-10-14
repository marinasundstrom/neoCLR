#!/usr/bin/env python3
"""Compile and execute the native JSON mapping error-composition sample."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('library', 'compiler', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    lib, compiler, runtime, output = (getattr(args, n).resolve() for n in ('library', 'compiler', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    catalog = json.loads((lib / 'bundle.json').read_text())
    modules = [lib / (name + '.dll') for name in catalog['assemblyNames']]
    context = ['--system', lib / catalog['runtimeSeed'], '--object-root', lib / 'System.Runtime.dll']
    for module in modules:
        context += ['--module', module]
    inputs = [Path(__file__), compiler, runtime, lib / 'bundle.json', lib / catalog['runtimeSeed'],
              lib / 'NeoCLR.ClassLibrary.props', lib / 'RuntimeAwait.rvn', lib / 'RuntimeGenerator.rvn',
              compiler.parent / 'Raven.CodeAnalysis.dll', compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll', *modules]
    report = dict(passed=False, commands=[], inputs={}, scope='native metadata and interpreter JSON mapping error composition')

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=output, capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    def check(name, sources, expected=None):
        directory = output / name
        directory.mkdir()
        root = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
        ET.SubElement(root, 'Import', Project=str(lib / 'NeoCLR.ClassLibrary.props'))
        properties = ET.SubElement(root, 'PropertyGroup')
        for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName=name,
                               EnableDefaultCompileItems='false').items():
            ET.SubElement(properties, key).text = value
        items = ET.SubElement(root, 'ItemGroup')
        for source in sources:
            inputs.append(source)
            ET.SubElement(items, 'Compile', Include=str(source))
        project = directory / (name + '.rvnproj')
        ET.ElementTree(root).write(project, encoding='unicode')
        run(['dotnet', compiler, 'neoclr', '--project', project])
        artifact = directory / 'bin/neoclr' / (name + '.dll')
        inputs.append(artifact)
        run([runtime, 'verify', artifact, *context])
        if expected is not None:
            result = run([runtime, 'run', artifact, '--execution', 'interpreter', '--instructions', '100000000', *context])
            assert result.stdout == expected and not result.stderr, result.stdout + result.stderr

    try:
        check('JsonMapping', [HERE / 'Mapping.rvn', HERE / 'Main.rvn'], 'JSON object mapping checks passed\n')
        report['passed'] = True
    finally:
        report['inputs'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs if p.is_file()}
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS native JSON mapping error composition')


if __name__ == '__main__':
    main()
