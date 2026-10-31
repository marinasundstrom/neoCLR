"""Rebuild HTTP header collection libraries over a matching qualified development bundle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--base-bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
bundle = out / 'bundle'
shutil.copytree(args.base_bundle.resolve(), bundle)
lib = bundle / 'lib'
env = dict(os.environ, NeoClrBundleRoot=str(bundle))
components = [
    ('System.Web', 'runtime/System.Web/src/System.Web', 'Neo.*;System.Web'),
    ('Neo.Web', 'extensions/Neo.Web/src/Neo.Web', 'Neo.Web'),
]
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
evidence = json.loads((lib / 'build-evidence.json').read_text())
for name, source, remove in components:
    work = out / name
    work.mkdir()
    project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
    ET.SubElement(project, 'Import', Project=str(lib / 'NeoCLR.ClassLibrary.props'))
    properties = ET.SubElement(project, 'PropertyGroup')
    for key, value in dict(TargetFramework='net10.0', OutputType='Library',
                           AssemblyName=name, EnableDefaultCompileItems='false',
                           RavenNeoClrRuntimeAsyncHelper='').items():
        ET.SubElement(properties, key).text = value
    items = ET.SubElement(project, 'ItemGroup')
    ET.SubElement(items, 'Reference', Remove=remove)
    ET.SubElement(items, 'Compile', Include=str(ROOT / source / '**/*.rvn'))
    ET.indent(project)
    path = work / (name + '.rvnproj')
    ET.ElementTree(project).write(path, encoding='unicode')
    command = ['dotnet', str(bundle / 'sdk/tools/rvnc/rvnc.dll'), 'neoclr', '--project', str(path)]
    subprocess.run(command, env=env, check=True)
    for suffix in ('.dll', '.xml', '.docs'):
        built = work / 'bin/neoclr' / (name + suffix)
        if built.is_dir():
            shutil.copytree(built, lib / built.name, dirs_exist_ok=True)
        else:
            shutil.copy2(built, lib / built.name)
    evidence.setdefault('componentBuilds', []).append(dict(
        assembly=name, command=command, exitCode=0,
        compilerSha256=sha(bundle / 'sdk/tools/rvnc/rvnc.dll'),
        sources={path.relative_to(ROOT).as_posix(): sha(path) for path in (ROOT / source).rglob('*.rvn')},
        artifacts={name + suffix: sha(lib / (name + suffix)) for suffix in ('.dll', '.xml')}))
(lib / 'build-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
for directory, filename in ((lib, 'bundle.json'), (bundle, 'manifest.json')):
    path = directory / filename
    catalog = json.loads(path.read_text())
    catalog['files'] = {file.relative_to(directory).as_posix(): sha(file)
                        for file in directory.rglob('*') if file.is_file() and file != path}
    path.write_text(json.dumps(catalog, indent=2) + '\n')
print(bundle)
