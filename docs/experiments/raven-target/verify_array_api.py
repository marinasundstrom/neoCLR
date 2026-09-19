"""Execute the generic array API probes produced by Probe --array-api."""
import argparse
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--system', required=True, type=Path)
args = parser.parse_args()
expected = {
    'library-array-callbacks': '7\n42\nFirst\nSecond\n',
    'library-array-foreach': 'Parse\nDivide\nEquals\nToString\nCompareTo\n42\n1\n2\n3\n',
    'library-array-shapes': '0\n0\nBoolean elements\nSystem.Int32\nSystem.String\n0\n255\n65535\né\n42\n',
    'library-managed-array-metadata': '42\n2\n1\nSystem.Int32\n0\nEmpty\nLength\nCount\nItem\n4\n42\n8\n50\n2\n',
}
for name, output in expected.items():
    program = args.output.resolve() / (name + '.neoil')
    for operation in ('verify', 'run'):
        result = subprocess.run([str(args.runtime.resolve()), operation, str(program),
                                 '--system', str(args.system.resolve())],
                                capture_output=True, text=True, timeout=60)
        if result.returncode or operation == 'run' and result.stdout != output:
            raise AssertionError(f'{name} {operation}:\n{result.stdout}\n{result.stderr}')
    print(f'{name}: verified and executed')
