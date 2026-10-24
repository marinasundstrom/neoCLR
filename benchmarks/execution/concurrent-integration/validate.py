import argparse, hashlib, json, subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 source=out/'native-gc.c';run(['python3',base/'generate.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 for san in [False,True]:
  for name in ['lifetime','native-gc-test','native-gc-seed-test']:
   test=base/'lifetime.c' if name=='lifetime' else root/'docs/experiments/aot-console'/f'{name}.c'
   exe=out/f'{name}-{san}'
   cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc',test,root/'runtime/native/gc/root-probe.c',source,'-o',exe]
   if name=='lifetime':cmd+=['-DNEOCLR_GC_PACED']
   if san:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd);run([exe])
 r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source,*base.glob('*.[ch]'),base/'generate.py',base/'validate.py']};r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
