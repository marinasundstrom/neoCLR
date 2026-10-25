"""Acquisition-inclusive scalar-store CPU by stores per activation; no HTTP claim."""
import argparse,json,hashlib,subprocess,statistics
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);base=Path(__file__).resolve().parent;root=base.parents[2]
r=dict(passed=False,scope=__doc__,commands=[],trials=[])
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 source=out/'native-gc.c';run(['python3',base/'generate.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 for mode in ['checked','borrowed']:
  cmd=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_SHADOW_SPARSE_INDEX','-DNEOCLR_LIVE_ALLOCATION_INDEX','-DNEOCLR_LIVE_EXACT_INDEX','-DNEOCLR_LIVE_INLINE_BASE','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base/'store-acquisition-benchmark.c',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',source,'-o',out/mode]
  if mode=='borrowed':cmd+=['-DNEOCLR_LIVE_STORE_HANDLE']
  run(cmd)
 for pair in range(6):
  for mode in (['checked','borrowed'] if pair%2==0 else ['borrowed','checked']):
   lines=run([out/mode]).stdout.splitlines();assert len(lines)==14
   rows=[list(map(int,line.split())) for line in lines];assert [(row[0],row[1]) for row in rows]==[(m,b) for m in range(2) for b in [1,2,4,8,16,64,1024]]
   r['trials'].append(dict(pair=pair,mode=mode,cpuNs=[row[2] for row in rows]))
 r['pairedCpuRatios']=[statistics.median(next(t['cpuNs'][i] for t in r['trials'] if t['pair']==pair and t['mode']=='borrowed')/next(t['cpuNs'][i] for t in r['trials'] if t['pair']==pair and t['mode']=='checked') for pair in range(6)) for i in range(14)]
 r['cells']=[dict(destination=m,storesPerActivation=b,storesPerTrial=1048576) for m in ['outside','inactive-heap'] for b in [1,2,4,8,16,64,1024]]
 r['inputs']={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in [source,base/'store.h',base/'store-acquisition-benchmark.c',Path(__file__)]};r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
