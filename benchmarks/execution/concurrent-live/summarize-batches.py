"""Separate uninstrumented process costs from instrumented blocking/CPU scopes."""
import argparse
import gzip
import json
from pathlib import Path
import statistics


def read(path, profiled):
    data=path.read_bytes()
    r=json.loads(gzip.decompress(data) if path.suffix=='.gz' else data)
    assert r['passed'] and len(r['trials'])==6
    runs=[c for c in r['commands'] if c['stderr'].startswith('LIVE ')]
    assert len(runs)==6
    rows=[]
    for trial,command in zip(r['trials'],runs):
        counters={line.split()[0]:list(map(int,line.split()[1:])) for line in command['stderr'].splitlines()}
        row=dict(trial)
        row.update(counters)
        live,batch,times=counters['LIVE'],counters['BATCH'],counters['TIMES']
        assert len(live)==len(batch)==len(times)==5
        if row['mode']=='sync':
            assert live==batch==times==[0]*5
        else:
            assert live[2:]==[1280000,10000,0] and times[0]>0
            if row['mode']=='batched':
                assert batch[0]>0 and batch[1]<=4096 and batch[2]<=64
            if profiled:
                assert times[3]==640000 and times[1]>=times[2]>0 and times[4]>0
            else:
                assert times[1:]==[0]*4 and batch[3:]==[0,0]
        if profiled:
            assert len(counters['PAUSES'])==5
        rows.append(row)
    return rows


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--costs',type=Path,required=True)
    p.add_argument('--profile',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    costs,profile=read(a.costs,False),read(a.profile,True)
    means={mode:statistics.mean(r['processCpuNanoseconds'] for r in costs if r['mode']==mode) for mode in ['sync','concurrent','batched']}
    result=dict(costs=costs,profile=profile,meanProcessCpuNanoseconds=means,
                batchVsUnbatchedCpuPercent=100*(means['batched']/means['concurrent']-1),
                scope='Native graph only. TIMES: marker thread CPU, guarded-write total wall, max wall, count, CPU. PAUSES: begin total/max wall, finish or sync-collect total/max wall, owner begin/finish CPU. BATCH: acquisitions, max bytes/reads per hold, max/total hold wall. All times nanoseconds; non-profile zero means disabled. Scopes overlap; do not sum wall clocks.')
    a.output.write_text(json.dumps(result,indent=2)+'\n')
