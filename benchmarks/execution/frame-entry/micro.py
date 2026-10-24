import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--candidate-source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
base=out/'baseline.c';base.write_bytes(subprocess.check_output(['git','show','4d5fa423c:runtime/native/gc/root-probe.c']))
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
r=dict(passed=False,commands=[],inputs={})
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 for name,source in [('baseline',base),('candidate',a.candidate_source.resolve())]:
  for sanitized in [False,True]:
   exe=out/(name+str(sanitized));cmd=cc+['-O2','-std=c11','-DNEOCLR_NATIVE_GC','-I',root/'runtime/native/gc','-I',root/'runtime/native/memory',Path(__file__).with_name('entry.c'),source,root/'runtime/native/gc/native-gc.c','-o',exe]
   if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd)
 for sanitized in [False,True]:
  for name in ['baseline','candidate','candidate','baseline']:run([out/(name+str(sanitized))])
 for p in [Path(__file__),Path(__file__).with_name('entry.c'),base,a.candidate_source.resolve(),*(root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
