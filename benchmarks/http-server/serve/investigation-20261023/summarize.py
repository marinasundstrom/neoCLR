#!/usr/bin/env python3
"""Reconcile retained allocation traces and summarize separate uninstrumented loads."""
import argparse
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--evidence', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
def read(name):
    path = a.evidence/(name+'.json.gz')
    return json.loads(gzip.decompress(path.read_bytes()))
short, long = read('callers-200'), read('callers-400')
for trace in (short,long):
    assert trace['dropped'] == trace['droppedBytes'] == 0
    assert sum(r['count'] for r in trace['rows']) == trace['allocations']
    assert sum(r['count']*r['size'] for r in trace['rows']) == trace['allocatedBytes']
for name,count in [('requests-200',200),('requests-400',400)]:
    run = read(name)
    assert run['passed'] and run['measurement']['outcomes'] == {'ok':count}
    assert run['warmup']['outcomes'] == {'ok':20}
def by_kind(trace,kind,bytes=False):
    return sum(r['count']*(r['size'] if bytes else 1) for r in trace['rows'] if r['kind']==kind)
report = dict(nativeIncrementPerRequest={k:(long[k]-short[k])/200 for k in ('allocations','allocatedBytes')},
              nativeKinds={}, loads=[], stages=read('stages')['samples'])
for kind,name in enumerate(['text','objects','byteArrays','referenceSlotArrays','recordArrays','internEntries'],1):
    report['nativeKinds'][name] = dict(allocations=(by_kind(long,kind)-by_kind(short,kind))/200,
                                     payloadBytes=(by_kind(long,kind,True)-by_kind(short,kind,True))/200)
for name in ['load400a','load800','load400b','two-a','two-b']:
    run=read(name); assert run['passed']
    m=run['measurement']
    report['loads'].append(dict(name=name, **{k:m[k] for k in (
        'startUnixSeconds','rate','durationSeconds','offered','admitted','outcomes','completedWithinWindow',
        'successfulCompletionsPerSecond','offeredLatencySeconds','serverCpuSecondsInWindow','sampledPeakRssKiB')}))
sample=read('sample-summary')['threads'][0]
report['mainThreadSamples']=sample['samples']
report['mainThreadExclusivePercent']={k:100*v/sample['samples'] for k,v in list(sample['selfSamples'].items())[:10]}
promise=[r for r in short['rows'] if any('Tasks_Promise' in f['displayName'] and '..ctor' in f['displayName'] for f in r['frames'][:3]) and r['kind'] in (3,4,5)]
report['promiseResultArrays200']=dict(allocations=sum(r['count'] for r in promise),
    payloadBytes=sum(r['count']*r['size'] for r in promise))
report['inputs']={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in a.evidence.iterdir() if p.is_file()}
a.output.write_text(json.dumps(report,indent=2)+'\n')
