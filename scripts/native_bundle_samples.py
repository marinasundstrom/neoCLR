"""Current preview sample inputs shared by package creation and development refresh."""
from pathlib import Path
import shutil
import json

ROOT = Path(__file__).resolve().parents[1]


def sample_definitions():
    return {
        'collections': (['raven-target/samples/application-order-collections.rvn'],
                        (ROOT / 'docs/experiments/raven-target/samples/application-order-collections.expected.txt').read_text(), []),
        'tasks': (['raven-target/samples/library-async.rvn'], 'Suspended\n42\n', []),
        'json': (['json-object-mapping/Mapping.rvn', 'json-object-mapping/Main.rvn'],
                 'JSON object mapping checks passed\n', ['StationReport']),
        'http-json-server': (['json-object-mapping/HttpApplication.rvn', 'json-object-mapping/HttpServer.rvn'],
                             None, ['ReportPayload', 'Acknowledgement']),
        'http-json-client': (['json-object-mapping/HttpApplication.rvn', 'json-object-mapping/HttpClient.rvn'],
                             None, ['ReportPayload', 'Acknowledgement']),
    }


def refresh_samples(bundle, manifest):
    definitions = sample_definitions()
    if set(manifest['samples']) != set(definitions):
        raise ValueError('Cannot refresh an unknown bundle sample selection')
    for name, (sources, expected, roots) in definitions.items():
        folder = bundle / 'samples' / name
        for old in folder.glob('*.rvn'):
            old.unlink()
        for source in sources:
            shutil.copy2(ROOT / 'docs/experiments' / source, folder / Path(source).name)
        manifest['samples'][name] = dict(expected=expected, reflectionTypes=roots)


def configure_editor_tasks(bundle, names):
    for name in names:
        settings = bundle / 'samples' / name / '.vscode'
        settings.mkdir(exist_ok=True)
        arguments = ['${workspaceFolder}/../../tools/run-native-sample.py', name, '--execution', 'jit']
        inputs = []
        if name == 'http-json-server':
            inputs = [dict(id='requestCount', type='promptString',
                           description='Number of requests before the server exits (the client sends two)', default='2')]
            arguments += ['--', '${input:requestCount}']
        elif name == 'http-json-client':
            inputs = [dict(id='serverUrl', type='promptString',
                           description='Server URL using the port printed by the server', default='http://127.0.0.1:8080/')]
            arguments += ['--', '${input:serverUrl}']
        tasks = {'version': '2.0.0', 'tasks': [dict(
            label='neoCLR: Build and Run (JIT)', type='process', command='python3', args=arguments,
            group=dict(kind='build', isDefault=True), problemMatcher=[])]}
        native_arguments = ['${input:nativeBuildKit}/scripts/build-native-project.py',
                            '--project', '${workspaceFolder}/App.rvnproj',
                            '--output', '${input:nativeOutput}',
                            '--profile', 'http' if name.startswith('http-') else 'console']
        for type_name in sample_definitions()[name][2]:
            native_arguments += ['--reflection-type', type_name]
        tasks['tasks'].append(dict(label='neoCLR: Compile to Native', type='process',
                                   command='python3', args=native_arguments, problemMatcher=[]))
        inputs += [dict(id='nativeBuildKit', type='promptString',
                        description='Matching extracted native build kit directory (currently macOS ARM64)'),
                   dict(id='nativeOutput', type='promptString',
                        description='New absolute output directory for the native executable (must not exist)')]
        if inputs:
            tasks['inputs'] = inputs
        (settings / 'tasks.json').write_text(json.dumps(tasks, indent=2) + '\n')
