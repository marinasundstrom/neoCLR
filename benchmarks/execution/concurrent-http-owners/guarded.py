"""Relink an existing plaintext cohort with a no-marking guarded control."""
import argparse,hashlib,json,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,sample='plain',commands=[],inputs={},control=str(base))
try:
 original=json.loads((base/'results.json').read_text());assert original['passed'] and original['sample']=='plain'
 code=(base/'native-gc.c').read_text()
 start='candidate_autostart = reason == NEOCLR_GC_TRIGGER_PUBLICATION || reason == NEOCLR_GC_TRIGGER_ALLOCATION;'
 idle='if (!status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'
 assert code.count(start)==code.count(idle)==1
 source=out/'guarded-gc.c';source.write_text(code.replace(start,'candidate_autostart = 0; /* Synchronous control. */').replace(idle,idle.replace('if (','if (0 && ',1)))
 for workers in [1,4]:
  for mode in ['sync','live']:
   shutil.copy2(base/f'{mode}-{workers}',out/f'{mode}-{workers}')
  command=next(c['command'].copy() for c in original['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name==f'live-{workers}')
  command=[str(source) if x==str(base/'native-gc.c') else x for x in command];command[command.index('-o')+1]=str(out/f'guarded-{workers}')
  x=subprocess.run(command,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
  for item in command:
   path=Path(item)
   if path.is_file():r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),base/'results.json',base/'native-gc.c',*out.glob('*-1'),*out.glob('*-4')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
