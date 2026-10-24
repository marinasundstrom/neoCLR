"""Summarize bounded collector wall time, never label it application CPU share."""
import argparse,gzip,json,math,statistics
from pathlib import Path
PHASES=['index','roots','trace','cleanup','sweep']
def read(p):
 p=Path(p);return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def summarize(plain,complex_,phase_plain,phase_complex):
 out={}
 for sample,raw,phased in [('plain',plain,phase_plain),('json',complex_,phase_complex)]:
  assert raw['passed'] and phased['passed']
  out[sample]={}
  for mode in ['aot','jit']:
   base=[r for r in raw['trials'] if r['mode']==mode];rows=[r for r in phased['trials'] if r['mode']==mode]
   assert len(base)==len(rows)==3 and all(r['count']==1024 and not r['exitCode'] and not r['gc'][1] for r in base+rows)
   per_phase=[];pauses=[]
   for row in rows:
    lines=''.join(row['stderr']).splitlines()
    phases=list(map(int,next(s for s in lines if s.startswith('PHASES ')).split()[1:]))
    buckets=list(map(int,next(s for s in lines if s.startswith('PAUSES ')).split()[1:]))
    assert len(phases)==6 and len(buckets)==65 and phases[5]==row['gc'][0]
    assert sum(buckets[1:])==row['gc'][0] and sum(phases[:5])<=row['gc'][4]
    per_phase.append([n/1024/1000 for n in phases[:5]])
    upper={}
    for q in [.5,.95,.99]:
     target=math.ceil(q*sum(buckets[1:]));total=0
     for i,n in enumerate(buckets[1:]):
      total+=n
      if total>=target:upper[str(q)]=(2**(i+1))/1000;break
    pauses.append({'maximumMicroseconds':buckets[0]/1000,'bucketUpperMicroseconds':upper})
   means=[statistics.mean(row[i] for row in per_phase) for i in range(5)]
   out[sample][mode]={'collectorWallMicrosecondsPerRequest':[r['gc'][4]/1024/1000 for r in base],
    'phaseMeanMicrosecondsPerRequest':dict(zip(PHASES,means)),
    'phaseSharePercent':dict(zip(PHASES,[v/sum(means)*100 for v in means])),
    'instrumentedCollectorWallMicrosecondsPerRequest':[r['gc'][4]/1024/1000 for r in rows],
    'instrumentedPausesIncludingStartup':pauses}
 return out
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('inputs',nargs=4);p.add_argument('--output',required=True);a=p.parse_args()
 Path(a.output).write_text(json.dumps(summarize(*(read(p) for p in a.inputs)),indent=2)+'\n')
