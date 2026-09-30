"""Compile and run the Number and concrete parsing API contracts."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-number-contract-') as directory:
    work = Path(directory)
    shutil.copyfile(root / 'Main.rvn', work / 'Main.rvn')
    shutil.copyfile(root / 'Contracts.rvnproj', work / 'Contracts.rvnproj')
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Contracts.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    if 'callself ' not in app.read_text() or 'System.Number::op_Addition(Self,Self)' not in app.read_text():
        raise SystemExit('Numeric consumer lost native Self dispatch')
    run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']])
    marker = 'Number and concrete Parse contracts passed'
    output = run([artifacts['runtime'], 'run', app, '--system', artifacts['system']])
    if marker not in output:
        raise SystemExit('Missing success marker')
    for name, source, diagnostic in (
        ('boolean', 'import System.*\nfunc Calculate<T>(x: T) -> T where T: Number => x + T.One\nfunc Main() { Calculate<bool>(true) }', 'constraint'),
        ('extra-constraint', 'import System.*\nfunc Calculate<T>(x: T) -> T where T: Number, struct => x + T.One\nfunc Main() { Calculate<int>(1) }', 'Application specialization requires a supported nongeneric Self cloning or Number bound'),
    ):
        (work / 'Main.rvn').write_text(source + '\n')
        rejected = subprocess.run(['dotnet', str(artifacts['bridge']), '--project', str(work / 'Contracts.rvnproj'), str(work / name)], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if rejected.returncode == 0 or diagnostic.lower() not in rejected.stdout.lower():
            raise SystemExit(f'Expected {name} rejection: {rejected.stdout}')
    results = {'report': marker, 'rejected': ['Boolean Number argument', 'additional constraints']}

    if args.evidence:
        evidence = {'scope': 'Number and concrete primitive parsing', 'runs': results,
                    'sourceSha256': {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in ('Main.rvn',)},
                    'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest()
                               for name, path in artifacts.items()}}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
