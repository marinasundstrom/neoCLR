"""Validate finish wall/CPU distributions and preserve same-event outlier components."""
import argparse
import importlib.util
import json
from pathlib import Path

NAMES=['initial-wait','log-drain-mark','remark-dispatch-wait','final-collection','dispose','whole-finish']

def parse(stderr):
    lines=stderr.splitlines() if isinstance(stderr,str) else stderr
    rows={};event=None
    for line in lines:
        if line.startswith('FINISH_PHASE '):
            values=list(map(int,line.split()[1:])); assert len(values)==70 and all(x>=0 for x in values)
            kind,count,wall,wall_max,cpu,cpu_max,*bins=values
            assert kind in range(6) and kind not in rows and sum(bins)==count
            assert wall_max<=wall<=count*wall_max and cpu_max<=cpu<=count*cpu_max
            low=sum(n*(0 if i==0 else 1<<i) for i,n in enumerate(bins))
            high=sum(n*((1<<(i+1))-1) for i,n in enumerate(bins))
            assert low<=wall<=high
            if count:
                last=max(i for i,n in enumerate(bins) if n)
                assert (0 if last==0 else 1<<last)<=wall_max<1<<(last+1)
            rows[kind]=dict(count=count,wallNs=wall,maximumWallNs=wall_max,cpuNs=cpu,maximumCpuNs=cpu_max,bins=bins)
        elif line.startswith('FINISH_MAX '):
            assert event is None
            values=list(map(int,line.split()[1:]));assert len(values)==13 and all(x>=0 for x in values)
            event=dict(mask=values[0],phases=[dict(wallNs=values[1+i*2],cpuNs=values[2+i*2]) for i in range(6)])
    assert set(rows)==set(range(6)) and event is not None
    count=rows[5]['count']
    assert all(rows[i]['count']==count for i in [0,1,3,4]) and rows[2]['count']<=count
    assert event['mask'] in ([59,63] if count else [0])
    for metric in ['wallNs','cpuNs']:
        assert sum(rows[i][metric] for i in range(5))<=rows[5][metric]
        assert sum(event['phases'][i][metric] for i in range(5))<=event['phases'][5][metric]
    assert event['phases'][5]['wallNs']==rows[5]['maximumWallNs']
    for i,phase in enumerate(event['phases']):
        assert phase['wallNs']<=rows[i]['maximumWallNs'] and phase['cpuNs']<=rows[i]['maximumCpuNs']
        if not event['mask'] & (1<<i): assert phase==dict(wallNs=0,cpuNs=0)
    return rows,event

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--inputs',type=Path,nargs='+',required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    spec=importlib.util.spec_from_file_location('owner',Path(__file__).with_name('summarize-owner-pauses.py'))
    owner=importlib.util.module_from_spec(spec);spec.loader.exec_module(owner)
    result=dict(passed=False,scope=__doc__,trials=[])
    for path in a.inputs:
        data=json.loads(path.read_text());assert data['passed'] and data.get('diagnosticOnly')
        for trial in data['trials']:
            if trial['mode']=='sync':continue
            rows,event=parse(trial['stderr']);outer=owner.parse(trial['stderr'])
            assert rows[5]['count']==outer[2]['count']==trial['candidateCounts'][1]
            assert rows[5]['maximumWallNs']<=outer[2]['maximumNs']
            result['trials'].append(dict(sample=trial.get('sample',data.get('sample')),mode=trial['mode'],round=trial.get('round',trial.get('pair')),
                phases={NAMES[k]:dict(**r,quantileBounds={str(p):owner.quantile(r['bins'],p) for p in [50,95,99]}) for k,r in rows.items()},slowestFinish=event))
    result['passed']=True
    a.output.write_text(json.dumps(result,indent=2)+'\n')
