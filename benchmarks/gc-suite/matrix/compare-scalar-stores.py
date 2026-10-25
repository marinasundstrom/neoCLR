"""Interleave unchanged standalone samples with checked scalar stores off/on.
Build both variants before timing; same compiler, native sources and GC policy.
"""
import argparse,gzip,hashlib,json,os,platform,statistics,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--only',nargs='+')
p.add_argument('--rounds',type=int,default=7)
a=p.parse_args();assert a.rounds>=3
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
built=json.loads((a.build/'results.json').read_text());assert built['passed'] and built['iterations']==1
prior=json.loads(gzip.decompress((Path(__file__).parent/'aot-results.json.gz').read_bytes()))
cases=[c for c in built['cases'] if not a.only or c['sample'] in a.only]
assert cases and (not a.only or set(a.only)=={c['sample'] for c in cases})
r=dict(passed=False,scope=__doc__,system=platform.platform(),commands=[],trials=[],rows=[],inputs={},rounds=a.rounds)
def run(cmd,env=None):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=180)
 r['commands'].append(dict(command=cmd,environment=None if env is None else {k:v for k,v in env.items() if k.startswith('NEOCLR_')},exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 gc=out/'native-gc.c';provider=out/'text-arena.c'
 run(['python3',root/'benchmarks/execution/concurrent-live/native-scopes.py','--output',gc])
 run(['python3',root/'benchmarks/execution/concurrent-store/native-providers.py','--output',provider])
 for case in cases:
  name=case['sample'];n=next(n for n in case['native'] if n['mode']=='live')
  for mode in ['control','scalar']:
   print('Build',name,mode,flush=True)
   obj=out/(name+'-'+mode+'.o');selection=n['selectionCommand'].copy();selection[4]=str(obj)
   path=out/(name+'-'+mode+'.json');path.write_text(json.dumps(selection))
   env=dict(os.environ,**n['environment']);env.pop('NEOCLR_LIVE_SCALAR_ARRAY_STORES',None)
   env.pop('NEOCLR_LIVE_GENERIC_SCALAR_ARRAY_STORES',None)
   env['NEOCLR_LIVE_SELECTION_COMMAND']=str(path)
   if mode=='control':env['NEOCLR_LIVE_GENERIC_SCALAR_ARRAY_STORES']='1'
   run([a.runner.resolve(),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture'],env)
   suffix=name+'-live-budget-1024-16384-hybrid-inline'
   cmd=next(c['command'].copy() for c in prior['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1].endswith(suffix))
   old=n['selectionCommand'][4];assert old in cmd
   cmd=[str(obj) if v==old else str(gc) if v.endswith('/native-gc.c') else str(provider) if v.endswith('/text-arena.c') else v for v in cmd]
   cmd[cmd.index('-o')+1]=str(out/(name+'-'+mode))
   run(cmd)
 for iteration in range(a.rounds):
  for case in cases:
   name=case['sample']
   for mode in (['control','scalar'] if iteration%2==0 else ['scalar','control']):
    x=run([out/(name+'-'+mode)]);assert x.stdout==case['expectedPerIteration']
    values=next(line.split()[1:] for line in x.stderr.splitlines() if line.startswith('PROFILE '))
    counts=list(map(int,next(line.split()[1:] for line in x.stderr.splitlines() if line.startswith('CANDIDATE '))))
    assert len(counts)==6 and counts[0]==counts[1]+counts[2] and counts[5]==0
    row=dict(sample=name,mode=mode,round=iteration,warmup=iteration==0,cpuSeconds=float(values[0]),wallSeconds=float(values[1]),allocations=int(values[2]),allocatedBytes=int(values[3]),collections=int(values[4]),peakLiveBytes=int(values[5]),peakArenaBytes=int(values[6]),candidateCounts=counts)
    r['trials'].append(row)
 for case in cases:
  ts=[t for t in r['trials'] if t['sample']==case['sample']]
  assert len({(t['allocations'],t['allocatedBytes']) for t in ts})==1
  for mode in ['control','scalar']:
   xs=[t for t in ts if t['mode']==mode and not t['warmup']]
   r['rows'].append(dict(sample=case['sample'],mode=mode,**{k:statistics.median(t[k] for t in xs) for k in ['cpuSeconds','wallSeconds','allocations','allocatedBytes','peakArenaBytes']}))
 for c in r['commands']:
  for value in c['command']:
   path=Path(value)
   if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),a.build/'results.json',Path(__file__).parent/'aot-results.json.gz',root/'runtime/codegen/src/values.rs',root/'runtime/codegen/src/mutation_admission.rs',root/'benchmarks/execution/concurrent-live/store.h',*out.glob('*.json')]:r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
