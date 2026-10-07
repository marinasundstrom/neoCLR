#!/usr/bin/env python3
"""Qualify project compilation/execution against separate source-built native libraries."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'runtime-library-directory', 'data', 'networking', 'web', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    directory = args.runtime_library_directory.resolve()
    dependencies = [directory / 'System.Runtime.dll', args.data.resolve(), args.networking.resolve(), args.web.resolve()]
    seed = directory / 'System.runtime.neox'
    ownership = directory / 'ownership.json'
    source = ROOT / 'docs/experiments/http-headers/Main.rvn'
    shutil.copyfile(source, output / 'Main.rvn')
    project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
    properties = ET.SubElement(project, 'PropertyGroup')
    for name, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='Headers',
                           RavenTargetPlatform='NeoCLR', RavenMetadataFormat='NeoCLR',
                           RavenNeoClrCoreReference=str(args.core.resolve()), RavenNeoClrRuntimeSeed=str(seed),
                           RavenNeoClrBootstrapOwnership=str(ownership), RavenNeoClrObjectLibrary='System.Runtime').items():
        ET.SubElement(properties, name).text = value
    items = ET.SubElement(project, 'ItemGroup')
    for dependency in dependencies:
        reference = ET.SubElement(items, 'Reference', Include=dependency.stem)
        ET.SubElement(reference, 'HintPath').text = str(dependency)
    project_path = output / 'Headers.rvnproj'
    ET.ElementTree(project).write(project_path, encoding='unicode')
    report = dict(compilerRevision=args.compiler_revision, sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[])

    def run(command):
        result = subprocess.run(list(map(str, command)), capture_output=True, text=True, cwd=ROOT, timeout=180)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
        return result

    command = ['dotnet', args.compiler.resolve(), 'neoclr', '--project', project_path]
    result = run(command + ['--run', args.runtime.resolve()])
    artifact = output / 'bin/neoclr/Headers.dll'
    expected = f'Native build output: {artifact}\nHTTP header lookup checks passed\n'
    if result.returncode != 0 or result.stdout != expected:
        raise RuntimeError('Native project execution failed: ' + result.stdout + result.stderr)
    before = artifact.read_bytes()
    properties.find('RavenNeoClrObjectLibrary').text = 'Missing.Owner'
    ET.ElementTree(project).write(project_path, encoding='unicode')
    result = run(command)
    if result.returncode == 0 or 'exactly one registered native reference' not in result.stderr or artifact.read_bytes() != before:
        raise RuntimeError('Invalid Object ownership did not reject before output replacement')
    properties.find('RavenNeoClrObjectLibrary').text = 'System.Runtime'
    ET.ElementTree(project).write(project_path, encoding='unicode')
    paths = [Path(__file__), source, project_path, artifact, args.compiler, args.core, args.runtime, seed, ownership, *dependencies]
    paths += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report['hashes'] = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Native project execution and failed-output preservation passed')


if __name__ == '__main__':
    main()
