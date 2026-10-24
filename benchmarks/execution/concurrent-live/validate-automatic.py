import argparse, gzip, hashlib, json, os, subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--runner',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd,env=None):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120,env=env)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 source=out/'native-gc.c';run(['python3',base/'automatic.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 for san in [False,True]:
  for name in ['automatic-test','native-gc-test','native-gc-seed-test']:
   test=base/'automatic-test.c' if name=='automatic-test' else root/'docs/experiments/aot-console'/f'{name}.c'
   exe=out/f'{name}-{san}'
   cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base.parent/'concurrent-mutation-gate/gate.c',test,root/'runtime/native/gc/root-probe.c',source,'-o',exe]
   if name=='automatic-test':cmd+=['-DNEOCLR_GC_PACED']
   if san:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd);run([exe])
 # Reuse the qualified managed object; only the host's epoch start changes.
 store=base.parent/'concurrent-store'
 obj=out/'managed.o';obj.write_bytes(gzip.decompress((store/'managed.o.gz').read_bytes()))
 consumer=out/'managed-consumer.c'
 text=(store/'consumer.c').read_text()
 before,after=text.rsplit('CHECK(!experiment_gc_begin(&c,&frame,NULL,0));',1)
 consumer.write_text(before+'CHECK(!neoclr_gc_idle_v1(&c,&frame));'+after)
 common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_GC_PACED','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base.parent/'concurrent-mutation-gate/gate.c',consumer,root/'runtime/native/gc/root-probe.c',source]
 for san in [False,True]:
  exe=out/f'managed-{san}'
  cmd=common+[obj,'-DMANAGED_MAIN','-o',exe]
  if san:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  run(cmd);run([exe])
 host=out/'host.dylib';run(common+['-dynamiclib','-o',host])
 run([a.runner.resolve(),'mutation_store::tests::managed_pipeline_runs_on_live_collector','--exact','--ignored','--nocapture'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(host)))
 assert '1 passed' in r['commands'][-1]['stdout']
 r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source,*base.glob('*.[ch]'),base/'automatic.py',Path(__file__),consumer,obj,a.runner.resolve(),store/'consumer.c',base/'generate.py',base.parent/'concurrent-mutation-gate/gate.c',base.parent/'concurrent-mutation-gate/gate.h']};r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
