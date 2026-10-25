"""Stress code publication; optional negative control is expected to fail on affected hosts."""
import argparse, gzip, hashlib, json, os, platform, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--rounds',type=int,default=20)
p.add_argument('--negative-control',action='store_true')
a=p.parse_args();assert a.rounds>0
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
runner=a.runner.resolve();r=dict(passed=False,system=platform.platform(),trials=[],inputs={str(runner):hashlib.sha256(runner.read_bytes()).hexdigest()})
try:
 for level in ['none','speed']:
  for i in range(a.rounds):
   env=dict(os.environ,NEOCLR_TEST_OPT_LEVEL=level)
   env.pop('NEOCLR_TEST_SKIP_JIT_ICACHE',None)
   if a.negative_control:env['NEOCLR_TEST_SKIP_JIT_ICACHE']='1'
   cmd=[str(runner),'mutation_store::tests::repeated_stack_transfer_code_publication','--exact','--nocapture']
   x=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=60)
   r['trials'].append(dict(command=cmd,level=level,iteration=i,skipInvalidation=a.negative_control,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
   if not a.negative_control:assert x.returncode==0 and '1 passed' in x.stdout
 r['passed']=all(t['exitCode']==0 for t in r['trials']) if not a.negative_control else any(t['exitCode']!=0 for t in r['trials'])
 assert r['passed'],'Negative control did not reproduce; this is not proof the defect is absent'
finally:(out/'results.json.gz').write_bytes(gzip.compress((json.dumps(r,indent=2)+'\n').encode(),mtime=0))
