"""Compile and execute the standard Option/Result/TaskOutcome contract with the selected tools."""
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
with tempfile.TemporaryDirectory(prefix='neoclr-union-contract-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    shutil.copyfile(tools / 'samples/library-union-contract.rvn', root / 'Main.rvn')
    command = [sys.executable, str(tools / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=180)
    if result.returncode or not result.stdout.endswith('Union contract passed\n'):
        raise AssertionError(result.stdout + result.stderr)
    print(json.dumps({'union_contract': 'passed', 'checks': [
        'construction and case identity', 'active None versus default carrier',
        'TryGetValue preserves outputs on mismatch', 'IUnion.Value boxed cases',
        'Option/Result propagation', 'same-type success and error payloads',
        'TaskOutcome completion/cancellation and inactive default', 'Void and nested outcomes', 'reference identity and value copies'
    ]}, indent=2))
