"""Add matched JIT hosts to a qualified stack-bound TLS HTTP build."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3]
base=a.build.resolve();source=base/'build.json';built=json.loads(source.read_text());assert built['passed']
for path,digest in built['inputs'].items():
 assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build',path)
# The original paired build retains its base manifest, which owns selection commands.
original=next(Path(path) for path in built['inputs'] if path.endswith('/build.json'))
prior=json.loads(original.read_text())
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,scope=__doc__,recipes={},commands=[],inputs=dict(built['inputs']))
try:
 for mode in ['sync','live']:
  for layout in built.get('layouts',['split','grouped']):
   name=layout+'-'+mode
   link=next(c['command'].copy() for c in built['commands'] if c['command'][c['command'].index('-o')+1]==str(base/name))
   link=[v for v in link if not v.endswith('/main.c') and not v.endswith('/'+mode+'.o')]+['-dynamiclib']
   link[link.index('-o')+1]=str(out/(name+'.dylib'))
   x=subprocess.run(link,capture_output=True,text=True,timeout=180)
   r['commands'].append(dict(command=link,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
   old=prior['recipes'][mode+'-jit'];selection=json.loads(Path(old['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
   host=str(original.parent/(mode+'.dylib'));assert host in selection
   selection=[str(out/(name+'.dylib')) if v==host else v for v in selection]
   selected=out/(name+'-selection.json');selected.write_text(json.dumps(selection))
   command=old['command'].copy();command[0]=str(a.runner.resolve())
   env=dict(old['env'],NEOCLR_LIVE_SELECTION_COMMAND=str(selected),NEOCLR_TEST_JIT_ARENA='fallback')
   r['recipes'][name+'-jit']=dict(old,command=command,env=env)
   for path in [selected,out/(name+'.dylib')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [source,a.runner.resolve(),Path(__file__).resolve(),*[(root/'runtime/codegen/src'/n) for n in ['lib.rs','jit.rs','jit_memory.rs','jit_memory_fallback.rs']]]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')
