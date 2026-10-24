#!/usr/bin/env python3
"""Check cache eviction, reclamation and forged addresses with both sizes."""
import argparse, hashlib, json, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--candidate',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
cc=recipe[:recipe.index('-std=c11')];base=root/'docs/experiments/aot-console'
baseline=out/'baseline.c'
baseline.write_bytes(subprocess.check_output(['git','show','cb6bccb27:runtime/native/gc/native-gc.c']))
r=dict(passed=False,baselineRevision='cb6bccb27',commands=[],inputs={})
try:
 for label,source in [('baseline',baseline),('candidate',a.candidate.resolve())]:
  for sanitized in [False,True]:
   for profile in [False,True]:
    exe=out/f'{label}-{sanitized}-{profile}'
    cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC','-I',str(root/'runtime/native/gc'),'-I',str(root/'runtime/native/memory'),base/'native-gc-owned-test.c',base/'root-probe.c',source,'-o',exe]
    if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
    if profile:cmd+=['-DNEOCLR_GC_PROFILE']
    for command in [cmd,[exe]]:
     command=list(map(str,command));x=subprocess.run(command,capture_output=True,text=True,timeout=60)
     r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for path in [Path(__file__),baseline,a.candidate,base/'native-gc-owned-test.c',*(root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
