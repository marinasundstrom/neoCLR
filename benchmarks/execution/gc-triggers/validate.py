#!/usr/bin/env python3
"""Trigger classification, diagnostic reset and per-thread/delegation checks."""
import argparse,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],inputs={})
try:
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')];base=root/'docs/experiments/aot-console'
 for sanitized in [False,True]:
  for profile in [False,True]:
   for worker in [False,True]:
    exe=out/f'triggers-{sanitized}-{profile}-{worker}'
    cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PACED','-DNEOCLR_TEST_THREADS',str(base/'native-gc-trigger-test.c'),str(base/'root-probe.c'),str(base/'native-gc.c'),'-o',str(exe)]
    if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
    if profile:cmd+=['-DNEOCLR_GC_PROFILE']
    if worker:cmd+=['-DNEOCLR_WORKER_GC']
    for command in [cmd,[str(exe)]]:
     x=subprocess.run(command,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for path in [Path(__file__),base/'native-gc-trigger-test.c',*(root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
