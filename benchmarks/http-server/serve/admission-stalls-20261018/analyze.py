#!/usr/bin/env python3
"""Align diagnostic GC/scheduler intervals with every offered HTTP arrival."""
import argparse,json,math
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('directory',type=Path)
a=p.parse_args();t=json.loads((a.directory/'trace.json').read_text());m=json.loads((a.directory/'requests/results.json').read_text())['measurement']
assert t['dropped']==0
start=m['startUnixSeconds'];duration=m['durationSeconds']
drift=(t['endUnixNs']-t['originUnixNs'])-(t['endMonoNs']-t['originMonoNs'])
assert abs(drift)<1_000_000,'Clock mapping changed by >=1 ms'
events=[dict(startSeconds=(t['originUnixNs']+e[0]-t['originMonoNs'])/1e9-start,wallSeconds=e[1]/1e9,cpuSeconds=e[2]/1e9,kind=e[3],status=e[4]) for e in t['events']]
bins=[]
for i in range(math.ceil(duration*10)):
 left=i/10;right=min(duration,(i+1)/10);rs=[r for r in m['requests'] if left<=r['intendedSeconds']<right]
 overlapping=[e for e in events if e['startSeconds']<right and e['startSeconds']+e['wallSeconds']>left]
 bins.append(dict(startSeconds=left,offered=len(rs),misses=sum(r['result']!='ok' for r in rs),maxDispatchLagSeconds=max((r['dispatchLagSeconds'] for r in rs),default=0),
     maxOfferedLatencySeconds=max((r.get('offeredLatencySeconds',0) for r in rs),default=0),
     tracedGcOverlapSeconds=sum(max(0,min(right,e['startSeconds']+e['wallSeconds'])-max(left,e['startSeconds'])) for e in overlapping if e['kind']==0),
     tracedWaitOverlapSeconds=sum(max(0,min(right,e['startSeconds']+e['wallSeconds'])-max(left,e['startSeconds'])) for e in overlapping if e['kind']==1)))
assert sum(b["offered"] for b in bins)==m["offered"]
assert sum(b["misses"] for b in bins)==m["offered"]-m["outcomes"].get("ok",0)
inside=[e for e in events if 0<=e['startSeconds']<duration]
summary=dict(outcomes=m['outcomes'],clockDriftNs=drift,dropped=t['dropped'],totalsIncludingStartup=t['totals'],
    measurementWaitCalls=sum(e['kind']==1 for e in inside),measurementWaitSeconds=sum(e['wallSeconds'] for e in inside if e['kind']==1),
    longestIntervals=sorted(inside,key=lambda e:-e['wallSeconds'])[:20],bins=bins,
    method='100 ms bins by intended arrival. GC/acquire events below 100 us omitted; every wait recorded. Overlap is wall time, not CPU time; acquire includes GC. Unix/monotonic mapping checked for <1 ms endpoint drift. Instrumentation changes timing; correlation is not proof of causation.')
(a.directory/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print({k:v for k,v in summary.items() if k not in ('bins','longestIntervals')})
for row in sorted(bins,key=lambda x:-x['misses'])[:5]:print(row)
