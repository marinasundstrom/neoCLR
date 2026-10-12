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
        (settings / 'tasks.json').write_text(json.dumps({'version': '2.0.0', 'tasks': [dict(
            label='neoCLR: Run', type='process', command='python3', args=[
                '${workspaceFolder}/../../tools/run-native-sample.py', name],
            problemMatcher=[])]}, indent=2) + '\n')
