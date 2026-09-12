"""Verify UTF-8 slicing, propagation and conditional-output rejection fixtures."""
import argparse
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('probe', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
expected = 'Sliced\né\nSliced\n😀\nSliced\n\nInvalid boundary\nInvalid boundary\nOut of range\nOut of range\nOut of range\nOut of range\nOut of range\nSliced\n\n'
for command in ('verify', 'run'):
    run = subprocess.run([str(args.runtime.resolve()), command, str((args.probe / 'Slices.neoil').resolve())],
                         capture_output=True, text=True, timeout=60)
    if run.returncode or (command == 'run' and run.stdout != expected):
        raise AssertionError((command, run.stdout, run.stderr))
for name in ('IgnoredExtraction', 'InvertedExtraction', 'UninitializedError'):
    assert (args.probe / (name + '.rejected.txt')).is_file()
    assert not (args.probe / (name + '.neoil')).exists()
assert (args.probe / 'WrongCarrier.rejected.txt').is_file()
print('UTF-8 slicing, early error propagation and invalid output reads: passed')
