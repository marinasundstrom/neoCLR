"""Run HTTP lifecycle tests and require owner-local completed live epochs and cleanup."""
import argparse,json,subprocess,re,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
build=a.build.resolve();manifest=json.loads((build/'results.json').read_text());assert manifest['passed']
r={'passed':False,'cases':[],'inputs':{}}
try:
 for mode in ['sync','live']:
  for workers in [1,4]:
   directory=out/f'{mode}-{workers}';exe=build/f'{mode}-{workers}'
   command=['python3',str(root/'docs/experiments/http-worker-integration/validate.py'),'--executable',str(exe),'--workers',str(workers),'--requests','256','--output',str(directory)]
   x=subprocess.run(command,capture_output=True,text=True,timeout=180)
   row=dict(mode=mode,workers=workers,command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr);r['cases'].append(row);x.check_returncode()
   evidence=json.loads((directory/'validation.json').read_text());row['validation']=evidence;assert evidence['passed']
   for trial in evidence['runs']:
    errors=''.join(trial['stderr'])
    allocations=re.findall(r'^ALLOCATIONS (\d+)$',errors,re.M);assert len(allocations)==workers
    if mode=='live':
     counts=[list(map(int,line.split())) for line in re.findall(r'^CANDIDATE ([\d ]+)$',errors,re.M)]
     indices=[list(map(int,line.split())) for line in re.findall(r'^INDEX_METADATA ([\d ]+)$',errors,re.M)]
     assert len(counts)==workers and len(indices)==workers
     assert all(len(c)==6 and c[0]==c[1]+c[2] and c[5]==0 for c in counts)
     if not trial['injectedFault']:assert all(c[1]>0 for c in counts)
     assert all(len(i)==3 and i[0]>0 and i[1]==0 for i in indices)
     trial['checkedCandidateCounts']=counts;trial['checkedIndexMetadata']=indices
   for path in [exe,directory/'validation.json']:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),build/'results.json',root/'docs/experiments/http-worker-integration/validate.py']:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
