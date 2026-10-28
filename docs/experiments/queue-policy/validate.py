#!/usr/bin/env python3
"""Compare the real bounded FIFO scan with opt-in local/global queues."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);here=Path(__file__).resolve().parent;root=here.parents[2];native=root/'runtime/native'
report=dict(passed=False,platform=platform.platform(),commands=[],samples=[])
def run(cmd):
 cmd=list(map(str,cmd));r=subprocess.run(cmd,capture_output=True,text=True,timeout=60)
 report['commands'].append(dict(command=cmd,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr));r.check_returncode();return r.stdout
try:
 cc=[subprocess.check_output(['xcrun','--find','clang'],text=True).strip(),'-isysroot',subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()]
 sources=[native/'workers/native-workers.c',native/'gc/native-gc.c',native/'gc/root-probe.c']
 flags=['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-I',native/'workers']
 for policy in ('fifo','local'):
  defs=[] if policy=='fifo' else ['-DNEOCLR_WORKER_QUEUES']
  for variant,extra in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
   exe=out/f'{policy}-{variant}-test';run([*cc,*flags,*defs,*extra,here/'test.c',*sources,'-o',exe]);run([exe])
  run([*cc,*flags,*defs,here/'benchmark.c',*sources,'-o',out/f'{policy}-benchmark'])
 # 3 interleaved pairs, same fixed work per pair. Tiny screens 65,536 jobs;
 # skew/blocking screens 1,024 jobs. No tracing or sanitizers during timing.
 for workers in (1,2,4):
  for shape in range(4):
   for pair in range(3):
    for policy in (('fifo','local') if pair%2==0 else ('local','fifo')):
     sample=json.loads(run([out/f'{policy}-benchmark',workers,shape,4096 if shape<2 else 64]))
     sample.update(policy=policy,pair=pair);report['samples'].append(sample)
 paths=[*sources,*here.glob('*.c'),Path(__file__),*list((native/'workers').glob('*.h')),*list((native/'gc').glob('*.h'))]
 report['inputs']={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in paths}
 report['passed']=True
finally:(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS queue policy contract and 72 paired timing invocations')
