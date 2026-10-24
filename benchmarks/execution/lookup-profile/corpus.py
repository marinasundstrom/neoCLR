#!/usr/bin/env python3
"""Compare shared native lookup work in checked ordinary task/collection consumers."""
import json
from pathlib import Path
import subprocess
root=Path(__file__).resolve().parents[3]
out=root/'target/lookup-profile/retained-corpus';out.mkdir(exist_ok=False)
prior=json.loads((root/'target/gc-mode-comparison-final/results.json').read_text())
counts=json.loads((root/'target/lookup-profile/retained-plain/results.json').read_text())
host=root/'target/lookup-profile/retained-plain/host.dylib'
r=dict(passed=False,fields=counts['fields'],builds=[],trials=[])
try:
    for case in prior['cases']:
        sample=case['sample']
        old=root/'target/root-map/candidate.o'
        exe=out/sample
        cmd=counts['builds'][0]['command'].copy()
        cmd=[str(root/'target/gc-mode-comparison-final'/(sample+'.o')) if x==str(old) else x for x in cmd]
        cmd[cmd.index('-o')+1]=str(exe)
        x=subprocess.run(cmd,capture_output=True,text=True,timeout=90)
        r['builds'].append(dict(command=cmd,exitCode=x.returncode,stderr=x.stderr));x.check_returncode()
        jit=next(x['command'].copy() for x in case['runs'] if x.get('mode')=='jit' and not x.get('diagnostic'))
        jit[5]=str(host)
        for mode,command in [('aot',[str(exe)]),('jit',jit)]:
            x=subprocess.run(command,capture_output=True,text=True,timeout=60)
            row=dict(sample=sample,mode=mode,command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr)
            r['trials'].append(row);x.check_returncode()
            assert x.stdout==case['expected']
            row['counters']=list(map(int,next(line for line in x.stderr.splitlines() if line.startswith('COUNTS ')).split()[1:]))
            if mode=='jit':assert json.loads(next(line[11:] for line in x.stderr.splitlines() if line.startswith('JIT_RESULT ')))['guestExitCode']==0
    r['passed']=True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
