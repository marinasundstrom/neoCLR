#!/usr/bin/env python3
"""Stage editable VS Code samples around explicitly published experimental tools."""
import argparse
import json
import shutil
from pathlib import Path

root = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--library', type=Path, required=True)
args = parser.parse_args()
kit = args.output.resolve()
kit.mkdir(parents=True, exist_ok=True)
shutil.copytree(args.library, kit / 'library', dirs_exist_ok=True)
(kit / 'runtime').mkdir(exist_ok=True)
shutil.copy2(root / 'target/debug/neoclr', kit / 'runtime/neoclr')
for name, source in [('Unions', 'source-structural-unions'), ('Intersections', 'source-structural-intersections')]:
    folder = kit / name
    folder.mkdir(exist_ok=True)
    shutil.copy2(root / 'docs/experiments' / source / 'Main.rvn', folder / 'Main.rvn')
folder = kit / 'ExternalBounds'
folder.mkdir(exist_ok=True)
shutil.copy2(Path(__file__).with_name('ExternalBounds.rvn'), folder / 'Main.rvn')
contracts = kit / 'Contracts'
contracts.mkdir(exist_ok=True)
(contracts / 'Main.rvn').write_text('''public interface Reads<T> {
    func Read() -> T
}

public interface Adds {
    func Extra() -> int
}
''')
for name in ['Unions', 'Intersections', 'Contracts', 'ExternalBounds']:
    reference = '''<Reference Include="Contracts"><HintPath>../Contracts/bin/neoclr/Contracts.dll</HintPath></Reference>''' if name == 'ExternalBounds' else ''
    (kit / name / f'{name}.rvnproj').write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="../library/NeoCLR.ClassLibrary.props" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>{'Library' if name == 'Contracts' else 'Exe'}</OutputType>
    <AssemblyName>{name}</AssemblyName>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>

    <RavenNeoClrRuntimeAsyncHelper></RavenNeoClrRuntimeAsyncHelper>
  </PropertyGroup>
  <ItemGroup>
    <Compile Remove="../library/RuntimeAwait.rvn" />
    <Compile Remove="../library/RuntimeGenerator.rvn" />
    <Compile Include="Main.rvn" />
    {reference}
  </ItemGroup>
</Project>
''')
shutil.copy2(Path(__file__).with_name('run.py'), kit / 'run.py')
shutil.copy2(Path(__file__).with_name('README.md'), kit / 'README.md')
settings = {'raven.sdkPath': str(kit), 'raven.languageServerPath': str(kit / 'tools/language-server/Raven.LanguageServer.dll'), 'raven.autoBuildLanguageServerOnActivate': False}
tasks = []
for sample in ['Unions', 'Intersections', 'ExternalBounds']:
    tasks.append({'label': f'Run {sample}', 'type': 'process', 'command': 'python3', 'args': [str(kit / 'run.py'), sample], 'problemMatcher': [], 'group': 'test'})
(kit / 'StructuralTypes.code-workspace').write_text(json.dumps({'folders': [{'path': name} for name in ['Unions', 'Intersections', 'ExternalBounds', 'Contracts']], 'settings': settings, 'tasks': {'version': '2.0.0', 'tasks': tasks}}, indent=2) + '\n')
print(kit / 'StructuralTypes.code-workspace')
