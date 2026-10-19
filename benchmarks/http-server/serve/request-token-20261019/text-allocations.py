#!/usr/bin/env python3
"""Reconcile native allocation kinds and UTF-8 buffers; never infer typed liveness."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('trace', type=Path)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
raw = a.trace.read_bytes()
data = json.loads(gzip.decompress(raw) if a.trace.suffix == '.gz' else raw)
assert data['dropped'] == data['droppedBytes'] == 0
assert sum(r['count'] for r in data['rows']) == data['allocations']
assert sum(r['count'] * r['size'] for r in data['rows']) == data['allocatedBytes']
names = {1: 'text', 2: 'objects', 3: 'byteArrays', 4: 'referenceSlotArrays',
         5: 'recordArrays', 6: 'internEntries'}

def total(rows):
    rows = list(rows)
    return dict(allocations=sum(r['count'] for r in rows),
                payloadBytes=sum(r['count'] * r['size'] for r in rows))

def frames(row):
    return [f['displayName'] for f in row['frames']]

report = dict(input=str(a.trace), sha256=hashlib.sha256(raw).hexdigest(),
              total=total(data['rows']),
              kinds={name: total(r for r in data['rows'] if r['kind'] == kind)
                     for kind, name in names.items()},
              utf8EncodedArrays=total(r for r in data['rows'] if r['kind'] == 3
                                     and 'neoCLR.Runtime.Utf8Encode' in frames(r)),
              utf8DecodeSnapshots=total(r for r in data['rows'] if r['kind'] == 3
                                       and 'T_System.Text_Utf8.M_Decode' in frames(r)),
              textCallers=[dict(count=r['count'], payloadSize=r['size'], frames=frames(r))
                           for r in data['rows'] if r['kind'] == 1],
              limitations='Whole process including startup/warmup/shutdown. Payload bytes exclude allocator headers. Four caller frames; not retained/live bytes, CPU attribution or throughput evidence. Kind 4 is reference-slot arrays, not string objects.')
a.output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k: v for k, v in report.items() if k not in ['textCallers', 'limitations']}, indent=2))
