#!/usr/bin/env python3
"""Focused POSIX readiness and async owner/idle contracts with UBSan/bounds."""
import argparse, hashlib, json, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[4];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
base=root/'docs/experiments/aot-console';carrier=root/'docs/experiments/runtime-async'
report=dict(passed=False,commands=[],inputs={})
cc=['clang','-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations','-DNEOCLR_NATIVE_GC','-fsanitize=undefined,bounds','-I',str(base),'-I',str(carrier)]
def run(name,command):
 command=list(map(str,command));r=subprocess.run(command,capture_output=True,text=True,timeout=60)
 report['commands'].append(dict(name=name,command=command,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
 r.check_returncode()
try:
 for name in ['socket-wait-test','native-idle-drain-test','native-owned-body-test']:
  sources=[base/'native-gc.c',base/'root-probe.c']
  if name=='socket-wait-test': sources += [base/(name+'.c'),base/'text-arena.c']
  else:sources += [carrier/(name+'.c'),carrier/'native-runtime.c',base/'native-stack.c']
  run(name+'-build',[*cc,*sources,'-o',out/name]);run(name,[out/name])
 for folder in [root/'runtime/native',base,carrier]:
  for path in folder.rglob('*'):
   if path.suffix in ('.c','.h'): report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 report['passed']=True
finally:(out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
