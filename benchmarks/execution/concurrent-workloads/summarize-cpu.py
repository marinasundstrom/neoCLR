"""Summarize macOS wall-stack samples without treating waiting ticks as CPU time."""
import argparse
import collections
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
report = {'scope': __doc__, 'sample': data.get('sample', 'per-trial'), 'trials': [], 'inputs': {}}
for trial in data['trials']:
    profile = trial.get('stackProfile') or trial['cpuProfile']
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
    # Preserve caller offsets: sampled TLS work can otherwise hide which service
    # repeatedly resolves owner state. These remain wall-stack observations.
    callers = collections.Counter()
    stack = []
    unknown = 0
    graph = text.split('Sort by top of stack, same collapsed (when >= 5):', 1)[0]
    pattern = r'^([ +!:|]*)(\d+) (.+?)  \(in (.+?)\)(?: \+ ([0-9,]+))?  \[(?:0x[0-9a-f]+)(?:,0x[0-9a-f]+)*\]$'
    for line in graph.splitlines():
        if re.search(r'\bThread_\d+', line): stack = []
        match = re.fullmatch(pattern, line)
        if not match:
            # An unrecognized frame must not leave a previous sibling as the
            # apparent parent of a later TLS frame.
            prefix = re.match(r'^([ +!:|]*)(\d+) ', line)
            if prefix:
                while stack and stack[-1][0] >= len(prefix[1]): stack.pop()
            assert '_tlv_get_addr  (in ' not in line, line
            continue
        depth = len(match[1])
        while stack and stack[-1][0] >= depth: stack.pop()
        if match[3] == '_tlv_get_addr':
            count = int(match[2])
            if stack and stack[-1][0] == depth - 2:
                parent = stack[-1]
                callers[parent[1:]] += count
            else: unknown += count
        stack.append((depth, match[3], match[4], tuple(map(int, (match[5] or '0').split(',')))))
    report['trials'].append({'mode': trial['mode'], 'pair': trial.get('pair', trial.get('round')),
                            'sample': trial.get('sample', data.get('sample')),
                            'protocol': trial.get('protocol'),
                            'threads': threads, 'topFrames': top,
                            'tlsCallers': [{'symbol': key[0], 'image': key[1], 'offsets': key[2], 'observations': count} for key, count in callers.most_common()],
                            'unattributedTlsObservations': unknown,
                            'minimumReportedObservations': 5,
                            'note': 'All threads, including waits. Missing functions are not zero; observations are not CPU percentages.'})
    report['inputs'][str(path)] = digest
for path in [a.measurement, Path(__file__)]:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
a.output.write_text(json.dumps(report, indent=2) + '\n')
