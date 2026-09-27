"""Compile and run the Unicode casing and Int64 report API contracts."""
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
    shutil.copyfile(root / 'Main.rvn', work / 'Main.rvn')
    shutil.copyfile(root / 'Contracts.rvnproj', work / 'Contracts.rvnproj')
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Contracts.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']])
    marker = 'Unicode casing and Int64 report contracts passed'
    output = run([artifacts['runtime'], 'run', app, '--system', artifacts['system']])
    if marker not in output:
        raise SystemExit('Missing success marker')
    results = {'report': marker}
    if args.evidence:
        evidence = {'scope': 'Unicode casing and Int64 parsing/formatting', 'runs': results,
                    'sourceSha256': {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in ('Main.rvn',)},
                    'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest()
                               for name, path in artifacts.items()}}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
