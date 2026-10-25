import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(exist_ok=False)
r={'passed':False,'commands':[],'inputs':{}}
try:
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 for sanitizer in [False,True]:
  exe=out/f'boundary-{sanitizer}'
  cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC','-DNEOCLR_WORKER_GC','-I',root/'runtime/native/gc',root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',base/'native-test.c','-o',exe]
  if sanitizer:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  for command in [cmd,[exe]]:
   command=list(map(str,command));x=subprocess.run(command,capture_output=True,text=True,timeout=60)
   r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for path in [Path(__file__),base/'native-test.c',*(root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
