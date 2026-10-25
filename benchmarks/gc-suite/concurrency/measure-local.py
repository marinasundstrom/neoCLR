"""Interleave qualified shared-pool sync/live service/local allocation controls.
No wait instrumentation. Whole lifecycle CPU/wall, not an HTTP capacity claim.
"""
import argparse,hashlib,json,resource,statistics,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();assert not a.output.exists();root=Path(__file__).resolve().parents[3]
builds={name:root/'target'/directory for name,directory in [
 ('sync-service','gc-local-allocation-control-sync-v1'),('live-service','gc-local-allocation-control-live-v1'),
 ('sync-local','gc-local-allocation-sync-v1'),('live-local','gc-local-allocation-live-v1')]}
r=dict(passed=False,scope=__doc__,trials=[],inputs={})
try:
 for name,d in builds.items():
  manifest=json.loads((d/'results.json').read_text());assert manifest['passed']
  assert (d/'code.o').read_bytes()==(builds['sync-service']/'code.o').read_bytes()
  for path in [d/'results.json',d/'code.o',d/'optimized-snapshot']:
   r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for repeat in range(7):
  names=list(builds);i=repeat%4;names=names[i:]+names[:i]
  if repeat%2:names.reverse()
  for name in names:
   before=resource.getrusage(resource.RUSAGE_CHILDREN);start=time.perf_counter()
   x=subprocess.run([str(builds[name]/'optimized-snapshot')],capture_output=True,text=True,timeout=60)
   after=resource.getrusage(resource.RUSAGE_CHILDREN)
   t=dict(name=name,round=repeat,warmup=repeat==0,wallSeconds=time.perf_counter()-start,cpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr)
   r['trials'].append(t);x.check_returncode()
   for text in ['OWNER ALLOCATION 3904 requests','OWNER ALLOCATION 128 requests','OWNER CLOSE 488 requests','PASS compiled shared-heap graph/GC: 32 invocations','PASS close during four running compiled invocations']:assert text in x.stdout
   if name.startswith('live'):assert 'READ EPOCHS 20 started 20 finished' in x.stdout
 r['medians']={name:{key:statistics.median(t[key] for t in r['trials'] if t['name']==name and not t['warmup']) for key in ['wallSeconds','cpuSeconds']} for name in builds}
 r['inputs'][str(Path(__file__).resolve())]=hashlib.sha256(Path(__file__).read_bytes()).hexdigest();r['passed']=True
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(r['medians'],indent=2))
