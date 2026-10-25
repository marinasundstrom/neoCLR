"""Read-only forwarding proof audit against five retained checked-store objects."""
import argparse, hashlib, json, os, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--root',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],inputs={});runner=a.runner.resolve()
def record(path):r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
try:
 for sample in ['tasks','collections','arrays','plain','json']:
  ordinary=sample not in ['plain','json']
  build=a.root/('borrowed-managed-ordinary-build' if ordinary else 'borrowed-managed-http-build')/'results.json'
  record(build);data=json.loads(build.read_text());assert data['passed']
  original=next(n for c in data['cases'] if c['sample']==sample for n in c['native'] if n['mode']=='live') if ordinary else next(c for c in data['commands'] if c['sample']==sample)
  reference=a.root/'stack-elision-ordinary-build'/sample/'live.o' if ordinary else a.root/'stack-elision-http-build'/(sample+'.o')
  oldaudit=a.root/'borrowed-managed-ordinary-build'/sample/'audit.json' if ordinary else a.root/'borrowed-managed-http-build'/(sample+'-audit.json')
  prior=json.loads(oldaudit.read_text());record(oldaudit)
  command=original['selectionCommand'].copy();command[4]=str(out/(sample+'.o'))
  selection=out/(sample+'-command.json');selection.write_text(json.dumps(command,indent=2)+'\n');audit=out/(sample+'-audit.json')
  env=dict(os.environ,NEOCLR_TEST_OPT_LEVEL='speed',NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS='1',NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS='1',NEOCLR_LIVE_SELECTION_COMMAND=str(selection),NEOCLR_LIVE_SELECTION_MODE='live',NEOCLR_LIVE_STACK_ARGUMENT_AUDIT=str(audit))
  for key in ['NEOCLR_LIVE_BORROW_STORES','NEOCLR_LIVE_CONDITIONAL_BORROW','NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS']:env.pop(key,None)
  invoke=[str(runner),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']
  x=subprocess.run(invoke,env=env,capture_output=True,text=True,timeout=300)
  row=dict(sample=sample,command=invoke,selectionCommand=command,environment={k:v for k,v in env.items() if k.startswith('NEOCLR_')},exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr);r['commands'].append(row);x.check_returncode()
  current=json.loads(audit.read_text());assert current['forwarding']
  before={f['name']:f for f in prior['functions']};after={f['name']:f for f in current['functions']};assert before.keys()==after.keys()
  assert all(after[name]['candidateStores']>=before[name]['candidateStores'] for name in before)
  row['directCandidates']=sum(f['candidateStores'] for f in before.values());row['forwardedCandidates']=sum(f['candidateStores'] for f in after.values())
  row['changedFunctions']=[dict(name=name,before=before[name],after=after[name]) for name in before if before[name]['candidateStores']!=after[name]['candidateStores']]
  row['byteIdentical']=reference.read_bytes()==Path(command[4]).read_bytes();assert row['byteIdentical']
  for path in [reference,Path(command[4]),audit,runner]:record(path)
  print(sample,row['directCandidates'],row['forwardedCandidates'],flush=True)
 record(Path(__file__));record(Path('runtime/codegen/src/mutation_stack_audit.rs'));r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
