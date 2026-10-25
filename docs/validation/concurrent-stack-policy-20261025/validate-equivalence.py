import argparse,os,json,hashlib,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description="Verify integrated stack policy against retained candidate objects")
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--prior',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
prior=json.loads(a.prior.read_text())

r=dict(passed=False,cases=[])
try:
 for c in prior['commands']:
  oldenv=c.get('environment') or {};sel=oldenv.get('NEOCLR_LIVE_SELECTION_COMMAND','')
  if not sel.endswith('-scalar.json'):continue
  command=json.loads(Path(sel).read_text());oldobj=Path(command[4]);name=oldobj.stem;obj=out/(name+'.o');command[4]=str(obj);path=out/(name+'.json');path.write_text(json.dumps(command))
  env={k:v for k,v in os.environ.items() if not k.startswith('NEOCLR_')};env.update(NEOCLR_TEST_OPT_LEVEL='speed',NEOCLR_LIVE_SELECTION_COMMAND=str(path),NEOCLR_LIVE_SELECTION_MODE='live',NEOCLR_LIVE_STACK_POLICY='qualified')
  c['command'][0]=str(a.runner.resolve())
  x=subprocess.run(c['command'],env=env,capture_output=True,text=True,timeout=180)
  row=dict(name=name,command=c['command'],selection=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr);r['cases'].append(row);x.check_returncode()
  row['identical']=obj.read_bytes()==oldobj.read_bytes();row['sha256']=hashlib.sha256(obj.read_bytes()).hexdigest();row['priorObject']=str(oldobj);assert row['identical'];print(name,'identical',flush=True)
 assert len(r['cases'])==6
 r['passed']=True
finally:(out/'equivalence.json').write_text(json.dumps(r,indent=2)+'\n')
