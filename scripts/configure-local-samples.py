#!/usr/bin/env python3
"""Prepare editable samples for a matching installed neoCLR/Raven development bundle."""
import argparse
import json
from pathlib import Path
import shlex
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--sdk', type=Path, required=True)
    parser.add_argument('--http-runner', type=Path, required=True, help='Built release examples/measure_async binary')
    parser.add_argument('--source-ref', default='HEAD', help='Committed sample revision; ignores working-tree edits')
    parser.add_argument('--profile', type=Path, default=Path.home() / '.neoclr/vscode/raven-port-20260919')
    args = parser.parse_args()
    bundle, sdk, profile = (p.resolve() for p in (args.bundle, args.sdk, args.profile))
    revision = subprocess.check_output(['git', 'rev-parse', args.source_ref], cwd=ROOT, text=True).strip()
    if not (sdk / 'tools/language-server/Raven.LanguageServer.dll').is_file() or not (bundle / 'bin/neoclr').is_file():
        parser.error('Select an installed SDK and matching runtime bundle.')
    if not args.http_runner.is_file():
        parser.error('Build the HTTP runner with cargo build --locked --release --example measure_async.')
    samples = bundle / 'editable-samples'
    samples.mkdir(exist_ok=True)
    settings = {'raven.sdkPath': str(sdk), 'raven.languageServerPath': str(sdk / 'tools/language-server/Raven.LanguageServer.dll')}

    def source(path):
        return subprocess.check_output(['git', 'show', revision + ':' + path], cwd=ROOT)

    def project(folder, name, sources, run_arguments):
        folder.mkdir(parents=True, exist_ok=False)  # Never overwrite edited samples.
        for filename, origin in sources.items():
            (folder / filename).write_bytes(source(origin))
        document = ET.Element('Project', DefaultTargets='Build')
        properties = ET.SubElement(document, 'PropertyGroup')
        ET.SubElement(properties, 'NeoCLRRoot').text = str(bundle)
        ET.SubElement(properties, 'RavenSdkRoot').text = str(sdk)
        ET.SubElement(document, 'Import', Project='$(NeoCLRRoot)/build/NeoCLR.Raven.props')
        items = ET.SubElement(document, 'ItemGroup')
        for filename in sources:
            ET.SubElement(items, 'Compile', Include=filename)
        ET.SubElement(document, 'Import', Project='$(NeoCLRRoot)/build/NeoCLR.Raven.targets')
        ET.indent(document)
        project_path = folder / (name + '.rvnproj')
        ET.ElementTree(document).write(project_path, encoding='unicode')
        build = {'label': 'neoCLR: Build', 'type': 'process', 'command': 'dotnet',
                 'args': ['msbuild', str(project_path), '-v:minimal'],
                 'group': {'kind': 'build', 'isDefault': True}, 'problemMatcher': '$msCompile'}
        run = {'label': 'neoCLR: Run', 'type': 'process', 'command': str(bundle / 'bin/neoclr'),
               'args': ['run', str(folder / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil'), '--', *run_arguments],
               'options': {'cwd': str(folder)}, 'dependsOn': 'neoCLR: Build', 'dependsOrder': 'sequence', 'problemMatcher': []}
        if name in ('Server', 'Client'):
            run['command'] = str(bundle / 'tools/http-runner')
            run['args'] = [str(folder / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'),
                           '1024', '100000000', *(['--live-output'] if name == 'Server' else []), '--', *run_arguments]
        tasks = {'version': '2.0.0', 'tasks': [build, run]}
        if name == 'Client':
            tasks['inputs'] = [{'id': 'serverUrl', 'type': 'promptString', 'description': 'Server URL (use the port printed in the Server terminal)', 'default': 'http://127.0.0.1:8080/'}]
        (folder / '.vscode').mkdir()
        (folder / '.vscode/settings.json').write_text(json.dumps(settings, indent=2) + '\n')
        (folder / '.vscode/tasks.json').write_text(json.dumps(tasks, indent=2) + '\n')

    http = samples / 'http-json'
    if http.exists():
        parser.error('HTTP/JSON sample already exists; keeping your edits. Use a fresh bundle for another copy.')
    shutil.copy2(args.http_runner, bundle / 'tools/http-runner')
    shutil.copy2(ROOT / 'scripts/verify-local-http-json.py', bundle / 'tools/verify-local-http-json.py')
    for name, arguments in [('Server', ['100']), ('Client', ['${input:serverUrl}'])]:
        sources = {name + '.rvn': 'docs/experiments/' + ('json-object-mapping/HttpServer.rvn' if name == 'Server' else 'http-json/Client.rvn'),
                   'Application.rvn': 'docs/experiments/json-object-mapping/HttpApplication.rvn'}
        project(http / name.lower(), name, sources, arguments)
    workspace = {'folders': [{'name': 'Server', 'path': 'server'}, {'name': 'Client', 'path': 'client'}], 'settings': settings}
    workspace['tasks'] = {'version': '2.0.0', 'tasks': [{
        'label': 'neoCLR: Verify HTTP JSON flow', 'type': 'process', 'command': 'python3',
        'args': [str(bundle / 'tools/verify-local-http-json.py'), '--bundle', str(bundle)], 'problemMatcher': []}]}
    (http / 'HttpJson.code-workspace').write_text(json.dumps(workspace, indent=2) + '\n')
    for sample in ['library-calendar', 'library-async']:
        project(samples / sample, 'Demo', {'Main.rvn': 'docs/experiments/raven-target/samples/' + sample + '.rvn'}, [])
    (samples / 'source.json').write_text(json.dumps({'revision': revision, 'sdk': str(sdk)}, indent=2) + '\n')
    launcher = '''#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sample=${1:-http-json}
case "$sample" in
  http-json) target="$root/editable-samples/http-json/HttpJson.code-workspace" ;;
  library-calendar|library-async) target="$root/editable-samples/$sample" ;;
  *) echo "Samples: http-json (default), library-calendar, library-async" >&2; exit 2 ;;
esac
exec '/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code' --new-window --user-data-dir PROFILE --extensions-dir EXTENSIONS "$target"
'''.replace('PROFILE', shlex.quote(str(profile))).replace('EXTENSIONS', shlex.quote(str(profile / 'extensions')))
    (bundle / 'open-vscode.sh').write_text(launcher)
    (bundle / 'open-vscode.sh').chmod(0o755)
    print(bundle / 'open-vscode.sh')


if __name__ == '__main__':
    main()
