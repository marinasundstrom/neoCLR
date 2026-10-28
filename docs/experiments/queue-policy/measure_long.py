#!/usr/bin/env python3
"""Longer controls for the variable two-worker tiny-job cells."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binaries',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
report=dict(passed=False,samples=[],commands=[],binaries={p:hashlib.sha256((a.binaries/(p+'-benchmark')).read_bytes()).hexdigest() for p in ('fifo','local')})
try:
 for shape in (0,1):
  for pair in range(5):
   for policy in (('fifo','local') if pair%2==0 else ('local','fifo')):
    command=[str((a.binaries/(policy+'-benchmark')).resolve()),'2',str(shape),'65536']
    result=subprocess.run(command,capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=command,exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr));result.check_returncode()
    sample=json.loads(result.stdout);sample.update(policy=policy,pair=pair);report['samples'].append(sample)
 report['passed']=True
finally:(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS 20 longer two-worker controls')
