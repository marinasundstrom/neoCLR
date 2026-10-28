#!/usr/bin/env python3
"""Separate macOS stack samples of TCP resume under CPU/allocation pressure."""
import argparse
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binaries', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
report = []
for mode in ('sync', 'concurrent'):
    command = [str((args.binaries / f'{mode}-optimized').resolve()), '2', '4096']
    sample_command = None
    with (args.output / f'{mode}.json').open('w') as stream:
        process = subprocess.Popen(command, stdout=stream, stderr=subprocess.PIPE, text=True)
        sample_command = ['sample', str(process.pid), '2', '1', '-file',
                          str((args.output / f'{mode}.sample.txt').resolve())]
        capture = subprocess.run(sample_command, capture_output=True, text=True, timeout=30)
        _, error = process.communicate(timeout=60)
        report.append(dict(command=command, exitCode=process.returncode, stderr=error,
                           sampleCommand=sample_command, sampleExitCode=capture.returncode,
                           sampleStderr=capture.stderr, sampleStdout=capture.stdout))
        assert process.returncode == 0 and capture.returncode == 0, report[-1]
(args.output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS separate I/O resume stack captures')
