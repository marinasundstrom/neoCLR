#!/usr/bin/env python3
"""Build the native class-library projects with explicit bootstrap inputs and stage a coherent bundle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
PROJECTS = ROOT / 'runtime/raven/projects'
NAMES = ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'bootstrap-directory', 'translator', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    bootstrap = args.bootstrap_directory.resolve()
    bootstrap_manifest = bootstrap / 'bootstrap.json'
    if bootstrap_manifest.is_file():
        prepared = json.loads(bootstrap_manifest.read_text())
        if prepared.get('kind') != 'explicit-source-runtime-bootstrap':
            raise ValueError('Unsupported bootstrap preparation')
        for relative, expected in prepared['files'].items():
            path = (bootstrap / relative).resolve()
            if bootstrap not in path.parents or sha(path) != expected:
                raise ValueError('Prepared bootstrap hash mismatch: ' + relative)
        if sha(args.core) != prepared['files'][prepared['core']]:
            raise ValueError('Selected core differs from prepared bootstrap')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    seed = output / 'System.runtime.neox'
    environment = dict(os.environ, RavenNeoClrCoreReference=str(args.core.resolve()),
                       RavenNeoClrBootstrapSeed=str(bootstrap / 'System.neox'), RavenNeoClrRuntimeSeed=str(seed))
    report = dict(compilerRevision=args.compiler_revision, sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[],
        environment={name: environment[name] for name in ('RavenNeoClrCoreReference', 'RavenNeoClrBootstrapSeed', 'RavenNeoClrRuntimeSeed')})

    def run(command):
        result = subprocess.run(list(map(str, command)), env=environment, cwd=ROOT, capture_output=True, text=True, timeout=300)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'build-evidence.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    def artifact(name):
        return PROJECTS / name / 'bin/neoclr' / (name + '.dll')

    run(['dotnet', args.compiler.resolve(), 'neoclr', '--project', PROJECTS / 'System.Runtime/System.Runtime.rvnproj'])
    runtime_hash = sha(artifact('System.Runtime'))
    run(['dotnet', args.translator.resolve(), bootstrap / 'System.retained.json', seed, '--reference', artifact('System.Runtime')])
    # Bootstrap orchestration owns build ordering; the project loader still resolves
    # the full native reference graph and validates every prebuilt artifact.
    for name in NAMES[1:]:
        run(['dotnet', args.compiler.resolve(), 'neoclr', '--project', PROJECTS / name / (name + '.rvnproj'), '--no-build-references'])
    if sha(artifact('System.Runtime')) != runtime_hash:
        raise RuntimeError('Runtime changed during dependent project builds; finalized seed cannot be published')
    documentation = {}
    for name in NAMES:
        shutil.copyfile(artifact(name), output / (name + '.dll'))
        xml = artifact(name).with_suffix('.xml')
        markdown = artifact(name).with_suffix('.docs')
        if ET.parse(xml).findtext('./assembly/name') != name:
            raise RuntimeError('Documentation assembly mismatch: ' + str(xml))
        if not (markdown / 'manifest.json').is_file():
            raise RuntimeError('Missing generated Markdown documentation: ' + str(markdown))
        shutil.copyfile(xml, output / xml.name)
        shutil.copytree(markdown, output / markdown.name)
        documentation[name] = dict(xml=xml.name, markdown=markdown.name)

    shutil.copyfile(args.core.resolve(), output / 'Core.dll')
    shutil.copyfile(PROJECTS / 'System.Runtime/ownership.json', output / 'ownership.json')
    configuration = ET.Element('Project')
    properties = ET.SubElement(configuration, 'PropertyGroup')
    for name, value in dict(RavenTargetPlatform='NeoCLR', RavenMetadataFormat='NeoCLR',
                           RavenNeoClrCoreReference='$(MSBuildThisFileDirectory)Core.dll',
                           RavenNeoClrRuntimeSeed='$(MSBuildThisFileDirectory)System.runtime.neox',
                           RavenNeoClrBootstrapOwnership='$(MSBuildThisFileDirectory)ownership.json',
                           RavenNeoClrObjectLibrary='System.Runtime', RavenNeoClrAsyncLibrary='System.Runtime',
                           RavenNeoClrSourceObjectRoot='false', RavenNeoClrBootstrapIntrinsics='false').items():
        ET.SubElement(properties, name).text = value
    references = ET.SubElement(configuration, 'ItemGroup')
    for name in NAMES:
        reference = ET.SubElement(references, 'Reference', Include=name)
        ET.SubElement(reference, 'HintPath').text = '$(MSBuildThisFileDirectory)' + name + '.dll'
    configuration_path = output / 'NeoCLR.ClassLibrary.props'
    ET.indent(configuration)
    ET.ElementTree(configuration).write(configuration_path, encoding='unicode')
    artifacts = [output / (name + '.dll') for name in NAMES] + [seed, output / 'Core.dll', output / 'ownership.json', configuration_path]
    for name in NAMES:
        artifacts.append(output / documentation[name]['xml'])
        artifacts.extend(sorted((output / documentation[name]['markdown']).rglob('*')))
    artifacts = [path for path in artifacts if path.is_file()]
    # Publish the manifest last. An interrupted/failed build directory is not a completed bundle.
    manifest = dict(version=1, kind='explicit-bootstrap-native-class-library', assemblyNames=list(NAMES),
                    objectAssembly='System.Runtime', runtimeSeed=seed.name, projectConfiguration=configuration_path.name,
                    documentation=documentation,
                    files={path.relative_to(output).as_posix(): sha(path) for path in artifacts})
    inputs = [Path(__file__), args.compiler, args.core, args.translator, bootstrap / 'System.neox', bootstrap / 'System.retained.json']
    if bootstrap_manifest.is_file():
        inputs += [bootstrap_manifest, bootstrap / 'preparation-evidence.json']
    inputs += list(PROJECTS.glob('*/*.rvnproj')) + [PROJECTS / 'NativeLibrary.props', PROJECTS / 'System.Runtime/ownership.json']
    inputs += list((ROOT / 'runtime/raven/src/System').rglob('*.rvn')) + list((ROOT / 'runtime/raven/native').glob('*.rvn'))
    inputs += [args.translator.parent / 'NeoCLR.Metadata.Experimental.dll']
    inputs += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report['hashes'] = {str(path.resolve()): sha(path) for path in inputs + artifacts}
    (output / 'build-evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    (output / 'bundle.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print('Built native Runtime/Data/Networking/Web bundle: ' + str(output))


if __name__ == '__main__':
    main()
