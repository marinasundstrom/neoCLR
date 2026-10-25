"""Relink qualified standalone objects and interleave original/split validation.
Requires retained scalar cohort recipes; records current transitive native inputs.
"""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--prior',type=Path,required=True)
p.add_argument('--objects',type=Path,required=True)
p.add_argument('--samples',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--rounds',type=int,default=5)
p.add_argument('--stack-bounds',action='store_true',help='Compare split/grouped stack-bound TLS with unchanged root validation')
p.add_argument('--combine-validation',action='store_true',help='Combine grouped bounds and split storage validation')
a=p.parse_args();assert a.rounds>=3
assert not a.combine_validation or a.stack_bounds
modes=['control','combined' if a.combine_validation else 'grouped'] if a.stack_bounds else ['control','split']
variant='NEOCLR_GROUP_STACK_BOUNDS' if a.stack_bounds else 'NEOCLR_GC_SPLIT_STORAGE_VALIDATION'
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
prior=json.loads(a.prior.read_text());samples=json.loads(a.samples.read_text())
assert prior['passed'] and samples['passed'] and samples['iterations']==1
r=dict(passed=False,scope='Matched grouped stack bounds plus split storage validation.' if a.combine_validation else 'Matched stack-bound TLS layout; unchanged root validation.' if a.stack_bounds else __doc__,variant=variant,combinedValidation=a.combine_validation,commands=[],trials=[],rows=[],inputs={})
def run(command):
 command=list(map(str,command));x=subprocess.run(command,capture_output=True,text=True,timeout=180)
 r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 gc=out/'native-gc.c';provider=out/'text-arena.c'
 run(['python3',root/'runtime/native/gc/concurrent/generate.py','--output',gc])
 run(['python3',root/'runtime/native/gc/concurrent/native-providers.py','--output',provider])
 for case in samples['cases']:
  name=case['sample'];suffix=name+'-scalar'
  cmd=next(c['command'].copy() for c in prior['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1].endswith('/'+suffix))
  old=next(Path(v) for v in cmd if v.endswith('/'+suffix+'.o'))
  obj=a.objects.resolve()/(suffix+'.o');assert obj.read_bytes()==old.read_bytes()
  cmd=[str(obj) if v==str(old) else str(gc) if v.endswith('/native-gc.c') else str(provider) if v.endswith('/text-arena.c') else v.replace(str(root/'benchmarks/execution/concurrent-mutation-gate'),str(root/'runtime/native/gc/concurrent')) for v in cmd]
  assert not any('STORAGE_VALIDATION' in v for v in cmd)
  for mode in modes:
   link=cmd.copy();link[link.index('-o')+1]=str(out/(name+'-'+mode))
   if mode!= 'control':
    link+=['-D'+variant]
    if a.combine_validation:link+=['-DNEOCLR_GC_SPLIT_STORAGE_VALIDATION']
   print('Build',name,mode,flush=True);run(link)
 # Snapshot current inputs before timing, including canonical transitive sources.
 paths=[Path(__file__),a.prior,a.samples,*root.glob('runtime/native/**/*.c'),*root.glob('runtime/native/**/*.h'),*root.glob('runtime/native/gc/concurrent/*.py')]
 for c in r['commands']:paths += [Path(v) for v in c['command'] if Path(v).is_file()]
 r['inputs']={str(p.resolve()):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
 for iteration in range(a.rounds):
  for case in samples['cases']:
   name=case['sample']
   for mode in (modes if iteration%2==0 else list(reversed(modes))):
    x=run([out/(name+'-'+mode)]);assert x.stdout==case['expectedPerIteration']
    v=next(line.split()[1:] for line in x.stderr.splitlines() if line.startswith('PROFILE '))
    counts=list(map(int,next(line.split()[1:] for line in x.stderr.splitlines() if line.startswith('CANDIDATE '))))
    assert len(counts)==6 and counts[0]==counts[1]+counts[2] and counts[5]==0
    r['trials'].append(dict(sample=name,mode=mode,round=iteration,warmup=iteration==0,cpuSeconds=float(v[0]),wallSeconds=float(v[1]),allocations=int(v[2]),allocatedBytes=int(v[3]),collections=int(v[4]),peakLiveBytes=int(v[5]),peakArenaBytes=int(v[6]),candidateCounts=counts))
 for case in samples['cases']:
  ts=[t for t in r['trials'] if t['sample']==case['sample']]
  assert len({(t['allocations'],t['allocatedBytes']) for t in ts})==1
  for mode in modes:
   xs=[t for t in ts if t['mode']==mode and not t['warmup']]
   r['rows'].append(dict(sample=case['sample'],mode=mode,**{k:statistics.median(t[k] for t in xs) for k in ['cpuSeconds','wallSeconds','allocations','allocatedBytes','peakArenaBytes']}))
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
