"""Execute emitted-store AOT/JIT consumers against the live native collector."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent;root=base.parents[2]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd,env=None):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90,env=env)
 r['commands'].append(dict(command=cmd,environment=env and {k:v for k,v in env.items() if k.startswith('NEOCLR_MUTATION_')},exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
 x.check_returncode()
try:
 runner=a.runner.resolve();obj=out/'stores.o'
 run([runner,'mutation_store::tests::emits_aot_hook_import_for_both_targets','--exact'],dict(os.environ,NEOCLR_MUTATION_TEST_OBJECT=str(obj)))
 assert obj.exists(), 'runner filter must execute the emission test'
 managed=out/'managed.o'
 run([runner,'mutation_store::tests::ordinary_value_pipeline_lowers_only_when_opted_in','--exact'],dict(os.environ,NEOCLR_MUTATION_MANAGED_OBJECT=str(managed)))
 assert managed.exists()
 source=out/'native-gc.c'
 run(['python3',base.parent/'concurrent-live/generate.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 cc=recipe[:recipe.index('-std=c11')]
 common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base/'consumer.c',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',source]
 for sanitized in [False,True]:
  exe=out/f'aot-{sanitized}'
  cmd=common+[obj,'-DCONSUMER_MAIN','-o',exe]
  if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  run(cmd);run([exe])
 for name,value in [('missing',None),('legacy',0),('partial',1),('noboundary',7)]:
  stub=out/(name+'.c')
  query='int inert(void) { return 0; }' if value is None else 'uint64_t experiment_gc_live_capabilities_v1(uint32_t version) { (void)version; return '+str(value)+'; }'
  stub.write_text('#include <stdint.h>\n'+query+'\n')
  run(cc+['-std=c11','-Wall','-Wextra','-Werror','-dynamiclib',stub,'-o',out/(name+'.dylib')])
 run([runner,'mutation_store::tests::incompatible_live_hosts_rejected_before_binding','--exact','--ignored','--nocapture'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST_CASES=str(out)))
 assert '1 passed' in r['commands'][-1]['stdout']
 for paced in [False,True]:
  for sanitized in [False,True]:
   exe=out/f'managed-aot-{paced}-{sanitized}'
   cmd=common+[managed,'-DMANAGED_MAIN','-o',exe]
   if paced:cmd+=['-DNEOCLR_GC_PACED']
   if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd);run([exe])
 host=out/'host.dylib'
 run(common+['-dynamiclib','-o',host])
 run([runner,'mutation_store::tests::native_collector_jit_consumer','--exact','--ignored','--nocapture'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(host)))
 assert '1 passed' in r['commands'][-1]['stdout']
 run([runner,'mutation_store::tests::managed_pipeline_runs_on_live_collector','--exact','--ignored','--nocapture'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(host)))
 assert '1 passed' in r['commands'][-1]['stdout']
 paced_host=out/'paced-host.dylib'
 run(common+['-DNEOCLR_GC_PACED','-dynamiclib','-o',paced_host])
 run([runner,'mutation_store::tests::managed_pipeline_runs_on_live_collector','--exact','--ignored','--nocapture'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(paced_host)))
 assert '1 passed' in r['commands'][-1]['stdout']
 r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [runner,obj,managed,source,base/'consumer.c',Path(__file__),base/'managed.neoil',base.parent/'concurrent-live/contract.h',root/'runtime/codegen/src/values.rs',root/'runtime/codegen/src/fault_details.rs',root/'runtime/codegen/src/mutation_admission.rs',root/'runtime/codegen/src/frame_cleanup.rs',root/'runtime/codegen/src/gc_probe.rs',root/'runtime/codegen/src/mutation_store.rs',root/'runtime/codegen/src/mutation_store_tests.rs',base.parent/'concurrent-mutation-gate/gate.c',base.parent/'concurrent-mutation-gate/gate.h',root/'runtime/native/gc/native-gc.h',root/'runtime/native/gc/root-probe.c',root/'runtime/native/gc/root-probe.h']}
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
