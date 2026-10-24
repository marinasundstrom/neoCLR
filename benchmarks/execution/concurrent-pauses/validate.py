import argparse, json, subprocess, hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 candidate=out/'candidate.c';run(['python3',base.parent/'concurrent-integration/generate.py','--output',candidate])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 for name,source in [('sync',root/'runtime/native/gc/native-gc.c'),('candidate',candidate)]:
  instrumented=out/f'{name}-instrumented.c';run(['python3',base/'instrument.py','--source',source,'--output',instrumented])
  for san in [False,True]:
   exe=out/f'{name}-{san}';cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_GC_PACED','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc',base/'test.c',root/'runtime/native/gc/root-probe.c',instrumented,'-o',exe]
   if name=='candidate':cmd+=['-DNEOCLR_PAUSE_TEST_DELAY']
   if san:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd);run([exe])
 r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [*out.glob('*.c'),*base.glob('*.py'),base/'test.c']};r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
