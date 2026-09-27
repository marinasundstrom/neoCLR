"""Focused explicit string comparison, folding/hash and invalid-mode contracts."""
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
parser.add_argument('--runtime', required=True, type=Path)
add_toolchain_arguments(parser)
args = parser.parse_args()
tools = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-string-comparison-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(tools / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    results = {}

    def check(name, source, expected, success=True):
        (root / 'Main.rvn').write_text(source)
        result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=180)
        if (result.returncode == 0) != success or expected not in result.stdout + result.stderr:
            raise AssertionError(name + ': ' + result.stdout + result.stderr)
        results[name] = 'passed'

    check('contracts', (tools / 'samples/library-string-comparison.rvn').read_text(), 'String comparison contract passed\n')
    check('invalid mode', 'import System.*\nfunc Main() { String.Compare("a", "b", (StringComparison)99) }\n', 'Unsupported StringComparison', success=False)
    check('explicit mode required', 'import System.*\nfunc Main() { String.Compare("a", "b") }\n', 'RAV', success=False)
    print(json.dumps(results, indent=2))
