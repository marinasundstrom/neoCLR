"""Check error case APIs, wrong-case faults and invalid carrier defaults."""
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
with tempfile.TemporaryDirectory(prefix='neoclr-error-values-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    (root / 'Main.rvn').write_text('''import System.*
func Main() {
    let error = Int32ParseError.Overflow()
    error.GetInvalidFormat()
}
''')
    result = subprocess.run(command, capture_output=True, text=True, timeout=240)
    if result.returncode == 0 or 'Fault:' not in result.stderr:
        raise AssertionError(result.stdout + result.stderr)
    previous = set(root.rglob('App.neoil'))
    (root / 'Main.rvn').write_text('''import System.*
func Main() {
    let error = default(Int32ParseError)
    System.Console.WriteLine(error.ToString())
}
''')
    result = subprocess.run(command, capture_output=True, text=True, timeout=240)
    if result.returncode == 0 or 'uninitialized' not in result.stderr or set(root.rglob('App.neoil')) != previous:
        raise AssertionError(result.stdout + result.stderr)
    print('Wrong-case access faults; invalid carrier default is rejected before execution')
