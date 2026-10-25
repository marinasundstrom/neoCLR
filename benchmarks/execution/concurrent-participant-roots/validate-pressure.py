import argparse,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--read-epochs',action='store_true');p.add_argument('--reuse-roots',action='store_true');a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(exist_ok=False)
r={'passed':False,'commands':[],'inputs':{}}
try:
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 if a.read_epochs:
  command=['python3',str(root/'benchmarks/execution/concurrent-live/generate.py'),'--output',str(out/'native-gc.c')]
  x=subprocess.run(command,capture_output=True,text=True,timeout=60)
  r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for sanitized in [False,True]:
  exe=out/f'pressure-{sanitized}'
  cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_WORKER_GC','-DNEOCLR_GC_ALLOCATION_DISPATCH','-DNEOCLR_WORKER_ALLOCATION_OWNER','-DNEOCLR_WORKER_COLLECTION_OWNER','-DNEOCLR_WORKER_ROOT_SNAPSHOT','-DNEOCLR_WORKER_PACED','-I',root/'runtime/native/gc','-I',root/'runtime/native/workers',root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/gc/participant-roots.c',root/'runtime/native/workers/native-workers.c',base/'pressure-test.c','-o',exe]
  if a.reuse_roots:cmd+=['-DNEOCLR_WORKER_ROOT_REUSE']
  if a.read_epochs:
   cmd=[out/'native-gc.c' if str(x)==str(root/'runtime/native/gc/native-gc.c') else x for x in cmd]
   cmd+=['-DNEOCLR_WORKER_READ_EPOCHS','-DNEOCLR_GC_PROFILE','-I',root/'benchmarks/execution/concurrent-mutation-gate',root/'benchmarks/execution/concurrent-mutation-gate/gate.c']
  if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  for command in [cmd,[exe]]:
   command=list(map(str,command));x=subprocess.run(command,capture_output=True,text=True,timeout=60)
   r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for path in [Path(__file__),base/'pressure-test.c',*(root/'runtime/native/gc').glob('*.[ch]'),*(root/'runtime/native/workers').glob('*')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
