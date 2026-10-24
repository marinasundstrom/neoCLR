import argparse,hashlib,json,signal,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
r=dict(passed=False,commands=[],inputs={})
def run(cmd,expected=0):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=cmd,exitCode=x.returncode,expected=expected,stdout=x.stdout,stderr=x.stderr));assert x.returncode==expected,r['commands'][-1]
try:
 for san in [False,True]:
  for paced in [False,True]:
   exe=out/f'lifetime-{san}-{paced}'
   cmd=cc+['-O2','-std=c11','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_TEST_THREADS','-I',root/'runtime/native/gc','-I',root/'runtime/native/memory',root/'docs/experiments/aot-console/native-gc-state-test.c',a.source.resolve(),root/'runtime/native/gc/native-gc.c','-o',exe]
   if san:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   if paced:cmd+=['-DNEOCLR_GC_PACED']
   run(cmd);run([exe])
   for case in ['null','map','parked','head','phase','fault','leave']:run([exe,case],-signal.SIGABRT)
 for path in [Path(__file__),root/'docs/experiments/aot-console/native-gc-state-test.c',a.source.resolve(),*(root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
