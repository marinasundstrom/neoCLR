"""Run versioned GC samples as benchmarks or separately instrumented profiles."""
import argparse,hashlib,json,os,platform,statistics,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('kind',choices=['benchmark','profile'])
p.add_argument('--output',type=Path,required=True)
p.add_argument('--suite',choices=['all','http','standalone'],default='all')
p.add_argument('--build',type=Path,help='Reuse a qualified allocation-patterns build')
p.add_argument('--runner',type=Path,default=Path('target/release/deps/neoclr_codegen-12f33f26d2ff4df0'))
p.add_argument('--rounds',type=int,default=5)
p.add_argument('--requests',type=int,default=4096)
p.add_argument('--baseline',type=Path,help='Previous run.json for same-collector history comparison')
a=p.parse_args();assert a.rounds>0 and a.requests>0
root=Path(__file__).resolve().parents[2];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
spec_path=root/'benchmarks/gc-suite/samples.json';spec=json.loads(spec_path.read_text())
h=hashlib.sha256(spec_path.read_bytes())
for case in spec['cases']:h.update((root/'benchmarks/gc-suite'/case['source']).read_bytes())
hardware=dict(cpu=platform.processor(),logicalCpus=os.cpu_count(),memoryBytes=None)
if platform.system()=='Darwin':
 values=subprocess.check_output(['sysctl','-n','machdep.cpu.brand_string','hw.ncpu','hw.memsize'],text=True).splitlines()
 hardware=dict(cpu=values[0],logicalCpus=int(values[1]),memoryBytes=int(values[2]))
r=dict(passed=False,hardware=hardware,schemaVersion=1,kind=a.kind,suite=a.suite,sampleDefinitionSha256=h.hexdigest(),system=platform.platform(),rounds=a.rounds,requests=a.requests,policies={'sync':{'allocationBudget':256,'publicationBudget':4096},'concurrent':{'allocationBudget':1024,'publicationBudget':16384,'finishWatermark':50}},commands=[],rows=[],reports={})
def run(cmd):
 cmd=list(map(str,cmd));print('Run:', ' '.join(cmd),flush=True)
 env=dict(os.environ,NEOCLR_TEST_OPT_LEVEL='speed')
 x=subprocess.run(cmd,cwd=root,env=env,capture_output=True,text=True,timeout=1800)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
def keep(name,path):
 data=json.loads(path.read_text());assert data['passed'],path
 r['reports'][name]=dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
 return data
def median(ts,key):return statistics.median(t[key] for t in ts)
try:
 if a.suite in ['all','http']:
  command=['python3',root/'benchmarks/execution/concurrent-tuning/benchmark-http.py','--output',out/'http','--rounds',a.rounds,'--requests',a.requests]
  if a.kind=='profile':command+=['--profile-owner-pauses']
  run(command);keep('http',out/'http/results.json')
  for sample in ['plain','json']:
   data=keep('http-'+sample,out/'http'/sample/'results.json')
   assert all(t['requests']==a.requests and t['warmup']==16 and t['clientConcurrency']==4 for t in data['trials'])
   for mode in ['sync','live-budget-1024-16384-inline-index']:
    ts=[t for t in data['trials'] if t['mode']==mode]
    r['rows'].append(dict(sample='http-'+sample,collector='sync' if mode=='sync' else 'concurrent',cpuSeconds=median(ts,'processCpuSeconds'),unitsPerSecond=median(ts,'requestsPerSecond'),unit='requests',allocations=median(ts,'allocations'),peakArenaBytes=statistics.median(t['gcProfile']['peakArenaUsed'] for t in ts),allocatedBytes=statistics.median(t['gcProfile']['allocatedBytes'] for t in ts)))
 if a.suite in ['all','standalone']:
  build=a.build.resolve() if a.build else out/'build'
  if not a.build:run(['python3',root/'benchmarks/execution/concurrent-workloads/build.py','--suite','allocation-patterns','--stable-byrefs','--runner',a.runner.resolve(),'--output',build,'--iterations',1])
  built=keep('build',build/'results.json')
  assert built.get('suite')=='allocation-patterns' and built.get('stableByrefs') and built['optimizationLevel']=='speed'
  assert built['iterations']==1
  assert {c['sample'] for c in built['cases']}=={c['name'] for c in spec['cases']}
  for case in spec['cases']:
   src=root/'benchmarks/gc-suite'/case['source']
   assert built['inputs'][str(src.resolve())]==hashlib.sha256(src.read_bytes()).hexdigest(),('stale sample build',src)
  command=['python3',root/'benchmarks/execution/concurrent-workloads/measure.py','--build',build,'--output',out/'standalone','--rounds',a.rounds,'--allocation-index','--compare-exact-allocation-index','--compare-inline-base','--live-budgets','1024:16384','--finish-watermark',50,'--modes','sync','live-budget-1024-16384-hybrid-inline']
  if a.kind=='profile':command+=['--profile-owner-pauses','--profile-begin','--profile-finish','--profile-collection']
  run(command);data=keep('standalone',out/'standalone/results.json');assert not data['excluded']
  for case in spec['cases']:
   for mode in ['sync','live-budget-1024-16384-hybrid-inline']:
    ts=[t for t in data['trials'] if t['sample']==case['name'] and t['mode']==mode];assert len(ts)==a.rounds
    r['rows'].append(dict(sample=case['name'],collector='sync' if mode=='sync' else 'concurrent',cpuSeconds=median(ts,'cpuSeconds'),unitsPerSecond=case['itemsPerInvocation']/median(ts,'wallSeconds'),unit='items',allocations=median(ts,'allocations'),allocatedBytes=median(ts,'allocatedBytes'),peakArenaBytes=median(ts,'peakArenaUsed'),completedConcurrentEpochs=statistics.median(t.get('candidateCounts',[0,0])[1] for t in ts)))
 if a.kind=='profile':
  paths=[x['path'] for key,x in r['reports'].items() if key in ['http-plain','http-json','standalone']]
  run(['python3',root/'benchmarks/execution/concurrent-workloads/summarize-owner-pauses.py','--inputs',*paths,'--output',out/'owner-pauses.json']);keep('owner-pauses',out/'owner-pauses.json')
  if 'standalone' in r['reports']:
   for name in ['begin-phases','finish-phases','collection-parts']:
    run(['python3',root/f'benchmarks/execution/concurrent-workloads/summarize-{name}.py','--inputs',r['reports']['standalone']['path'],'--output',out/(name+'.json')]);keep(name,out/(name+'.json'))
 r['passed']=True
 (out/'run.json').write_text(json.dumps(r,indent=2)+'\n')
 if a.baseline:run(['python3',root/'benchmarks/gc-suite/compare.py',a.baseline,out/'run.json','--output',out/'self-comparison.json'])
except Exception:
 r['passed']=False
 raise
finally:(out/'run.json').write_text(json.dumps(r,indent=2)+'\n')
