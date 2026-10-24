"""Qualify accepted entry cancellation before generated arena reuse."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent;root=base.parents[2]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd,expected=True):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
 if expected:x.check_returncode()
 else:assert x.returncode!=0 and 'entry boundary line' in x.stderr
try:
 source=out/'native-gc.c'
 run(['python3',base/'automatic.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 cc=recipe[:recipe.index('-std=c11')]
 common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base/'entry-boundary.c',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c']
 for paced in [False,True]:
  for sanitized in [False,True]:
   exe=out/f'entry-{paced}-{sanitized}'
   flags=['-DNEOCLR_GC_PACED'] if paced else []
   if sanitized:flags+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(common+flags+[source,'-o',exe]);run([exe])
 negative=out/'negative.c'
 s=source.read_text();needle='    if (candidate_active) candidate_cancel(); /* admitted entry resets the arena */'
 assert s.count(needle)==1
 negative.write_text(s.replace(needle,''))
 exe=out/'negative'
 run(common+[negative,'-o',exe]);run([exe],False)
 r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source,base/'automatic.py',base/'entry-boundary.c',Path(__file__)]}
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
