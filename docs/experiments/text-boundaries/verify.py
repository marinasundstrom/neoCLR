"""Run only the application-level text foundation contract checks."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-text-boundaries-') as directory:
    work = Path(directory)
    for name in ('Main.rvn', 'Boundaries.rvn', 'Decoder.rvn', 'Boundaries.rvnproj'):
        shutil.copyfile(root / name, work / name)
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Boundaries.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    common = [app, '--system', artifacts['system']]
    run([artifacts['runtime'], 'verify', *common])
    results = {}
    for label, guest_args in [('contracts', []), ('splits-0-6', ['first']),
                              ('splits-7-13', ['last'])]:
        output = run([artifacts['runtime'], 'run', *common, '--', *guest_args])
        expected = ('Scalar and source-range contracts: passed',
                    'Decoding ownership, progress and errors: passed') if not guest_args else (
                    'Bounded UTF-8 split checks: passed',)
        if any(marker not in output for marker in expected):
            raise SystemExit(f'Missing success marker in {label}')
        results[label] = list(expected)
    if args.evidence:
        evidence = {'scope': 'Application prototype; no System API change',
                    'runs': results,
                    'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest()
                               for name, path in artifacts.items()}}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
