"""Check native buffer lifetime, bounds and admission through saved Raven sources."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--raven', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-native-check-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'), '--raven', str(args.raven.resolve()), '--runtime', str(args.runtime.resolve())]
    for name, body in [
        ('negative length', 'let values = Array<int>.Allocate(-1, 0)'),
        ('bounds', 'let values = Array<int>.Allocate(1, 0)\nConsole.WriteLine(values[1])'),
        ('after free', 'let values = Array<int>.Allocate(1, 0)\nlet pointer = values.GetElementAddress(0)\nvalues.Free()\nConsole.WriteLine(*pointer)'),
        ('double free', 'let values = Array<int>.Allocate(1, 0)\nvalues.Free()\nvalues.Free()')]:
        (root / 'Main.rvn').write_text('import System.*\nunsafe func Main() {\n' + body + '\n}')
        result = subprocess.run(command, capture_output=True, text=True, timeout=90)
        if result.returncode == 0 or 'Fault:' not in result.stderr:
            raise AssertionError(name + ': ' + result.stdout + result.stderr)
    before = set(root.rglob('App.neoil'))
    (root / 'Main.rvn').write_text('import System.*\nunsafe func Main() { let values = Array<string>.Allocate(1, "managed") }')
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    if result.returncode == 0 or set(root.rglob('App.neoil')) != before:
        raise AssertionError('Managed native element admitted: ' + result.stdout + result.stderr)
print('Native length/bounds/lifetime faults and unsupported managed layout rejection passed')
