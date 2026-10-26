"""Summarize paired lazy-JIT factors without pooling collectors or hiding ranges."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--measurement',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
r=json.loads(a.measurement.read_text())
assert r['passed'] and r['rounds']>=3 and not r.get('diagnosticOnly')
metrics=['readySeconds','firstResponseSeconds','readyThroughFirstResponseSeconds',
         'requestsPerSecond','clientP50Seconds','clientP99Seconds','invocationCpuSeconds',
         'allocations','allocatedBytes','collections','peakLiveBytes','peakArenaBytes']
def distribution(values):
    return dict(median=statistics.median(values),minimum=min(values),maximum=max(values),values=values)
groups={}
for t in r['trials']:
    assert t['passed'] and t['jit']['guestExitCode']==0
    groups.setdefault((t['sample'],t['protocol'],t['mode']),[]).append(t)
rows=[]
for (sample,protocol,mode),trials in sorted(groups.items()):
    assert len(trials)==r['rounds'] and len({t['round'] for t in trials})==r['rounds']
    row=dict(sample=sample,protocol=protocol,mode=mode,trials=len(trials),
             metrics={m:distribution([t[m] for t in trials]) for m in metrics})
    lazy=[t['jit']['lazyJit'] for t in trials]
    if lazy[0] is not None:
        row['lazyJit']={m:distribution([t[m] for t in lazy]) for m in lazy[0]}
        row['resolverMinusWorkerSeconds']=distribution([
            t['mutatorCompilationWaitSeconds']-t['firstCallCompilationSeconds'] for t in lazy])
    rows.append(row)
comparisons=[]
# Compare one factor at a time. Each ratio pairs the same round and workload.
for sample,protocol,_ in groups:
    for collector in ['sync','live']:
        for control,candidate,factor in [
            ('system-control','lazy','placement-lazy-default'),
            ('control','lazy','binding-system'),
            ('arena-control','arena','binding-arena'),
            ('control','arena-control','placement-lazy-unbound'),
            ('lazy','arena','placement-lazy-bound'),
            ('eager','arena-eager','placement-eager'),
            ('eager','control','lazy-system-unbound'),
            ('arena-eager','arena-control','lazy-arena-unbound'),
            ('arena-eager','arena','lazy-arena-bound')]:
            left=f'{control}-{collector}-jit';right=f'{candidate}-{collector}-jit'
            if (sample,protocol,left) not in groups or (sample,protocol,right) not in groups:continue
            key=(sample,protocol,left,right)
            if any(tuple(c[k] for k in ['sample','protocol','control','candidate'])==key for c in comparisons):continue
            by_round={t['round']:t for t in groups[(sample,protocol,left)]}
            pairs=groups[(sample,protocol,right)]
            comparisons.append(dict(sample=sample,protocol=protocol,collector=collector,factor=factor,
                control=left,candidate=right,percentChange={m:distribution([
                    100*(t[m]/by_round[t['round']][m]-1) for t in pairs]) for m in metrics if all(t[m]>0 for t in by_round.values())}))
    for prefix in ['eager','arena-eager','control','lazy','arena-control','arena']:
        left=f'{prefix}-sync-jit';right=f'{prefix}-live-jit'
        key=(sample,protocol,left,right)
        if (sample,protocol,left) not in groups or (sample,protocol,right) not in groups:continue
        if any(tuple(c[k] for k in ['sample','protocol','control','candidate'])==key for c in comparisons):continue
        by_round={t['round']:t for t in groups[(sample,protocol,left)]}
        comparisons.append(dict(sample=sample,protocol=protocol,factor='collector',control=left,candidate=right,
            percentChange={m:distribution([100*(t[m]/by_round[t['round']][m]-1)
                for t in groups[(sample,protocol,right)]]) for m in metrics if all(t[m]>0 for t in by_round.values())}))
report=dict(passed=True,scope=__doc__,rounds=r['rounds'],rows=rows,comparisons=comparisons,
    checkedResponses=sum(t['requests']+t['warmup'] for t in r['trials']),
    caveats=['Ranges retain every trial; paired ratios are descriptive, not confidence intervals.',
             'Invocation CPU includes cold compilation and collector threads, not just timed traffic.',
             'Loopback clients and configured delays do not establish saturated server capacity.'],
    inputs={str(path.resolve()):hashlib.sha256(path.read_bytes()).hexdigest() for path in [a.measurement,Path(__file__)]})
a.output.write_text(json.dumps(report,indent=2)+'\n')
