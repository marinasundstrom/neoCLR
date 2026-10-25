"""Differential old-identity validation over fragmented layouts and negative controls."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import signal
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(command,negative=False):
 x=subprocess.run(list(map(str,command)),capture_output=True,text=True,timeout=60)
 r['commands'].append(dict(command=list(map(str,command)),exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
 if negative:assert x.returncode==-signal.SIGABRT
 else:x.check_returncode()
try:
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror']
 for sanitized in [False,True]:
  exe=out/str(sanitized);flags=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all'] if sanitized else []
  run(cc+flags+[base/'seed-linear-test.c','-o',exe]);run([exe])
 header=(base.parents[2]/'runtime/native/gc/concurrent/seed-linear.h').read_text();assert header.count('now->kind!=old->kind')==1
 (out/'seed-linear.h').write_text(header.replace('now->kind!=old->kind','0'))
 (out/'negative.c').write_text((base/'seed-linear-test.c').read_text())
 run(cc+[out/'negative.c','-o',out/'negative']);run([out/'negative'],True)
 r['inputs']={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in [Path(__file__),base/'seed-linear.h',base/'seed-linear-test.c']};r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
