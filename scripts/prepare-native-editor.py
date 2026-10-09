#!/usr/bin/env python3
"""Prepare an explicit native VS Code project from already validated bootstrap artifacts."""
import argparse
import json
from pathlib import Path
import shutil
import xml.etree.ElementTree as X

p = argparse.ArgumentParser(description=__doc__)
tooling = p.add_mutually_exclusive_group(required=True)
tooling.add_argument('--raven', type=Path, help='Development Raven checkout')
tooling.add_argument('--sdk', type=Path, help='Extracted native-enabled Raven SDK')
p.add_argument('--installed-editor', action='store_true', help='Use the installed extension server without a development override')
for name in ('runtime', 'output'):
    p.add_argument('--' + name, type=Path, required=True)
for name in ('core', 'seed', 'ownership', 'bundle'):
    p.add_argument('--' + name, type=Path)
p.add_argument('--reference', type=Path, action='append')
a = p.parse_args()
if a.bundle:
    if any((a.core, a.seed, a.ownership, a.reference)):
        p.error('--bundle cannot be combined with individual bootstrap/reference options')
elif not all((a.core, a.seed, a.ownership, a.reference)):
    p.error('Supply --bundle or all of --core, --seed, --ownership and --reference')
root = Path(__file__).resolve().parents[1]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
refs = out / 'references'
refs.mkdir()
if a.bundle:
    shutil.copytree(a.bundle.resolve(), refs, dirs_exist_ok=True)
else:
    for source, name in [(a.core, 'Core.dll'), (a.seed, 'System.neox'), (a.ownership, 'ownership.json')]:
        shutil.copyfile(source, refs / name)
    for source in a.reference:
        shutil.copyfile(source, refs / source.name)
        xml = source.with_suffix('.xml')
        markdown = source.with_suffix('.docs')
        if xml.is_file():
            shutil.copyfile(xml, refs / xml.name)
        if markdown.is_dir():
            shutil.copytree(markdown, refs / markdown.name)
shutil.copyfile(root / 'docs/experiments/raven-target/samples/application-order-collections.rvn', out / 'Main.rvn')
shutil.copyfile(root / 'docs/experiments/raven-target/samples/application-order-collections.expected.txt', out / 'expected.txt')
project = X.Element('Project', Sdk='Microsoft.NET.Sdk')
props = X.SubElement(project, 'PropertyGroup')
if a.bundle:
    project.insert(0, X.Element('Import', Project='references/NeoCLR.ClassLibrary.props'))
    for name, value in dict(TargetFramework='net10.0', OutputType='Exe').items():
        X.SubElement(props, name).text = value
    items = X.SubElement(project, 'ItemGroup')
else:
    for name, value in dict(TargetFramework='net10.0', OutputType='Exe',
        RavenTargetPlatform='NeoCLR', RavenMetadataFormat='NeoCLR',
        RavenNeoClrCoreReference='references/Core.dll', RavenNeoClrRuntimeSeed='references/System.neox',
        RavenNeoClrBootstrapOwnership='references/ownership.json', RavenNeoClrAsyncLibrary='Numbers',
        RavenNeoClrBootstrapIntrinsics='true').items():
        X.SubElement(props, name).text = value
    items = X.SubElement(project, 'ItemGroup')
    for ref in a.reference:
        X.SubElement(X.SubElement(items, 'Reference', Include=ref.stem), 'HintPath').text = 'references/' + ref.name
X.indent(project)
X.ElementTree(project).write(out / 'App.rvnproj', encoding='unicode')
if a.sdk:
    server = a.sdk.resolve() / 'tools/language-server/Raven.LanguageServer.dll'
    compiler = a.sdk.resolve() / 'tools/rvnc/rvnc.dll'
else:
    server = a.raven.resolve() / 'src/Raven.LanguageServer/bin/Debug/net10.0/Raven.LanguageServer.dll'
    compiler = a.raven.resolve() / 'src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll'
vscode = out / '.vscode'
vscode.mkdir()
editor_settings = {'raven.trace.server': 'verbose'}
if not a.installed_editor:
    editor_settings['raven.languageServerPath'] = str(server)
(vscode / 'settings.json').write_text(json.dumps(editor_settings, indent=2))
tasks = []
for name, tail in [('Build', []), ('Run', ['--run', str(a.runtime.resolve())])]:
    tasks.append(dict(label='neoCLR: ' + name, type='process', command='dotnet',
        args=[str(compiler), 'neoclr', '--project', '${workspaceFolder}/App.rvnproj'] + tail,
        group='build' if name == 'Build' else 'test', problemMatcher=[]))
(vscode / 'tasks.json').write_text(json.dumps({'version': '2.0.0', 'tasks': tasks}, indent=2))
(out / 'acceptance.json').write_text(json.dumps({'compiler': str(compiler), 'runtime': str(a.runtime.resolve()), 'classLibraryBundle': bool(a.bundle), 'bundleDocumentation': bool(a.bundle and json.loads((refs / 'bundle.json').read_text()).get('documentation'))}, indent=2))
print(out)

# Compile two versions of an ordinary library outside the application source root.
# Keeping those sources outside the workspace proves artifact-only consumption.
import subprocess
library_root = out.with_name(out.name + '-library')
for version in ('One', 'Two'):
    lib = library_root / version
    lib.mkdir(parents=True, exist_ok=False)
    (lib / 'Library.rvn').write_text('/// API documentation for the editor fixture.\npublic class EditorApi {\n    /// Returns the **documented answer**.\n    static func Version' + version + '() -> int => 42\n}\n')
    library = X.Element('Project', Sdk='Microsoft.NET.Sdk')
    if a.bundle:
        X.SubElement(library, 'Import', Project=str(refs / 'NeoCLR.ClassLibrary.props'))
    group = X.SubElement(library, 'PropertyGroup')
    if a.bundle:
        for key, value in dict(TargetFramework='net10.0', OutputType='Library').items():
            X.SubElement(group, key).text = value
    else:
        for key, value in dict(TargetFramework='net10.0', OutputType='Library', RavenTargetPlatform='NeoCLR',
            RavenMetadataFormat='NeoCLR', RavenNeoClrCoreReference=str(refs / 'Core.dll'),
            RavenTypeOfAssemblyName='', RavenTypeOfInfoType='', RavenTypeOfContextType='').items():
            X.SubElement(group, key).text = value
    X.ElementTree(library).write(lib / 'EditorLibrary.rvnproj', encoding='unicode')
    subprocess.run(['dotnet', str(compiler), 'neoclr', '--project', str(lib / 'EditorLibrary.rvnproj')], check=True)
shutil.copyfile(library_root / 'One/bin/neoclr/EditorLibrary.dll', refs / 'EditorLibrary.dll')
shutil.copyfile(library_root / 'One/bin/neoclr/EditorLibrary.xml', refs / 'EditorLibrary.xml')
shutil.copytree(library_root / 'One/bin/neoclr/EditorLibrary.docs', refs / 'EditorLibrary.docs')
X.SubElement(X.SubElement(items, 'Reference', Include='EditorLibrary'), 'HintPath').text = 'references/EditorLibrary.dll'
X.indent(project)
X.ElementTree(project).write(out / 'App.rvnproj', encoding='unicode')
(out / 'Probe.rvn').write_text('func EditorProbe() -> int => EditorApi.VersionOne()\n')
config = json.loads((out / 'acceptance.json').read_text())
config['replacementLibrary'] = str(library_root / 'Two/bin/neoclr/EditorLibrary.dll')
(out / 'acceptance.json').write_text(json.dumps(config, indent=2))

dotnet = out.with_name(out.name + '-dotnet')
dotnet.mkdir(exist_ok=False)
(dotnet / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType></PropertyGroup></Project>')
(dotnet / 'Main.rvn').write_text('func Main() -> int => System.Math.Abs(-42)\n')

async_root = out.with_name(out.name + '-async')
async_root.mkdir(exist_ok=False)
async_project = X.fromstring(X.tostring(project))
for hint in async_project.findall('.//HintPath'):
    hint.text = str(out / hint.text)
for name in ('RavenNeoClrCoreReference', 'RavenNeoClrRuntimeSeed', 'RavenNeoClrBootstrapOwnership'):
    element = async_project.find('PropertyGroup/' + name)
    if element is not None:
        element.text = str(out / element.text)
if a.bundle:
    async_project.find('Import').set('Project', str(refs / 'NeoCLR.ClassLibrary.props'))
X.ElementTree(async_project).write(async_root / 'Async.rvnproj', encoding='unicode')
shutil.copyfile(root / 'docs/experiments/raven-target/samples/library-async.rvn', async_root / 'Main.rvn')
(async_root / 'expected.txt').write_text('Suspended\n42\n')
workspace = out.with_suffix('.code-workspace')
workspace.write_text(json.dumps({'folders': [{'name': name, 'path': str(folder)}
    for name, folder in [('Native', out), ('DotNet', dotnet), ('Async', async_root)]],
    'settings': editor_settings}, indent=2))
print(workspace)
