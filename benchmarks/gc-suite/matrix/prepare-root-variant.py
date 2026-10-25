"""Relink matched HTTP objects with the experimental split storage validator."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
base=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
original=base/'build.json';built=json.loads(original.read_text())
assert built['passed']
for path,digest in built['inputs'].items():
 assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build',path)
r=dict(passed=False,scope=__doc__,experiment='NEOCLR_GC_SPLIT_STORAGE_VALIDATION',recipes=dict(built['recipes']),commands=[],inputs=dict(built['inputs']))
def run(command):
 x=subprocess.run(command,capture_output=True,text=True,timeout=180)
 r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 for mode in ['sync','live']:
  link=next(c['command'].copy() for c in built['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1]==str(base/mode))
  assert not any('STORAGE_VALIDATION' in v for v in link)
  link+=['-DNEOCLR_GC_SPLIT_STORAGE_VALIDATION']
  link[link.index('-o')+1]=str(out/mode);run(link)
  r['recipes']['split-'+mode+'-aot']=dict(execution='aot',collector=mode,command=[str(out/mode)],env={})
  shared=[v for v in link if not v.endswith('/main.c') and v!=str(base/(mode+'.o'))]+['-dynamiclib']
  shared[shared.index('-o')+1]=str(out/(mode+'.dylib'));run(shared)
  old=built['recipes'][mode+'-jit'];selection=json.loads(Path(old['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
  assert str(base/(mode+'.dylib')) in selection
  selection=[str(out/(mode+'.dylib')) if v==str(base/(mode+'.dylib')) else v for v in selection]
  selected=out/(mode+'-selection.json');selected.write_text(json.dumps(selection))
  r['recipes']['split-'+mode+'-jit']=dict(old,env=dict(old['env'],NEOCLR_LIVE_SELECTION_COMMAND=str(selected)))
  for path in [out/mode,out/(mode+'.dylib'),selected]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [original,Path(__file__).resolve()]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')
