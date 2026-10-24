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
 runner=a.runner.resolve();obj=out/'statics.o'
 run([runner,'mutation_store::tests::managed_statics_emit_guarded_publication','--exact'],dict(os.environ,NEOCLR_MUTATION_MANAGED_OBJECT=str(obj)))
 assert obj.exists()
 source=out/'native-gc.c'
 run(['python3',base.parent/'concurrent-live/automatic.py','--output',source])
 provider=out/'text-arena.c'
 run(['python3',base/'native-providers.py','--output',provider])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 cc=recipe[:recipe.index('-std=c11')]
 common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',root/'runtime/native/memory','-I',base.parent/'concurrent-mutation-gate',base/'statics-consumer.c',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',provider,source]
 for paced in [False,True]:
  flags=['-DNEOCLR_GC_PACED'] if paced else []
  for sanitized in [False,True]:
   exe=out/f'aot-{paced}-{sanitized}'
   cmd=common+flags+[obj,'-DMANAGED_MAIN','-o',exe]
   if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd);run([exe])
  host=out/f'host-{paced}.dylib'
  run(common+flags+['-dynamiclib','-o',host])
  run([runner,'mutation_store::tests::managed_statics_run_on_live_collector','--exact','--ignored','--nocapture'],dict(os.environ,NEOCLR_MUTATION_TEST_HOST=str(host)))
  assert '1 passed' in r['commands'][-1]['stdout']
 r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [runner,obj,source,provider,base/'native-providers.py',root/'runtime/codegen/src/values.rs',base/'statics-consumer.c',Path(__file__),base/'statics.neoil',base.parent/'concurrent-live/automatic.py',root/'runtime/codegen/src/mutation_admission.rs',root/'runtime/codegen/src/mutation_store_tests.rs',root/'runtime/native/memory/text-arena.c',root/'runtime/native/gc/root-probe.c']}
 legacy=[str(root/'runtime/native/memory/text-arena.c') if str(v)==str(provider) else str(v) for v in common]+[str(obj),'-DMANAGED_MAIN','-o',str(out/'legacy')]
 x=subprocess.run(legacy,capture_output=True,text=True,timeout=90)
 r['legacyProvider']=dict(command=legacy,exitCode=x.returncode,stderr=x.stderr)
 assert x.returncode!=0 and '_experiment_gc_static_storage_rooted_v1' in x.stderr
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
