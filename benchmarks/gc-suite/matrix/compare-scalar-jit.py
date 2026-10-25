"""Matched JIT scalar-store control/candidate comparison using qualified AOT hosts.
Invocation CPU includes collector workers/cleanup; JIT compilation is separate.
"""
import argparse,hashlib,json,os,platform,statistics,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--aot',type=Path,required=True)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--rounds',type=int,default=4)
p.add_argument('--root-validation',action='store_true',help='Compare original/split root validators with the same typed-store compiler policy')
a=p.parse_args();assert a.rounds>=3
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
built=json.loads((a.build/'results.json').read_text());prior=json.loads((a.aot/'results.json').read_text())
assert built['passed'] and prior['passed'] and built['iterations']==1
modes=['control','split'] if a.root_validation else ['control','scalar']
if a.root_validation:
 for path,digest in prior['inputs'].items():
  assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale AOT input',path)
r=dict(passed=False,rootValidation=a.root_validation,scope='Matched JIT root-validator comparison; compilation is separate.' if a.root_validation else __doc__,system=platform.platform(),commands=[],trials=[],rows=[],inputs=dict(prior['inputs']) if a.root_validation else {},rounds=a.rounds)
def run(cmd,env=None):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=180)
 r['commands'].append(dict(command=cmd,environment=None if env is None else {k:v for k,v in env.items() if k.startswith('NEOCLR_')},exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 control_link=None
 for mode in modes:
  cmd=next(c['command'].copy() for c in prior['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1]==str(a.aot.resolve()/('compute-'+mode)))
  objects=[v for v in cmd if v.endswith('.o')];assert len(objects)==1
  obj=objects[0]
  if a.root_validation:
   assert cmd.count('-DNEOCLR_GC_SPLIT_STORAGE_VALIDATION')==(mode=='split')
   normalized=[v for v in cmd if v!='-DNEOCLR_GC_SPLIT_STORAGE_VALIDATION']
   normalized[normalized.index('-o')+1]='OUTPUT'
   if control_link is None:control_link=normalized
   else:assert normalized==control_link, 'root comparison must use identical objects and host inputs'
  cmd=[v for v in cmd if v!=obj and not v.endswith('/main.c')]+['-dynamiclib']
  cmd[cmd.index('-o')+1]=str(out/(mode+'.dylib'));run(cmd)
 for iteration in range(a.rounds):
  for case in built['cases']:
   name=case['sample'];native=next(n for n in case['native'] if n['mode']=='live')
   for mode in (modes if iteration%2==0 else list(reversed(modes))):
    print(iteration,name,mode,flush=True)
    original=native['selectionCommand']
    selection=[original[0],'--jit-managed',*original[1:4],str(out/(mode+'.dylib')),*original[5:]]
    path=out/(name+'-'+mode+'.json');path.write_text(json.dumps(selection))
    env=dict(os.environ,**native['environment']);env.pop('NEOCLR_LIVE_SCALAR_ARRAY_STORES',None)
    env.pop('NEOCLR_LIVE_GENERIC_SCALAR_ARRAY_STORES',None)
    env['NEOCLR_LIVE_SELECTION_COMMAND']=str(path)
    if a.root_validation:
     for flag in ['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS','NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS','NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS','NEOCLR_LIVE_SSA_BYREF_ARGUMENTS']:
      env.pop(flag,None)
     env['NEOCLR_LIVE_STACK_POLICY']='qualified'
    elif mode=='control':env['NEOCLR_LIVE_GENERIC_SCALAR_ARRAY_STORES']='1'
    x=run([a.runner.resolve(),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture'],env)
    guest=x.stdout.split('running 1 test\n',1)[1].split('test mutation_store::tests::full_http_selection_compiles_with_live_gc ... ok',1)[0]
    assert guest==case['expectedPerIteration'],repr(guest)
    values=next(line.split()[1:] for line in x.stderr.splitlines() if line.startswith('PROFILE '))
    counts=list(map(int,next(line.split()[1:] for line in x.stderr.splitlines() if line.startswith('CANDIDATE '))))
    jit=json.loads(next(line[len('JIT_RESULT '):] for line in x.stderr.splitlines() if line.startswith('JIT_RESULT ')))
    assert jit['guestExitCode']==0 and len(counts)==6 and counts[0]==counts[1]+counts[2] and counts[5]==0
    r['trials'].append(dict(sample=name,mode=mode,round=iteration,warmup=iteration==0,cpuSeconds=float(values[0]),wallSeconds=float(values[1]),allocations=int(values[2]),allocatedBytes=int(values[3]),collections=int(values[4]),peakLiveBytes=int(values[5]),peakArenaBytes=int(values[6]),candidateCounts=counts,jit=jit))
    (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
 for case in built['cases']:
  name=case['sample'];ts=[t for t in r['trials'] if t['sample']==name]
  expected={(t['allocations'],t['allocatedBytes']) for t in prior['trials'] if t['sample']==name}
  assert {(t['allocations'],t['allocatedBytes']) for t in ts}==expected and len(expected)==1
  for mode in modes:
   xs=[t for t in ts if t['mode']==mode and not t['warmup']]
   r['rows'].append(dict(sample=name,mode=mode,**{k:statistics.median(t[k] for t in xs) for k in ['cpuSeconds','wallSeconds','allocations','allocatedBytes','peakArenaBytes']}))
 for c in r['commands']:
  for value in c['command']:
   path=Path(value)
   if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 for case in built['cases']:
  for value in case['native'][0]['selectionCommand']:
   path=Path(value)
   if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),a.build/'results.json',a.aot/'results.json',*out.glob('*-control.json'),*out.glob('*-scalar.json'),*out.glob('*-split.json'),*out.glob('*.dylib')]:r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
