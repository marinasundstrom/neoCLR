"""Create an isolated, editable Raven project from a completed target probe."""
import argparse
import json
from pathlib import Path
import shutil
from configure_tasks import configure

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path)
parser.add_argument('raven', type=Path)
parser.add_argument('--collections', action='store_true', help='Use a completed --interfaces probe and the target iteration profile')
parser.add_argument('--server', type=Path, help='Override the built server, e.g. the installed VSIX server')
parser.add_argument('--sdk', type=Path, help='Optional local SDK for this workspace')
parser.add_argument('--runtime', type=Path, default=Path(__file__).resolve().parents[3] / 'target/debug/neoclr', help='Built neoCLR executable for the tasks')
args = parser.parse_args()
output, raven = args.output.resolve(), args.raven.resolve()
core = (output if args.collections else output / 'union-probe') / 'NeoCLR.CoreProbe.dll'
server = (args.server or raven / 'src/Raven.LanguageServer/bin/Debug/net11.0/Raven.LanguageServer.dll').resolve()
if not core.is_file() or not server.is_file():
    raise SystemExit('Run the target probe and build Raven.LanguageServer for net11.0 first.')
project = output / 'editor'
project.mkdir(exist_ok=False)
(project / '.vscode').mkdir()
shutil.copyfile(core, project / core.name)
shutil.copyfile(Path(__file__).parent / ('samples/library-workflow.rvn' if args.collections else 'samples/library-result.rvn'), project / 'Main.rvn')
(project / 'Demo.rvnproj').write_text('''<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net11.0</TargetFramework>
    <OutputType>Exe</OutputType>
    <RavenMetadataCoreAssemblyName>NeoCLR.CoreProbe</RavenMetadataCoreAssemblyName>
    <RavenUseHostFrameworkReferences>false</RavenUseHostFrameworkReferences>
    <RavenPropagationAssemblyName>NeoCLR.CoreProbe</RavenPropagationAssemblyName>
    <RavenPropagationInterfaceType>System.Propagatable`3</RavenPropagationInterfaceType>
    <ImplicitImports>disable</ImplicitImports>
    <RavenFrameworkProjections>None</RavenFrameworkProjections>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup>
    <Compile Include="Main.rvn" />
    <Reference Include="NeoCLR.CoreProbe"><HintPath>NeoCLR.CoreProbe.dll</HintPath></Reference>
  </ItemGroup>
</Project>
''')
if args.collections:
    project_file = project / 'Demo.rvnproj'
    project_file.write_text(project_file.read_text().replace('<ImplicitImports>',
        '<RavenIterationAssemblyName>NeoCLR.CoreProbe</RavenIterationAssemblyName>\n'
        '    <RavenIterationIterableType>System.Collections.Iterable`1</RavenIterationIterableType>\n'
        '    <RavenIterationIteratorType>System.Collections.Iterator`1</RavenIterationIteratorType>\n'
        '    <ImplicitImports>'))
settings = {'raven.languageServerPath': str(server)}
if args.sdk:
    settings['raven.sdkPath'] = str(args.sdk.resolve())
(project / '.vscode/settings.json').write_text(json.dumps(settings, indent=2)+'\n')
configure(project / 'Demo.rvnproj', raven, args.runtime.resolve())
print(project)
