#!/usr/bin/env python3
"""Build the native class-library projects from source-owned native core contracts and stage a coherent bundle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
def project_directory(name):
    if name.startswith('Neo.'):
        return ROOT / 'extensions' / '.'.join(name.split('.')[:2]) / 'src' / name
    return ROOT / 'runtime' / name / 'src' / name
NAMES = ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web', 'Neo.DependencyInjection.Abstractions', 'Neo.DependencyInjection', 'Neo.Localization.Abstractions', 'Neo.Localization', 'Neo.Logging.Abstractions', 'Neo.Logging', 'Neo.Configuration.Abstractions', 'Neo.Configuration', 'Neo.Configuration.Json')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    parser.add_argument('--with-postgresql', action='store_true', help='Include the optional local PostgreSQL development provider')
    args = parser.parse_args()
    names = NAMES + (('System.Data.PostgreSql',) if args.with_postgresql else ())
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    seed = output / 'System.runtime.neox'
    environment = dict(os.environ)
    # Inherited bootstrap settings must not contaminate a native-only build.
    for name in ('RavenNeoClrCoreReference', 'RavenNeoClrBootstrapSeed',
                 'RavenNeoClrBootstrapOwnership', 'RavenNeoClrBootstrapIntrinsics',
                 'RavenNeoClrSourceObjectRoot', 'RavenNeoClrObjectLibrary',
                 'RavenNeoClrRuntimeSeed'):
        environment.pop(name, None)
    report = dict(compilerRevision=args.compiler_revision, sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[], output=str(output))

    def run(command):
        result = subprocess.run(list(map(str, command)), env=environment, cwd=ROOT, capture_output=True, text=True, timeout=300)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'build-evidence.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    def artifact(name):
        return project_directory(name) / 'bin/neoclr' / (name + '.dll')

    run(['dotnet', args.compiler.resolve(), 'neoclr', '--project', project_directory('System.Runtime') / 'System.Runtime.rvnproj'])
    runtime_hash = sha(artifact('System.Runtime'))
    # The reserved System module is empty; all core definitions belong to System.Runtime.
    seed_source = output / 'System.runtime.neoil'
    seed_source.write_text('.module System\n')
    run([args.runtime.resolve(), 'assemble', seed_source, seed, '--format', 'neox'])
    # Build ordering is explicit; imports still validate each native reference.
    for name in names[1:]:
        run(['dotnet', args.compiler.resolve(), 'neoclr', '--project', project_directory(name) / (name + '.rvnproj'), '--no-build-references'])
    if sha(artifact('System.Runtime')) != runtime_hash:
        raise RuntimeError('Runtime changed during dependent project builds; finalized seed cannot be published')
    documentation = {}
    for name in names:
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

    shutil.copyfile(project_directory('System.Runtime') / 'ownership.json', output / 'ownership.json')
    configuration = ET.Element('Project')
    properties = ET.SubElement(configuration, 'PropertyGroup')
    for name, value in dict(RavenTargetPlatform='NeoCLR', RavenMetadataFormat='NeoCLR',
                           RavenNeoClrNativeCoreReference='$(MSBuildThisFileDirectory)System.Runtime.dll',
                           RavenNeoClrRuntimeSeed='$(MSBuildThisFileDirectory)System.runtime.neox',
                           RavenNeoClrRuntimeContracts='$(MSBuildThisFileDirectory)ownership.json',
                           RavenNeoClrAsyncLibrary='System.Runtime').items():
        ET.SubElement(properties, name).text = value
    # SDK-owned, assembly-private source bridge. An explicit empty helper property
    # after this import retains compiler state-machine lowering.
    ET.SubElement(properties, 'RavenNeoClrRuntimeAsyncHelper',
                  Condition="'$(RavenNeoClrRuntimeAsyncHelper)' == ''").text = 'RuntimeAwaitProvider.Bridge'
    ET.SubElement(properties, 'RavenNeoClrRuntimeAsyncProviderImported').text = 'true'
    providers = [output / name for name in ('RuntimeAwait.rvn', 'RuntimeGenerator.rvn')]
    items = ET.SubElement(configuration, 'ItemGroup')
    for provider in providers:
        shutil.copyfile(ROOT / 'runtime/raven/compiler-services' / provider.name, provider)
        ET.SubElement(items, 'Compile', Include='$(MSBuildThisFileDirectory)' + provider.name,
                      Condition="'$(RavenNeoClrRuntimeAsyncHelper)' == 'RuntimeAwaitProvider.Bridge'")
    references = ET.SubElement(configuration, 'ItemGroup')
    for name in names:
        reference = ET.SubElement(references, 'Reference', Include=name)
        ET.SubElement(reference, 'HintPath').text = '$(MSBuildThisFileDirectory)' + name + '.dll'
    configuration_path = output / 'NeoCLR.ClassLibrary.props'
    ET.indent(configuration)
    ET.ElementTree(configuration).write(configuration_path, encoding='unicode')
    artifacts = [output / (name + '.dll') for name in names] + [seed, output / 'ownership.json', configuration_path] + providers
    for name in names:
        artifacts.append(output / documentation[name]['xml'])
        artifacts.extend(sorted((output / documentation[name]['markdown']).rglob('*')))
    artifacts = [path for path in artifacts if path.is_file()]
    # Publish the manifest last. An interrupted/failed build directory is not a completed bundle.
    manifest = dict(version=1, kind='native-source-core-class-library', assemblyNames=list(names),
                    objectAssembly='System.Runtime', runtimeSeed=seed.name, projectConfiguration=configuration_path.name,
                    documentation=documentation,
                    files={path.relative_to(output).as_posix(): sha(path) for path in artifacts})
    inputs = [Path(__file__), args.compiler, args.runtime]
    inputs += [ROOT / 'runtime/NativeLibrary.props', project_directory('System.Runtime') / 'ownership.json']
    for name in names:
        directory = project_directory(name)
        inputs.append(directory / (name + '.rvnproj'))
        inputs += list((directory / ('Neo' if name.startswith('Neo.') else 'System')).rglob('*.rvn'))
        inputs += list((directory / 'neoCLR').rglob('*.rvn'))
    inputs += list((ROOT / 'runtime/raven/compiler-services').glob('*.rvn'))
    inputs += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report['hashes'] = {str(path.resolve()): sha(path) for path in inputs + artifacts}
    (output / 'build-evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    (output / 'bundle.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print('Built native ' + '/'.join(names) + ' bundle: ' + str(output))


if __name__ == '__main__':
    main()
