"""Qualify copied participant roots against the actual synchronous native collector."""
import argparse,hashlib,json,subprocess,signal
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
 source=root/'runtime/native/gc/participant-roots.c'
 text=source.read_text();needle='if (walk(owner,parts,count,buffer->values,&used)) abort();'
 assert text.count(needle)==1
 broken=out/'stale-roots.c';broken.write_text(text.replace(needle,'if (0) abort();'))
 negative=[str(broken) if x==str(source) else x for x in cmd if not x.startswith('-fsanitize=') and x!='-fno-sanitize-recover=all']
 negative[negative.index('-o')+1]=str(out/'stale-roots')
 x=subprocess.run(negative,capture_output=True,text=True,timeout=60)
 r['commands'].append(dict(command=negative,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 x=subprocess.run([str(out/'stale-roots')],capture_output=True,text=True,timeout=60)
 r['negativeStaleRoots']=dict(exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr)
 assert x.returncode==-signal.SIGABRT
 for path in [*base.glob('*.[ch]'),Path(__file__),* (root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
