"""Profile shared-pool handoffs on qualified retained compiled graph objects.
Diagnostic source copies only; original lifecycle workload and checks are retained.
"""
import argparse,hashlib,json,re,resource,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--local-allocation',action='store_true')
a=p.parse_args();root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,diagnosticOnly=__doc__,localAllocation=a.local_allocation,commands=[],trials=[],inputs={})
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 builds={mode:root/'target'/name for mode,name in [('sync','compiled-paced-owner-final'),('live','compiled-read-epochs-qualified')]}
 assert (builds['sync']/'code.o').read_bytes()==(builds['live']/'code.o').read_bytes()
 worker=root/'runtime/native/workers'
 for name in ['native-workers.c','compiled-workers.inc','allocation-owner.inc']:
  source=(worker/name).read_text()
  source=re.sub(r'#include "(.*?)"',lambda m:'#include "'+str((worker/m[1]).resolve())+'"' if m[1] not in ['compiled-workers.inc','allocation-owner.inc'] else m[0],source)
  if name=='native-workers.c':source=source.replace('#include <time.h>','#include <time.h>\n#include "'+str(base/'waits.h')+'"')
  (out/name).write_text(source)
 for mode,directory in builds.items():
  manifest=json.loads((directory/'results.json').read_text());assert manifest['passed']
  recipe=next(c['command'].copy() for c in manifest['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1].endswith('optimized-snapshot'))
  recipe=[str(out/'native-workers.c') if v==str(worker/'native-workers.c') else v for v in recipe]
  recipe[recipe.index('-o')+1]=str(out/mode)
  if a.local_allocation:recipe+=['-DNEOCLR_WORKER_LOCAL_ALLOCATION']
  run(recipe)
 for repetition in range(3):
  for mode in (['sync','live'] if repetition%2==0 else ['live','sync']):
   before=resource.getrusage(resource.RUSAGE_CHILDREN);began=time.perf_counter();x=run([out/mode]);after=resource.getrusage(resource.RUSAGE_CHILDREN)
   waits={name:dict(calls=int(count),wallNs=int(wall),threadCpuNs=int(cpu)) for name,count,wall,cpu in re.findall(r'^WAIT (\w+) (\d+) (\d+) (\d+)$',x.stderr,re.M)}
   assert len(waits)==9 and waits['compiled_boundary']['calls']>0
   assert (waits['compiled_allocate']['calls']==0) if a.local_allocation else (waits['compiled_allocate']['calls']>=4520)
   assert 'PASS compiled shared-heap graph/GC: 32 invocations' in x.stdout and 'PASS close during four running compiled invocations' in x.stdout
   assert 'OWNER ALLOCATION 3904 requests' in x.stdout and 'OWNER ALLOCATION 128 requests' in x.stdout and 'OWNER CLOSE 488 requests' in x.stdout
   r['trials'].append(dict(mode=mode,round=repetition,wallSeconds=time.perf_counter()-began,cpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime,waits=waits,stdout=x.stdout,stderr=x.stderr))
 for path in [Path(__file__),base/'waits.h',*out.glob('*.c'),*out.glob('*.inc'),*[build/'code.o' for build in builds.values()],*[build/'results.json' for build in builds.values()]]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
