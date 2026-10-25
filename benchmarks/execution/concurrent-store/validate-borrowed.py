"""Qualify explicit borrowed-store lowering against the native collector, not full admission."""
import argparse,hashlib,json,os,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--runner',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);base=Path(__file__).resolve().parent;root=base.parents[2];runner=a.runner.resolve()
r=dict(passed=False,commands=[])
def run(cmd,env=None):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120,env=env)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,environment={k:v for k,v in (env or {}).items() if k.startswith('NEOCLR_')}));x.check_returncode();return x
try:
 obj=out/'borrowed.o'
 run([runner,'mutation_store::tests::borrowed_store_object_imports_and_frame_cleanup','--exact'],dict(os.environ,NEOCLR_BORROWED_STORE_OBJECT=str(obj)))
 assert obj.exists()
 source=out/'native-gc.c';run(['python3',base.parent/'concurrent-live/generate.py','--output',source])
 main=out/'main.c';main.write_text('#include "native-gc.h"\nextern int32_t framed(void *,void *,uint64_t);\nextern int native_store_consumer(int32_t (*)(void *,void *,uint64_t));\nint main(void) { return native_store_consumer(framed); }\n')
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 common=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-DNEOCLR_LIVE_ALLOCATION_INDEX','-DNEOCLR_LIVE_EXACT_INDEX','-DNEOCLR_LIVE_INLINE_BASE','-DNEOCLR_LIVE_STORE_HANDLE','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base/'consumer.c',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',source]
 for sanitizer in [False,True]:
  exe=out/f'aot-{sanitizer}';cmd=common+[obj,main,'-o',exe]
  if sanitizer:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  run(cmd);run([exe])
 host=out/'host.dylib';run(common+['-dynamiclib','-o',host])
 x=run([runner,'mutation_store::tests::borrowed_store_native_jit_consumer','--exact','--ignored'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(host)))
 assert '1 passed' in x.stdout
 stub=out/'legacy.c';stub.write_text('#include <stdint.h>\nuint64_t experiment_gc_live_capabilities_v1(uint32_t v) { return v==1?31:0; }\n')
 legacy=out/'legacy.dylib';run(recipe[:recipe.index('-std=c11')]+['-dynamiclib',stub,'-o',legacy])
 cmd=[str(runner),'mutation_store::tests::borrowed_store_native_jit_consumer','--exact','--ignored','--nocapture']
 x=subprocess.run(cmd,capture_output=True,text=True,timeout=120,env=dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(legacy)))
 r['negativeHost']=dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr)
 assert x.returncode and 'host lacks boundary completion or same-thread borrowed stores' in x.stdout+x.stderr
 r['inputs']={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in [runner,obj,source,base/'consumer.c',Path(__file__),base.parent/'concurrent-live/contract.h',root/'runtime/codegen/src/mutation_store.rs',root/'runtime/codegen/src/mutation_store_tests.rs',root/'runtime/codegen/src/mutation_admission.rs']};r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
