"""Check removed error helpers and valid inactive union defaults."""
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
    if result.returncode == 0 or 'GetInvalidFormat' not in result.stdout + result.stderr:
        raise AssertionError(result.stdout + result.stderr)
    (root / 'Main.rvn').write_text('''import System.*
func Main() {
    let error = default(Int32ParseError)
    System.Console.WriteLine(error.ToString())
}
''')
    result = subprocess.run(command, capture_output=True, text=True, timeout=240)
    if result.returncode != 0 or 'Empty' not in result.stdout:
        raise AssertionError(result.stdout + result.stderr)
    print('Removed case helper rejected; inactive default executes as Empty')
