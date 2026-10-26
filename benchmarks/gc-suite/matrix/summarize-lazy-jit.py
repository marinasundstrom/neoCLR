"""Summarize retained lazy-JIT medians without pooling dissimilar timing scopes."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--http',type=Path,required=True)
p.add_argument('--samples',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
http=json.loads(a.http.read_text());samples=json.loads(a.samples.read_text())
assert http['passed'] and samples['passed']
r=dict(http=[],samples=[],inputs={str(p.resolve()):hashlib.sha256(p.read_bytes()).hexdigest() for p in [a.http,a.samples,Path(__file__)]})
def scope(trials):
    assert len(trials)>=3 and all(t['passed'] for t in trials)
    keys=['readySeconds','firstResponseSeconds','readyThroughFirstResponseSeconds','coldObservedP99Seconds','requestsPerSecond','wholeProcessCpuSeconds','invocationCpuSeconds','allocations','allocatedBytes','collections']
    result={'runs':len(trials)}
    for key in keys:
        if all(key in t for t in trials):
            values=[t[key] for t in trials]
            result[key]=dict(median=statistics.median(values),min=min(values),max=max(values))
    lazy=[t['jit']['lazyJit'] for t in trials if t.get('jit',{}).get('lazyJit')]
    if lazy:
        result['lazyJit']={key:dict(median=statistics.median(t[key] for t in lazy),min=min(t[key] for t in lazy),max=max(t[key] for t in lazy)) for key in lazy[0]}
    return result
for kind,data in [('http',http),('samples',samples)]:
    for name in sorted({t['sample'] for t in data['trials']}):
        for collector in ['sync','live']:
            pair={}
            for mode in ['eager','lazy']:
                trials=[t for t in data['trials'] if t['sample']==name and t['collector']==collector and mode in t['mode'].split('-')]
                pair[mode]=scope(trials)
            changes={key:100*(pair['lazy'][key]['median']/pair['eager'][key]['median']-1) for key in pair['eager'] if key.endswith('Seconds') or key=='requestsPerSecond'}
            r[kind].append(dict(sample=name,collector=collector,**pair,changePercent=changes))
r['checkedHttpResponses']=sum(t['requests']+t['warmup'] for t in http['trials'])
r['standaloneInvocations']=len(samples['trials'])
a.output.write_text(json.dumps(r,indent=2)+'\n')
for kind in ['http','samples']:
    print(kind)
    for row in r[kind]:
        e=row['eager'];l=row['lazy']
        print(row['sample'],row['collector'],'startup',round(e['readySeconds']['median'],3),round(l['readySeconds']['median'],3),'change',round(row['changePercent']['readySeconds'],2))
        if kind=='http':
            print('  RPS',round(e['requestsPerSecond']['median']),round(l['requestsPerSecond']['median']),'change',round(row['changePercent']['requestsPerSecond'],2),
                  'cold ms',round(e['firstResponseSeconds']['median']*1000,2),round(l['firstResponseSeconds']['median']*1000,2))
