"""Check terminal callback faults and rejection of uninitialized callback use."""
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
with tempfile.TemporaryDirectory(prefix='neoclr-delegate-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    (root / 'Main.rvn').write_text('''import System.*
import System.Collections.*
func Fail(value: int) -> bool {
    let result = Result<int, Error>.Ok(value)
    result.GetErrorCase()
    return false
}
func Main() {
    let values = ArrayList<int>()
    values.Add(42)
    values.Exists(Fail)
}
''')
    run = subprocess.run(command, capture_output=True, text=True, timeout=90)
    if run.returncode == 0 or 'Fault:' not in run.stderr:
        raise AssertionError(run.stdout + run.stderr)
    previous = set(root.rglob('App.neoil'))
    (root / 'Main.rvn').write_text('''import System.*
func Main() {
    let callback = default(Func<int>)
    Console.WriteLine(callback())
}
''')
    run = subprocess.run(command, capture_output=True, text=True, timeout=90)
    if run.returncode == 0 or set(root.rglob('App.neoil')) != previous:
        raise AssertionError(run.stdout + run.stderr)
    print('Callback fault remains terminal; null/default callback is rejected before execution')
