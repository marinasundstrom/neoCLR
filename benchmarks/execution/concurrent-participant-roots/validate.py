"""Qualify copied participant roots against the actual synchronous native collector."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],inputs={})
try:
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 for sanitized in [False,True]:
  exe=out/f'test-{sanitized}'
  cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_WORKER_GC','-I',str(root/'runtime/native/gc'),str(root/'runtime/native/gc/participant-roots.c'),str(base/'test.c'),str(root/'runtime/native/gc/root-probe.c'),str(root/'runtime/native/gc/native-gc.c'),'-o',str(exe)]
  if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  for command in [cmd,[str(exe)]]:
   x=subprocess.run(command,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for path in [*base.glob('*.[ch]'),Path(__file__),* (root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
