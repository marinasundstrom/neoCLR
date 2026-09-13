"""Check native buffer lifetime, bounds and admission through saved Raven sources."""
import argparse
from runner_options import add_toolchain_arguments, runner_arguments
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
add_toolchain_arguments(parser)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-native-check-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'), *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    for name, body in [
        ('double free', 'let pointer = NativeMemory.Alloc(default(nuint))\nNativeMemory.Free(pointer)\nNativeMemory.Free(pointer)')]:
        (root / 'Main.rvn').write_text('import System.*\nimport System.Runtime.InteropServices.*\nunsafe func Main() {\n' + body + '\n}')
        result = subprocess.run(command, capture_output=True, text=True, timeout=90)
        if result.returncode == 0 or 'Fault:' not in result.stderr:
            raise AssertionError(name + ': ' + result.stdout + result.stderr)
    before = set(root.rglob('App.neoil'))
    (root / 'Main.rvn').write_text('import System.*\nimport System.Runtime.InteropServices.*\nunsafe func Main() { let values = Array<string>.Allocate(1, "managed") }')
    result = subprocess.run(command, capture_output=True, text=True, timeout=90)
    if result.returncode == 0 or set(root.rglob('App.neoil')) != before:
        raise AssertionError('Removed native array API admitted: ' + result.stdout + result.stderr)
print('Native release fault and removed array descriptor API rejection passed')
