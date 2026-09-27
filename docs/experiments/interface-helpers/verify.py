"""Compile and run the interface defaults and static helper contracts."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-interface-contract-') as directory:
    work = Path(directory)
    shutil.copyfile(root / 'Main.rvn', work / 'Main.rvn')
    shutil.copyfile(root / 'Contracts.rvnproj', work / 'Contracts.rvnproj')
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Contracts.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']])
    marker = 'Interface helper contracts passed'
    output = run([artifacts['runtime'], 'run', app, '--system', artifacts['system']])
    if marker not in output:
        raise SystemExit('Missing success marker')
    for name, source, diagnostic in (
        ('private-access', (root / 'Main.rvn').read_text().replace('Calculation.Offset() != 2', 'Calculation.Twice(1) != 2'), 'inaccessible'),
        ('private-instance', (root / 'Main.rvn').read_text().replace('private static func Twice', 'private func Twice').replace('Calculation.Twice(Seed())', 'Twice(Seed())'), 'Unsupported interface member'),
    ):
        (work / 'Main.rvn').write_text(source + '\n')
        rejected = subprocess.run(['dotnet', str(artifacts['bridge']), '--project', str(work / 'Contracts.rvnproj'), str(work / name)], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if rejected.returncode == 0 or diagnostic.lower() not in rejected.stdout.lower():
            raise SystemExit(f'Expected {name} rejection: {rejected.stdout}')
    results = {'report': marker, 'rejected': ['external private static access', 'private instance helper outside bounded admission']}

    if args.evidence:
        evidence = {'scope': 'Nominal interface defaults and public/private static helpers', 'runs': results,
                    'sourceSha256': {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in ('Main.rvn',)},
                    'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest()
                               for name, path in artifacts.items()}}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
