"""Validate owner begin/capture nesting and joined-marker accounting."""
import argparse
import hashlib
import json
from pathlib import Path

def parse(stderr, candidate):
    lines=stderr.splitlines() if isinstance(stderr,str) else stderr
    rows={};marker=None
    for line in lines:
        if line.startswith('BEGIN_PHASE '):
            values=list(map(int,line.split()[1:]));assert len(values)==5 and min(values)>=0
            kind,count,wall,cpu,failures=values
            assert kind in (0,1) and kind not in rows and failures<=count
            rows[kind]=dict(count=count,wallNs=wall,cpuNs=cpu,failures=failures)
        elif line.startswith('MARKER_WORK '):
            values=list(map(int,line.split()[1:]));assert len(values)==6 and min(values)>=0 and marker is None
            marker=dict(zip(['cpuNs','headerBytes','payloadBytes','barrierWords','validJobs','invalidJobs'],values))
    assert set(rows)=={0,1} and marker is not None
    assert len(candidate)==6 and candidate[0]==candidate[1]+candidate[2] and candidate[5]==0
    assert rows[0]['failures']==rows[1]['failures']==0
    assert rows[0]['count']==rows[1]['count']==candidate[0]
    assert marker['validJobs']==candidate[0] and marker['invalidJobs']==0
    remainder={metric:rows[0][metric]-rows[1][metric] for metric in ['wallNs','cpuNs']}
    assert min(remainder.values())>=0
    return dict(begin=rows[0],capture=rows[1],beginRemainder=remainder,marker=marker)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--inputs',type=Path,nargs='+',required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();result=dict(passed=False,trials=[],inputs={})
    for path in a.inputs:
        data=json.loads(path.read_text());assert data['passed'] and data.get('diagnosticOnly')
        for trial in data['trials']:
            if trial['mode']=='sync':continue
            assert trial['exitCode']==0
            result['trials'].append(dict(sample=trial.get('sample',data.get('sample')),mode=trial['mode'],round=trial.get('round',trial.get('pair')),**parse(trial['stderr'],trial['candidateCounts'])))
        result['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    assert result['trials'];result['passed']=True
    a.output.write_text(json.dumps(result,indent=2)+'\n')
