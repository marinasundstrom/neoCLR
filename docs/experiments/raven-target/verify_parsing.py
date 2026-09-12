"""Verify Int32 parsing, propagation and conditional-output rejection fixtures."""
import argparse
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('probe', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
expected = 'Parsed\n42\nParsed\n-2147483648\nParsed\n2147483647\nParsed\n7\nOverflow\nOverflow\nInvalid format\nInvalid format\nInvalid format\nInvalid format\n'
for command in ('verify', 'run'):
    run = subprocess.run([str(args.runtime.resolve()), command, str((args.probe / 'Parsing.neoil').resolve())],
                         capture_output=True, text=True, timeout=60)
    if run.returncode or (command == 'run' and run.stdout != expected):
        raise AssertionError((command, run.stdout, run.stderr))
for name in ('IgnoredExtraction', 'InvertedExtraction', 'UninitializedError'):
    assert (args.probe / (name + '.rejected.txt')).is_file()
    assert not (args.probe / (name + '.neoil')).exists()
assert (args.probe / 'WrongCarrier.rejected.txt').is_file()
print('Int32 parsing, early error propagation and invalid output reads: passed')
