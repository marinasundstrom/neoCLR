"""Qualify the opt-in owned allocation index against an independent address model."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--exact-allocation-index',action='store_true')
p.add_argument('--address-sanitizer',action='store_true',help='Also run ASan where the host sanitizer runtime can initialize.')
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent;root=base.parents[2]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r={'passed':False,'exactAllocationIndex':a.exact_allocation_index,'addressSanitizer':a.address_sanitizer,'commands':[]}
def run(command,negative=False):
 command=list(map(str,command))
 try:x=subprocess.run(command,capture_output=True,text=True,timeout=90)
 except subprocess.TimeoutExpired as error:
  r['commands'].append(dict(command=command,timedOut=True,timeoutSeconds=90))
  raise
 r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
 if negative: assert x.returncode!=0 and 'index line' in x.stderr
 else:x.check_returncode()
try:
 source=out/'native-gc.c';run(['python3',base/'automatic.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 cc=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-DNEOCLR_LIVE_ALLOCATION_INDEX','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c']
 if a.exact_allocation_index: cc += ['-DNEOCLR_LIVE_EXACT_INDEX']
 for sanitize in ['none','undefined']+(['address'] if a.address_sanitizer else []):
  exe=out/f'model-{sanitize}'
  flags=[] if sanitize=='none' else [('-fsanitize=address,undefined' if sanitize=='address' else '-fsanitize=undefined,bounds'),'-fno-sanitize-recover=all']
  run(cc+flags+[base/'allocation-index-test.c',source,'-o',exe]);run([exe])
 for label,needle in [('reclaim','                owned_index_clear((unsigned char *)(b+1)-a->data);'),('reset','    owned_index_reset();')]:
  text=source.read_text();assert text.count(needle)==1
  broken=out/f'{label}.c';broken.write_text(text.replace(needle,'    /* deliberately missing revocation */'))
  exe=out/label
  run(cc+['-Wno-unused-function',base/'allocation-index-test.c',broken,'-o',exe]);run([exe],True)
 r['inputs']={str(path):hashlib.sha256(path.read_bytes()).hexdigest() for path in [source,base/'allocation-index.h',base/'allocation-index-test.c',base/'generate.py',base/'automatic.py',base/'store.h',Path(__file__)]}
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
