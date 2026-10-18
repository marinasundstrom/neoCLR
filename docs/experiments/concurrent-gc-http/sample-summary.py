#!/usr/bin/env python3
"""Summarize macOS sample call trees without treating inclusive counts as CPU time."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('sample', type=Path)
p.add_argument('--guest-build', type=Path, required=True)
a = p.parse_args()
build = json.loads(a.guest_build.read_text())
functions = json.loads(next(c['stdout'] for c in build['commands'] if c['phase'] == 'native-compilation'))['functions']
def readable(name):
    name = re.sub(r'NeoMetadata_[0-9A-F]+\.', '', name)
    return re.sub(r'(?<=[_])[0-9A-F]{2,}', lambda m: bytes.fromhex(m[0]).decode(errors='replace'), name)
names = {'neoclr_value_' + str(f['compiledIndex']): readable(f['name']) for f in functions}
text = a.sample.read_text().split('Call graph:\n', 1)[1].split('Total number in stack', 1)[0]
threads = []
stack = []
for line in text.splitlines():
    t = re.match(r'\s+(\d+) (Thread_.*)', line)
    if t:
        thread = dict(samples=int(t[1]), label=t[2], nodes=[])
        threads.append(thread)
        stack = []
        continue
    m = re.match(r'[ +!:|]*(\d+) (.+?)  \(in ', line)
    if not m:
        assert not line.strip(), line
        continue
    depth, count, symbol = m.start(1), int(m[1]), m[2]
    while stack and stack[-1]['depth'] >= depth:
        stack.pop()
    if stack:
        stack[-1]['self'] -= count
    node = dict(depth=depth, count=count, self=count, symbol=symbol,
                ancestors=[s['symbol'] for s in stack])
    thread['nodes'].append(node)
    stack.append(node)
for thread in threads:
    own, inclusive, allocation_callers = Counter(), Counter(), Counter()
    for node in thread.pop('nodes'):
        assert node['self'] >= 0, node
        own[node['symbol']] += node['self']
        if node['symbol'] not in node['ancestors']:
            inclusive[node['symbol']] += node['count']
        if node['symbol'] == 'neoclr_gc_allocate_v1':
            caller = next((s for s in reversed(node['ancestors']) if s.startswith('neoclr_value_')), '<native>')
            allocation_callers[caller + ' ' + names.get(caller, '<unmapped>')] += node['self']
    assert sum(own.values()) == thread['samples'], (thread['label'], sum(own.values()))
    thread['selfSamples'] = dict(own.most_common())
    thread['inclusiveManagedSamples'] = {k + ' ' + names[k]: v for k, v in inclusive.most_common() if k in names}
    thread['allocatorSelfByNearestManagedCaller'] = dict(allocation_callers.most_common())
# Independently check every entry in sample's collapsed leaf summary.
collapsed = a.sample.read_text().split('Sort by top of stack, same collapsed (when >= 5):', 1)[1].split('Binary Images:', 1)[0]
combined = Counter()
for thread in threads:
    combined.update(thread['selfSamples'])
for symbol, count in re.findall(r'^\s+(.+?)  \(in .*?\)\s+(\d+)\s*$', collapsed, re.M):
    assert combined[symbol] == int(count), (symbol, combined[symbol], count)
print(json.dumps(dict(method='Wall samples, including waiting threads. Exclusive counts partition each thread; inclusive managed counts overlap and must not be added. Allocator caller attribution is nearest generated frame, not allocated type. No CPU-duration or speedup inference.',
    inputs={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in [a.sample, a.guest_build, Path(__file__)]}, threads=threads), indent=2))
