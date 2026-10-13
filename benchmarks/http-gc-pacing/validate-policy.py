#!/usr/bin/env python3
"""Qualify diagnostics/explicit ownership checks without promoting pacing."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'benchmarks/http-server'))
import benchmark
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--evaluation', type=Path, required=True)
p.add_argument('--ownership-recipe', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
e = json.loads(a.evaluation.read_text()); assert e['passed'] and not e['promotionEligible']
q = json.loads(a.ownership_recipe.read_text()); assert q['passed']
out = a.output.resolve();out.mkdir(parents=True,exist_ok=False)
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
report = dict(passed=False,promotionEligible=False,commands=[],http={},inputs={})
for value in [*e['inputs'],*q['inputsSha256'],__file__,str(a.evaluation),str(a.ownership_recipe),str(ROOT/'benchmarks/http-server/profile-gc.c')]:
    if Path(value).is_file():report['inputs'][str(Path(value).resolve())]=sha(value)
def run(cmd):
    r=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
    report['commands'].append(dict(command=cmd,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    if r.returncode:raise RuntimeError(r.stderr or r.stdout)
try:
    for variant in ['stress','paced']:
        cmd=next(c['command'][:] for c in e['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name==variant)
        cmd[cmd.index('-o')+1]=str(out/variant)
        cmd+=['-DNEOCLR_GC_PROFILE',str(ROOT/'benchmarks/http-server/profile-gc.c')]
        run(cmd)
        gc=out/(variant+'.json')
        result=benchmark.server_sample(['/usr/bin/env','NEOCLR_HTTP_PROFILE_OUTPUT='+str(gc),str(out/variant)],e['workload'])
        stats=json.loads(gc.read_text().splitlines()[0])
        assert stats['allocations']==stats['reclaimed'] and stats['failures']==0
        report['http'][variant]=dict(validation=result,gc=stats)
    probe=str(a.evaluation.resolve().parent/'paced-probe.c')
    for name in ['native-owned-body-test','native-idle-drain-test']:
        cmd=next(c['command'][:] for c in q['commands'] if c['label']==name+'-build')
        cmd=[probe if v.endswith('/aot-console/root-probe.c') else v for v in cmd]
        cmd[cmd.index('-o')+1]=str(out/name)
        run(cmd);run([str(out/name)])
    for path,digest in report['inputs'].items():assert sha(path)==digest,path
    report['passed']=True
finally:
    (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
