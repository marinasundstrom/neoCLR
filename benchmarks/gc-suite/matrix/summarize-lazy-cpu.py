"""Summarize exported xctrace CPU cycle samples, excluding environment metadata.

Export only the cpu-profile table. Percentages are shares of sampled cycles in
the capture, not process utilization or whole-invocation phase percentages.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--input', type=Path, nargs='+', required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
report = dict(scope=__doc__, captures=[], inputs={})
for path in a.input:
    tree = ET.parse(path)
    assert not tree.findall('.//environment'), 'Export only cpu-profile tables'
    definitions = {node.attrib['id']: node for node in tree.iter() if 'id' in node.attrib}

    def resolve(node):
        return definitions[node.attrib['ref']] if 'ref' in node.attrib else node

    leaves, inclusive, threads = Counter(), Counter(), Counter()
    total = samples = unresolved = 0
    for row in tree.findall('.//row'):
        weight = int(resolve(row.find('cycle-weight')).text)
        stack = resolve(row.find('backtrace'))
        frames = [resolve(frame).get('name', '<unknown generated/native address>')
                  for frame in stack.findall('frame')]
        thread = resolve(row.find('thread')).get('fmt', '<unknown>')
        total += weight
        samples += 1
        threads[thread] += weight
        if frames:
            leaves[frames[0]] += weight
            for name in set(frames):
                inclusive[name] += weight
        else:
            unresolved += weight
    assert samples and total

    def rows(counter):
        return [dict(symbol=name, cycles=count, sampledCyclePercent=100*count/total)
                for name, count in counter.most_common()]

    report['captures'].append(dict(path=str(path), samples=samples, cycles=total,
                                   missingStackCycles=unresolved, leaf=rows(leaves),
                                   inclusive=rows(inclusive), threads=rows(threads)))
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
report['inputs'][str(Path(__file__).resolve())] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
a.output.write_text(json.dumps(report, indent=2)+'\n')
