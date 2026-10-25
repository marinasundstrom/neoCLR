"""Relink identical HTTP objects with split/grouped native stack-bound TLS."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--baseline-source',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3]
base=a.build.resolve();source=root/'docs/experiments/aot-console/native-stack.c'
baseline=a.baseline_source.resolve();original=base/'build.json'
built=json.loads(original.read_text());assert built['passed']
for path,digest in built['inputs'].items():
 checked=baseline if Path(path)==source else Path(path)
 assert hashlib.sha256(checked.read_bytes()).hexdigest()==digest,('stale build',path)
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,scope=__doc__,recipes={},commands=[],inputs=dict(built['inputs']))
r['inputs'].pop(str(source),None)
def run(command):
 x=subprocess.run(command,capture_output=True,text=True,timeout=180)
 r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 for mode in ['sync','live']:
  old=next(c['command'] for c in built['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1]==str(base/mode))
  assert str(source) in old
  for grouped in [False,True]:
   name=('grouped-' if grouped else 'split-')+mode
   command=[str(source if grouped else baseline) if v==str(source) else v for v in old]
   command+=['-I',str(source.parent)]
   if grouped:command+=['-DNEOCLR_GROUP_STACK_BOUNDS']
   command[command.index('-o')+1]=str(out/name);run(command)
   r['recipes'][name+'-aot']=dict(execution='aot',collector=mode,command=[str(out/name)],env={})
   r['inputs'][str(out/name)]=hashlib.sha256((out/name).read_bytes()).hexdigest()
 for path in [source,baseline,original,Path(__file__).resolve()]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')
