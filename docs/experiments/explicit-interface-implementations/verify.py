"""Compile and run the explicit interface implementation contracts."""
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


artifact_hashes = {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in artifacts.items()}
source_hashes = {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in ('Main.rvn', 'Contracts.rvnproj')}


def run(command):
    result = subprocess.run([str(part) for part in command], text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout, end='', flush=True)
    result.check_returncode()
    return result.stdout


with tempfile.TemporaryDirectory(prefix='neoclr-explicit-contract-') as directory:
    work = Path(directory)
    shutil.copyfile(root / 'Main.rvn', work / 'Main.rvn')
    shutil.copyfile(root / 'Contracts.rvnproj', work / 'Contracts.rvnproj')
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Contracts.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    marker = 'Explicit interface contracts passed'
    output = run([artifacts['runtime'], 'run', app, '--system', artifacts['system']])
    if marker not in output:
        raise SystemExit('Missing success marker')
    for name, source, diagnostic in (
        ('ordinary-access', (root / 'Main.rvn').read_text().replace('only.Read() != 7', 'OnlyExplicit().Read() != 7'), 'Read'),
        ('undeclared-interface', (root / 'Main.rvn').read_text().replace('class OnlyExplicit : Right', 'class OnlyExplicit'), 'interface'),
    ):
        (work / 'Main.rvn').write_text(source + '\n')
        rejected = subprocess.run(['dotnet', str(artifacts['bridge']), '--project', str(work / 'Contracts.rvnproj'), str(work / name)], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if rejected.returncode == 0 or diagnostic.lower() not in rejected.stdout.lower():
            raise SystemExit(f'Expected {name} rejection: {rejected.stdout}')
    results = {'report': marker, 'rejected': ['ordinary access to explicit-only member', 'explicit implementation without interface declaration']}

    if args.evidence:
        evidence = {'scope': 'Explicit application class interface implementations', 'runs': results,
                    'sourceSha256': source_hashes,
                    'sha256': artifact_hashes}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
