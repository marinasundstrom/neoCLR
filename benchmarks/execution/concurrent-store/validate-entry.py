"""Execute emitted-store AOT/JIT consumers against the live native collector."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--exact-allocation-index',action='store_true')
p.add_argument('--allocation-index',action='store_true')
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();assert not a.exact_allocation_index or a.allocation_index
base=Path(__file__).resolve().parent;root=base.parents[2]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,allocationIndex=a.allocation_index,exactAllocationIndex=a.exact_allocation_index,commands=[])
def run(cmd,env=None):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90,env=env)
 r['commands'].append(dict(command=cmd,environment=env and {k:v for k,v in env.items() if k.startswith('NEOCLR_MUTATION_')},exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
 x.check_returncode()
r['optimizationLevel']=os.environ.get('NEOCLR_TEST_OPT_LEVEL','none')
try:
 runner=a.runner.resolve();obj=out/'entry.o'
 run([runner,'mutation_store::tests::managed_public_entry_emits_guarded_transition','--exact'],dict(os.environ,NEOCLR_MUTATION_MANAGED_OBJECT=str(obj)))
 assert obj.exists()
 consumer=base/'entry-consumer.c'
 source=out/'native-gc.c'
 run(['python3',base.parent/'concurrent-live/automatic.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 cc=recipe[:recipe.index('-std=c11')]
 if a.exact_allocation_index: cc += ['-DNEOCLR_LIVE_EXACT_INDEX']
 if a.allocation_index:cc+=['-DNEOCLR_LIVE_ALLOCATION_INDEX']
 common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',consumer,base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/memory/text-arena.c',source]
 for paced in [False,True]:
  flags=['-DNEOCLR_GC_PACED'] if paced else []
  for sanitized in [False,True]:
   exe=out/f'aot-{paced}-{sanitized}'
   cmd=common+flags+[obj,'-DMANAGED_MAIN','-o',exe]
   if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd);run([exe])
  host=out/f'host-{paced}.dylib'
  run(common+flags+['-dynamiclib','-o',host])
  run([runner,'mutation_store::tests::managed_public_entry_runs_on_live_collector','--exact','--ignored','--nocapture'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(host)))
  assert '1 passed' in r['commands'][-1]['stdout']
 r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [runner,obj,source,consumer,Path(__file__),root/'runtime/codegen/src/values.rs',root/'runtime/codegen/src/lib.rs',root/'runtime/codegen/src/jit.rs',base/'entry.neoil',base.parent/'concurrent-live/automatic.py',root/'runtime/codegen/src/mutation_admission.rs',root/'runtime/codegen/src/mutation_store_tests.rs',root/'runtime/native/memory/text-arena.c',root/'runtime/native/gc/root-probe.c']}
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
