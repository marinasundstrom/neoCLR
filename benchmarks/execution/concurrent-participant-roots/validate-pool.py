"""Rebuild and compare copied-root and direct-root compiled worker pools."""
import argparse,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
prior=json.loads((root/'docs/experiments/native-worker-pool/compiled-parallel-validation.json').read_text());r={'passed':False,'commands':[],'inputs':{}}
def run(cmd):
 x=subprocess.run(cmd,capture_output=True,text=True,timeout=120);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for value in cmd:
  p=Path(value)
  if p.is_file():r['inputs'][str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
try:
 compile_command=next(c['command'].copy() for c in prior['commands'] if '--native-worker-safepoints' in c['command'])
 old_object=next(x for x in compile_command if x.endswith('.o'));compile_command[compile_command.index(old_object)]=str(out/'code.o');run(compile_command)
 for old in prior['commands']:
  if '-o' not in old['command']:continue
  for snapshot in [False,True]:
   cmd=[str(out/'code.o') if x==old_object else x for x in old['command']]
   name=Path(cmd[cmd.index('-o')+1]).name+('-snapshot' if snapshot else '-control');exe=out/name;cmd[cmd.index('-o')+1]=str(exe)
   if snapshot:cmd+=['-DNEOCLR_WORKER_ROOT_SNAPSHOT',str(root/'runtime/native/gc/participant-roots.c')]
   run(cmd);run([str(exe)])
 for path in [Path(__file__),root/'runtime/native/workers/compiled-workers.inc',* (root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
