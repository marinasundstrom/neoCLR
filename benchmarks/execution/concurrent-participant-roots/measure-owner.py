"""Bounded diagnostic cost of allocation routing, not HTTP performance acceptance."""
import argparse, hashlib, json, resource, statistics, subprocess, time
from pathlib import Path
p=argparse.ArgumentParser()
p.add_argument('--control',type=Path,required=True)
p.add_argument('--owner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--broadcast',type=Path)
a=p.parse_args()
assert not a.output.exists()
r={'passed':False,'trials':[],'inputs':{}}
try:
 directories={'control':a.control,'owner':a.owner}
 if a.broadcast:directories['broadcast']=a.broadcast
 manifests={name:json.loads((directory/'results.json').read_text()) for name,directory in directories.items()}
 assert all(m['passed'] for m in manifests.values())
 assert all((directory/'code.o').read_bytes()==(a.control/'code.o').read_bytes() for directory in directories.values())
 programs={name:(directory/'optimized-snapshot').resolve() for name,directory in directories.items()}
 for path in [Path(__file__),*programs.values(),*[directory/file for directory in directories.values() for file in ['code.o','results.json']]]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for repeat in range(6):
  names=list(programs);offset=repeat%len(names);order=names[offset:]+names[:offset]
  if repeat%2:order.reverse()
  for name in order:
   before=resource.getrusage(resource.RUSAGE_CHILDREN);start=time.monotonic_ns()
   x=subprocess.run([str(programs[name])],capture_output=True,text=True,timeout=60)
   wall=(time.monotonic_ns()-start)/1e9;after=resource.getrusage(resource.RUSAGE_CHILDREN)
   cpu=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime
   r['trials'].append(dict(name=name,repeat=repeat,warmup=repeat==0,wallSeconds=wall,cpuSeconds=cpu,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
   x.check_returncode()
   assert '24544 coordinated collections' in x.stdout and '448 coordinated collections' in x.stdout and 'PASS close during four running compiled invocations' in x.stdout
   if name!='control':assert 'OWNER ALLOCATION 3904 requests' in x.stdout and 'OWNER ALLOCATION 128 requests' in x.stdout and 'OWNER CLOSE 488 requests' in x.stdout
 r['medians']={name:{metric:statistics.median(t[metric] for t in r['trials'] if t['name']==name and not t['warmup']) for metric in ['wallSeconds','cpuSeconds']} for name in programs}
 r['ownerOverControl']={metric:r['medians']['owner'][metric]/r['medians']['control'][metric] for metric in ['wallSeconds','cpuSeconds']}
 r['passed']=True
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(r['medians']));print(json.dumps(r['ownerOverControl']))
