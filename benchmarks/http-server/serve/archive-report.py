#!/usr/bin/env python3
"""Retain complete reports losslessly and publish compact, linked summaries."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('source', type=Path)
p.add_argument('destination', type=Path)
a = p.parse_args()
data = a.source.read_bytes()
compressed = gzip.compress(data, mtime=0)
a.destination.parent.mkdir(parents=True, exist_ok=True)
a.destination.with_suffix('.json.gz').write_bytes(compressed)
report = json.loads(data)
report.pop('build', None)
report['rawReport'] = dict(file=a.destination.with_suffix('.json.gz').name,
                         sha256=hashlib.sha256(compressed).hexdigest(),
                         uncompressedSha256=hashlib.sha256(data).hexdigest())
for trial in report.get('trials', [report]):
    for key in ('measurement', 'warmup'):
        if key in trial:
            trial[key].pop('requests', None)
            trial[key].pop('resources', None)
a.destination.write_text(json.dumps(report, indent=2)+'\n')
