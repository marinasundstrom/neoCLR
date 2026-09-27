"""Compile and run only the shared encoding-selection contract probe."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, required=True, type=Path)
parser.add_argument('--evidence', type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parent
artifacts = {name: getattr(args, name).resolve() for name in
             ('runtime', 'bridge', 'system', 'reference')}
for path in artifacts.values():
    if not path.is_file():
        parser.error(f'Missing artifact: {path}')


def run(command):
    result = subprocess.run([str(part) for part in command], text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout, end='', flush=True)
    result.check_returncode()
    return result.stdout


with tempfile.TemporaryDirectory(prefix='neoclr-encoding-selection-') as directory:
    work = Path(directory)
    for name in ('Selection.rvn', 'SelectionMain.rvn'):
        shutil.copyfile(root / name, work / name)
    project = (root / 'Boundaries.rvnproj').read_text()
    for name in ('Main.rvn', 'Boundaries.rvn', 'Decoder.rvn', 'Consumer.rvn'):
        project = project.replace(f'    <Compile Include="{name}" />\n', '')
    project = project.replace('<ItemGroup>', '<ItemGroup>\n    <Compile Include="SelectionMain.rvn" />\n    <Compile Include="Selection.rvn" />', 1)
    (work / 'Selection.rvnproj').write_text(project)
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Selection.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']])
    results = {}
    for label, marker in [('selection-read', 'Encoding selection reader and independent decoder checks passed'),
                          ('selection-write', 'Encoding selection writer and strict rejection checks passed')]:
        output = run([artifacts['runtime'], 'run', app, '--system', artifacts['system'], '--', label])
        if marker not in output:
            raise SystemExit(f'Missing success marker: {label}')
        results[label] = [marker]
    if args.evidence:
        evidence = {'scope': 'Application prototype; no System API change', 'runs': results,
                    'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest()
                               for name, path in artifacts.items()}}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
