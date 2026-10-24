"""Compile retained full HTTP selections through the current ordinary/live AOT pipeline."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--mode',choices=['both','sync','live'],default='both')
a=p.parse_args();root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r={'passed':False,'commands':[],'inputs':{},'scope':'Compilation only; no host execution or HTTP throughput claim.'}
try:
 runner=a.runner.resolve()
 for sample,recipe in [('plain','build.json'),('json','json-build.json')]:
  path=root/'target/root-map'/recipe;data=json.loads(path.read_text())
  original=data.get('command') or data['commands'][0]['command']
  for mode in (['sync','live'] if a.mode=='both' else [a.mode]):
   command=original.copy();command[4]=str(out/f'{sample}-{mode}.o')
   input_path=out/f'{sample}-{mode}-command.json';input_path.write_text(json.dumps(command,indent=2)+'\n')
   env=dict(os.environ,NEOCLR_LIVE_SELECTION_COMMAND=str(input_path),NEOCLR_LIVE_SELECTION_MODE=mode)
   args=[str(runner),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']
   try:
    x=subprocess.run(args,env=env,cwd=root,capture_output=True,text=True,timeout=240)
    r['commands'].append({'sample':sample,'mode':mode,'command':args,'selectionCommand':command,'exitCode':x.returncode,'stdout':x.stdout,'stderr':x.stderr})
   except subprocess.TimeoutExpired as e:
    r['commands'].append({'sample':sample,'mode':mode,'command':args,'selectionCommand':command,'timeoutSeconds':240,'stdout':str(e.stdout or ''),'stderr':str(e.stderr or '')})
    raise
   for item in command[2:]:
    candidate=Path(item)
    if candidate.is_file():r['inputs'][str(candidate)]=hashlib.sha256(candidate.read_bytes()).hexdigest()
 for path in [runner,Path(__file__),root/'runtime/codegen/src/lib.rs',root/'runtime/codegen/src/values.rs',root/'runtime/codegen/src/mutation_store_tests.rs',root/'runtime/codegen/src/mutation_admission.rs']:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=all(c.get('exitCode')==0 and '1 passed' in c['stdout'] for c in r['commands'])
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
raise SystemExit(0 if r['passed'] else 1)
