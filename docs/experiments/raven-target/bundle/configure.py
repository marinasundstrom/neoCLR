"""Configure the extracted demonstration for its current location."""
import argparse
import json
from pathlib import Path
import sys

root = Path(__file__).resolve().parent
sys.path.insert(0, str(root / 'tools'))
from configure_tasks import configure
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--sdk', type=Path, help='Optional separately installed experimental Raven SDK')
args = parser.parse_args()
project = root / 'demo/Demo.rvnproj'
configure(project, None, root / 'bin/neoclr', root / 'tools/bridge/Probe.dll', root / 'lib/System.neoil')
settings = {'raven.languageServerPath': str(root / 'tools/server/Raven.LanguageServer.dll')}
if args.sdk:
    settings['raven.sdkPath'] = str(args.sdk.resolve())
(project.parent / '.vscode/settings.json').write_text(json.dumps(settings, indent=2) + '\n')
# Primary Raven workflows use standalone MSBuild; retain the older runner project.
for relative in ('msbuild-demo', 'project-reference-demo/App'):
    folder = root / relative
    if not folder.is_dir():
        continue
    (folder / '.vscode').mkdir(exist_ok=True)
    (folder / '.vscode/settings.json').write_text(json.dumps(settings, indent=2) + '\n')
    if args.sdk:
        tasks = {'version': '2.0.0', 'tasks': [{
            'label': 'neoCLR: Build with MSBuild', 'type': 'process', 'command': 'dotnet',
            'args': ['msbuild', str(folder / 'Demo.rvnproj'),
                     '-p:RavenSdkRoot=' + str(args.sdk.resolve()), '-v:minimal'],
            'group': {'kind': 'build', 'isDefault': True}, 'problemMatcher': '$msCompile'}, {
            'label': 'neoCLR: Run (MSBuild)', 'type': 'process',
            'command': str(root / 'bin/neoclr'),
            'args': ['run', str(folder / 'bin/neoclr/Debug/App.neoil'),
                     '--system', str(folder / 'bin/neoclr/Debug/System.neoil')],
            'options': {'cwd': str(folder)}, 'dependsOn': 'neoCLR: Build with MSBuild',
            'dependsOrder': 'sequence', 'problemMatcher': []}]}
        (folder / '.vscode/tasks.json').write_text(json.dumps(tasks, indent=2) + '\n')
    print('Open MSBuild project in VS Code:', folder)
if not args.sdk:
    print('Supply --sdk /path/to/matching/raven-sdk to configure MSBuild build/run tasks.')
print('Advanced compiler/import runner project:', project.parent)
