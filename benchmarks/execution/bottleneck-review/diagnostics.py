#!/usr/bin/env python3
"""Collect current worker stacks and plain/JSON lookup counts after capacity trials."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

root=Path(__file__).resolve().parents[3]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--directory',type=Path,required=True)
a=p.parse_args();d=a.directory.resolve()
peer=json.loads((d/'peers/comparison.json').read_text());assert peer['passed']
binary=d/'peers/neoclr-4'
command=next(x['command'] for x in peer['commands'] if str(binary) in x['command'] and '-o' in x['command'])
build=dict(passed=True,workers=4,commands=[dict(command=command)],inputs=peer['inputs'],
           scope='Current production sources, uninstrumented peer binary; reused only in separate profiling run.')
(d/'worker-build.json').write_text(json.dumps(build,indent=2)+'\n')
cmds=[
 ['python3',str(root/'benchmarks/http-server/primitive-comparison/stack-profile/capture.py'),'--build-report',str(d/'worker-build.json'),'--client',str(d/'peers/client'),'--output',str(d/'worker-profile')],
 ['python3',str(root/'benchmarks/http-server/primitive-comparison/stack-profile/analyze.py'),'--capture',str(d/'worker-profile/capture.json'),'--build-report',str(d/'worker-build.json'),'--compilation',str(root/'target/root-map/build.json'),'--object',str(root/'target/root-map/candidate.o'),'--output',str(d/'worker-analysis')],
 ['python3',str(root/'benchmarks/http-server/primitive-comparison/peer-refresh/profile.py'),'--build',str(root/'target/primitive-comparison-build/build.json'),'--object',str(root/'target/root-map/candidate.o'),'--client',str(d/'peers/client'),'--output',str(d/'gc-timing')],
]
for sample in ['plain','json']:
 cmds.append(['python3',str(root/'benchmarks/execution/lookup-profile/counts.py'),'--sample',sample,'--requests','4','12','--output',str(d/(sample+'-counts'))])
r=dict(passed=False,commands=[],inputs={})
try:
 for cmd in cmds:
  for arg in cmd:
   path=Path(arg)
   if path.is_file():r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
  x=subprocess.run(cmd,capture_output=True,text=True,timeout=240)
  r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
  (d/'diagnostics.json').write_text(json.dumps(r,indent=2)+'\n')
  x.check_returncode()
  print(x.stdout[-1500:],flush=True)
 r['passed']=True
finally:
 (d/'diagnostics.json').write_text(json.dumps(r,indent=2)+'\n')
