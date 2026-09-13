"""Check the order workflow's observable state and persisted report in isolation."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from runner_options import add_toolchain_arguments, runner_arguments

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--runtime', type=Path, required=True)
add_toolchain_arguments(parser)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-order-workflow-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    source = (bridge / 'samples/application-orders.rvn').read_text()
    (root / 'Main.rvn').write_text(source)
    run = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=120)
    expected = 'Saved\nOrder report exceeds limit\nOrder not found\nSaved\nQueued\nOrder: Coffee\nPending orders\nTea\n'
    assert run.returncode == 0 and run.stdout.endswith(expected), run.stdout + run.stderr
    assert (root / 'neoclr-orders.txt').read_text() == 'Order: Coffee'
    (root / 'Main.rvn').write_text(source.replace('neoclr-orders.txt', 'missing-parent/report.txt'))
    run = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=120)
    expected = 'Order report could not be written\nOrder report exceeds limit\nOrder not found\nQueued\nQueued\nOrder report could not be read\nPending orders\nCoffee\nTea\n'
    assert run.returncode == 0 and run.stdout.endswith(expected), run.stdout + run.stderr
    assert (root / 'neoclr-orders.txt').read_text() == 'Order: Coffee'
    assert not (root / 'missing-parent').exists()
    (root / 'Main.rvn').write_text(source.replace('Process(store, 8, 2)', 'Process(store, 8, 64)'))
    run = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=120)
    expected = 'Saved\nSaved\nOrder not found\nSaved\nSaved\nOrder: Tea\nPending orders\n'
    assert run.returncode == 0 and run.stdout.endswith(expected), run.stdout + run.stderr
    assert (root / 'neoclr-orders.txt').read_text() == 'Order: Tea'
print(json.dumps({'workflow': 'passed', 'persisted_report': 'passed', 'failed_write_preserves_state': 'passed', 'missing_parent_errors': 'passed', 'deferred_pending_summary': 'passed', 'all_saved_summary_is_empty': 'passed'}, indent=2))
