"""Run the bounded view prototype against an installed Raven/neoCLR bundle."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
args = parser.parse_args()
bundle = args.bundle.resolve()
here = Path(__file__).resolve().parent
source = (here / 'Main.rvn').read_text()
declarations = source.split('func Main() {')[0]
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-readonly-view-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(bundle / 'demo' / name, root / name)
    command = [sys.executable, str(bundle / 'tools/run_project.py'), str(root / 'Demo.rvnproj'),
               '--bridge', str(bundle / 'tools/bridge/Probe.dll'),
               '--system', str(bundle / 'lib/System.neoil'), '--runtime', str(bundle / 'bin/neoclr')]
    cases = [
        ('shared buffer, base reads, mutable elements and escaping view', source, '1\n7\n1\n42\n99\n1\n11\n'),
        ('empty view', declarations + 'func Main() { let dogs: Dog[] = []; let view: AnimalView = DogArrayView(dogs); WriteLine(view.Count) }', '0\n'),
    ]
    for name, text, expected in cases:
        (root / 'Main.rvn').write_text(text)
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert run.returncode == 0 and run.stdout.endswith(expected), (name, run.stdout, run.stderr)
        results[name] = 'passed'
    for name, body in [
        ('no setter', 'let view: AnimalView = MakeView(); view[0] = Dog(8)'),
        ('no backing field access', 'let view = DogArrayView([Dog(7)]); WriteLine(view.data.Length)'),
        ('mutable array widening stays invalid', 'let dogs: Dog[] = [Dog(7)]; let animals: Animal[] = dogs; WriteLine(animals.Length)'),
    ]:
        (root / 'Main.rvn').write_text(declarations + 'func Main() { ' + body + ' }')
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert run.returncode != 0 and 'error RAV' in run.stderr, (name, run.stdout, run.stderr)
        assert 'Verified saved project:' not in run.stdout, (name, run.stdout)
        results[name] = 'rejected during binding'
print(json.dumps(results, indent=2))
