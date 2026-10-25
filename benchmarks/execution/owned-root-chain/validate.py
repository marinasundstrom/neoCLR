#!/usr/bin/env python3
"""Owned ancestor proof, mutable fallback and worker/pacing regression checks."""
import argparse
import hashlib
import json
from pathlib import Path
import signal
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--scalar-validation',action='store_true');p.add_argument('--split-validation',action='store_true');a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
base=root/'docs/experiments/aot-console'
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
cc=recipe[:recipe.index('-std=c11')]
r=dict(passed=False,scalarValidation=a.scalar_validation,splitValidation=a.split_validation,commands=[],inputs={})
def run(cmd,expected=0):
 x=subprocess.run(list(map(str,cmd)),capture_output=True,text=True,timeout=60)
 r['commands'].append(dict(command=list(map(str,cmd)),exitCode=x.returncode,expected=expected,stdout=x.stdout,stderr=x.stderr))
 (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
 assert x.returncode==expected,r['commands'][-1]
try:
 for sanitized in [False,True]:
  flags=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-I',str(base)]
  if a.scalar_validation:flags+=['-DNEOCLR_GC_SCALAR_STORAGE_VALIDATION']
  if a.split_validation:flags+=['-DNEOCLR_GC_SPLIT_STORAGE_VALIDATION']
  if sanitized:flags+=['-fsanitize=undefined,bounds']
  for paced in [False,True]:
   for worker in [False,True]:
    exe=out/f'chain-{sanitized}-{paced}-{worker}'
    cmd=flags+['-DNEOCLR_NATIVE_GC']
    if paced:cmd+=['-DNEOCLR_GC_PACED']
    if worker:cmd+=['-DNEOCLR_WORKER_GC']
    run(cmd+[base/'native-gc-owned-chain-test.c',base/'root-probe.c',base/'native-gc.c','-o',exe])
    run([exe])
    for case in ['fault','foreign-fault','mutable-transient']:run([exe,case],-signal.SIGABRT)
  for gc in [False,True]:
   exe=out/f'checked-{sanitized}-{gc}'
   cmd=flags+[base/'root-validation-test.c',base/'root-probe.c']
   if gc:cmd+=['-DNEOCLR_NATIVE_GC',base/'native-gc.c']
   run(cmd+['-o',exe])
   for case in ['address','width','flags','fault','transient','plan-missing','plan-hint-missing','plan-length','plan-short','batch-address','batch-width','batch-flags','batch-transient','context-mutated','context-null','context-ancestor-null']:
    run([exe,case],-signal.SIGABRT)
   for case in ['plan-moved','batch-valid','context-valid','state-valid']:run([exe,case])
   if gc:
    for case in ['owned-entry-invalid','owned-mixed-invalid','owned-transient-invalid','owned-handoff-invalid','owned-handoff-fallback']:run([exe,case],-signal.SIGABRT)
    for case in ['owned-valid','owned-handoff-valid']:run([exe,case])
 for path in [Path(__file__),base/'native-gc-owned-chain-test.c',base/'root-validation-test.c',* (root/'runtime/native/gc').glob('*.[ch]')]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
print('Owned-chain and checked-root configurations passed')
