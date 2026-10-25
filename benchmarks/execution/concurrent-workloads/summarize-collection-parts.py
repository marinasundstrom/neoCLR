"""Validate disjoint final-collection attribution and retain the unmeasured remainder."""
import argparse
import importlib.util
import json
from pathlib import Path

NAMES=['build-index','frame-roots','snapshot-seed','sweep']

def parse(stderr, final):
    lines=stderr.splitlines() if isinstance(stderr,str) else stderr
    rows={}
    for line in lines:
        if not line.startswith('COLLECTION_PART '):continue
        v=list(map(int,line.split()[1:]));assert len(v)==70 and all(x>=0 for x in v)
        kind,count,wall,wall_max,cpu,cpu_max,*bins=v
        assert kind in range(4) and kind not in rows and sum(bins)==count
        assert wall_max<=wall<=count*wall_max and cpu_max<=cpu<=count*cpu_max
        assert sum(n*(0 if i==0 else 1<<i) for i,n in enumerate(bins))<=wall<=sum(n*((1<<(i+1))-1) for i,n in enumerate(bins))
        if count:
            b=max(i for i,n in enumerate(bins) if n)
            assert (0 if b==0 else 1<<b)<=wall_max<1<<(b+1)
        rows[kind]=dict(count=count,wallNs=wall,maximumWallNs=wall_max,cpuNs=cpu,maximumCpuNs=cpu_max,bins=bins)
    assert set(rows)==set(range(4))
    assert rows[2]['count']==rows[3]['count']==final['count']
    remainder={m:final[m]-sum(r[m] for r in rows.values()) for m in ['wallNs','cpuNs']}
    assert min(remainder.values())>=0
    return rows,remainder

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--inputs',type=Path,nargs='+',required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    spec=importlib.util.spec_from_file_location('finish',Path(__file__).with_name('summarize-finish-phases.py'))
    finish=importlib.util.module_from_spec(spec);spec.loader.exec_module(finish)
    result=dict(passed=False,scope=__doc__,trials=[])
    for path in a.inputs:
        data=json.loads(path.read_text());assert data['passed'] and data.get('diagnosticOnly')
        for trial in data['trials']:
            if trial['mode']=='sync':continue
            phases,event=finish.parse(trial['stderr'])
            assert phases[3]['count']==trial['candidateCounts'][1]
            rows,remainder=parse(trial['stderr'],phases[3])
            result['trials'].append(dict(sample=trial.get('sample',data.get('sample')),mode=trial['mode'],round=trial.get('round',trial.get('pair')),parts={NAMES[k]:v for k,v in rows.items()},finalCollection=phases[3],remainder=remainder))
    result['passed']=True
    a.output.write_text(json.dumps(result,indent=2)+'\n')
