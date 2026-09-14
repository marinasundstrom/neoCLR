"""Verify and run the direct-runtime preview samples against the supplied library."""
import argparse
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--system', required=True, type=Path)
parser.add_argument('--samples', required=True, type=Path)
args = parser.parse_args()
expected = {
    'type-categories.neoil': '42\n7\n9\n',
    'generic_managed_array.neoil': '42\n',
    'native_memory.neoil': '42\n',
    'result-void.neoil': 'Completed\nNot saved\n',
}
for name, output in expected.items():
    common = [str((args.samples / name).resolve()), '--system', str(args.system.resolve())]
    subprocess.run([str(args.runtime.resolve()), 'verify', *common], check=True)
    result = subprocess.run([str(args.runtime.resolve()), 'run', *common],
                            check=True, capture_output=True, text=True)
    if result.stdout != output or result.stderr:
        raise SystemExit(f'{name}: unexpected output: {result.stdout!r}, stderr: {result.stderr!r}')
    print(f'{name}: passed')
