"""Summarize macOS wall-stack samples without treating waiting ticks as CPU time."""
import argparse
import hashlib
import json
from pathlib import Path
import re

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--measurement', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
data = json.loads(a.measurement.read_text())
assert data['passed'] and data.get('diagnosticOnly')
report = {'scope': __doc__, 'sample': data['sample'], 'trials': [], 'inputs': {}}
for trial in data['trials']:
    profile = trial['cpuProfile']
    path = Path(profile['path'])
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    assert profile['exitCode'] == 0 and digest == profile['sha256']
    text = path.read_text()
    section = text.split('Sort by top of stack, same collapsed (when >= 5):\n', 1)[1].split('Binary Images:', 1)[0]
    top = []
    for line in section.splitlines():
        match = re.fullmatch(r'\s+(.+?)\s+\(in (.+?)\)\s+(\d+)\s*', line)
        if match:
            top.append({'symbol': match[1], 'image': match[2], 'observations': int(match[3])})
        else:
            assert not line.strip(), line
    threads = [{'observations': int(m[1]), 'thread': m[2], 'description': m[3].strip()}
               for m in re.finditer(r'^\s+(\d+) Thread_(\d+)([^\n]*)$', text, re.MULTILINE)]
    assert top and threads
    report['trials'].append({'mode': trial['mode'], 'pair': trial['pair'],
                            'threads': threads, 'topFrames': top,
                            'minimumReportedObservations': 5,
                            'note': 'All threads, including waits. Missing functions are not zero; observations are not CPU percentages.'})
    report['inputs'][str(path)] = digest
for path in [a.measurement, Path(__file__)]:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
a.output.write_text(json.dumps(report, indent=2) + '\n')
