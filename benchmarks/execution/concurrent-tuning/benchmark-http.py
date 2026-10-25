"""Recurring plaintext/JSON sync versus tuned concurrent HTTP comparison.

Retain raw reports and exact collector policies; diagnostic pause runs are separate
from throughput evidence. This does not disable collection or claim saturation.
"""
import argparse,hashlib,json,platform,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--index-only',action='store_true')
p.add_argument('--output',type=Path,required=True)
p.add_argument('--sync-build',type=Path,default=Path('target/concurrent-workloads/optimizer-http-build'))
p.add_argument('--plain-live-object',type=Path,default=Path('target/stable-byref-plain-fixed-build/on.o'))
p.add_argument('--json-live-object',type=Path,default=Path('target/stable-byref-json-build/on.o'))
p.add_argument('--requests',type=int,default=4096)
p.add_argument('--rounds',type=int,default=3)
p.add_argument('--concurrency',type=int,default=4)
p.add_argument('--allocation-budget',type=int,default=1024)
p.add_argument('--publication-budget',type=int,default=16384)
p.add_argument('--finish-watermark',type=int,choices=range(1,100),default=50)
p.add_argument('--profile-owner-pauses',action='store_true')
a=p.parse_args();assert a.requests>0 and a.rounds>0 and a.allocation_budget>0 and a.publication_budget>0
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
mode=f'live-budget-{a.allocation_budget}-{a.publication_budget}-inline-index'
if a.index_only:mode=mode.replace('-inline-index','-index-only-inline-index')
r=dict(passed=False,scope='Loopback fixed-work AOT, checked response bodies and EOF; no saturation or cross-framework ranking',diagnosticOnly=a.profile_owner_pauses,system=platform.platform(),revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),requests=a.requests,rounds=a.rounds,concurrency=a.concurrency,syncPolicy=dict(allocations=256,publications=4096,heapBytes=1048576),livePolicy=dict(allocations=a.allocation_budget,publications=a.publication_budget,watermark=a.finish_watermark,heapBytes=1048576,gateWords=4096,indexOnly=a.index_only),commands=[],samples={})
try:
 for sample,obj in [('plain',a.plain_live_object),('json',a.json_live_object)]:
  print('Benchmark',sample,flush=True)
  command=['python3',str(root/'benchmarks/execution/concurrent-store/measure-plaintext.py'),'--sync-object',str((a.sync_build/f'{sample}-sync.o').resolve()),'--live-object',str(obj.resolve()),'--output',str(out/sample),'--sample',sample,'--requests',str(a.requests),'--pairs',str(a.rounds),'--concurrency',str(a.concurrency),'--allow-allocation-variation','--compare-allocation-index','--compare-exact-allocation-index','--compare-inline-base','--live-budgets',f'{a.allocation_budget}:{a.publication_budget}','--finish-watermark',str(a.finish_watermark),'--modes','sync',mode]
  if a.index_only:command+=['--compare-index-only']
  if a.profile_owner_pauses:command+=['--profile-owner-pauses']
  x=subprocess.run(command,cwd=root,text=True,capture_output=True,timeout=600)
  r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
  path=out/sample/'results.json';data=json.loads(path.read_text());assert data['passed']
  assert data['budgetsByMode']=={'sync':{'allocations':256,'publications':4096},mode:{'allocations':a.allocation_budget,'publications':a.publication_budget}}
  assert bool(data.get('diagnosticOnly'))==a.profile_owner_pauses
  r['samples'][sample]=dict(report=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest(),summary=data['summary'])
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
