#!/usr/bin/env python3
"""Differential scalar/batch tracing including header/padding/interior candidates."""
import argparse,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--candidate-source',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],inputs={})
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x.stdout
try:
 baseline=out/'baseline-gc.c';baseline.write_text(run(['git','show','82fad1e00:runtime/native/gc/native-gc.c']))
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 base=root/'docs/experiments/aot-console'
 for sanitized in [False,True]:
  for profile in [False,True]:
   outputs=[]
   for label,gc in [('baseline',baseline),('candidate',a.candidate_source.resolve())]:
    exe=out/f'{label}-{sanitized}-{profile}'
    cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC','-I',root/'runtime/native/gc',base/'native-gc-scan-test.c',base/'root-probe.c',gc,'-o',exe]
    if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
    if profile:cmd+=['-DNEOCLR_GC_PROFILE']
    run(cmd);outputs.append(run([exe]))
   assert outputs[0]==outputs[1],outputs
 for path in [Path(__file__),a.candidate_source.resolve(),base/'native-gc-scan-test.c',*(root/'runtime/native').rglob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
