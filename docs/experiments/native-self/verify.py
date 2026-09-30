"""Compile and execute native Self cloning for an application struct and class."""
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
artifacts = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent


def run(command):
    result = subprocess.run([str(part) for part in command], text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=180)
    print(result.stdout, end='', flush=True)
    result.check_returncode()
    return result.stdout


with tempfile.TemporaryDirectory(prefix='neoclr-native-self-') as directory:
    work = Path(directory)
    for name in ('Main.rvn', 'Contracts.rvnproj'):
        shutil.copyfile(here / name, work / name)
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Contracts.rvnproj', work / 'out'])
    app = work / 'out/App.neoil'
    if 'callself borrow ' not in app.read_text() or 'instance System.Clonable::Clone()' not in app.read_text():
        raise SystemExit('Importer lost borrowed native Self dispatch')
    run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']])
    report = 'Generic Self cloning passed'
    if report not in run([artifacts['runtime'], 'run', app, '--system', artifacts['system']]):
        raise SystemExit('Missing clone success marker')
    source = (here / 'Main.rvn').read_text()
    for name, changed in (
        ('inherited-bound', source.replace('Copy<BaseClone>(InheritedClone())', 'Copy<InheritedClone>(InheritedClone())')),
        ('redeclared-base-result', source.replace('class InheritedClone : BaseClone {}', 'class InheritedClone : BaseClone, Clonable {}')),
        ('missing-bound', source.replace(' where T: Clonable', '')),
        ('obsolete-arity', source.replace('where T: Clonable', 'where T: Clonable<T>')),
        ('wrong-result', source.replace('func Clone() -> Self => Cell(Value)', 'func Clone() -> int => Value')),
        ('erased-receiver', source.replace('value: T) -> T where T: Clonable', 'value: Clonable) -> T where T: Clonable')),
    ):
        (work / 'Main.rvn').write_text(changed)
        rejected = subprocess.run(['dotnet', str(artifacts['bridge']), '--project', str(work / 'Contracts.rvnproj'), str(work / name)],
                                  text=True, capture_output=True, timeout=180)
        expected = ('Application specialization requires a supported nongeneric Self cloning or Number bound:'
                    if name == 'obsolete-arity' else 'error RAV')
        if rejected.returncode == 0 or expected not in rejected.stdout + rejected.stderr:
            raise SystemExit(f'Expected rejection for {name}: {rejected.stdout}{rejected.stderr}')
    if args.evidence:
        evidence = {'report': report, 'rejected': ['missing bound', 'obsolete generic arity', 'wrong Self result', 'erased receiver', 'inherited derived bound', 'redeclared base result'],
                    'sourceSha256': hashlib.sha256((here / 'Main.rvn').read_bytes()).hexdigest(),
                    'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in artifacts.items()}}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
