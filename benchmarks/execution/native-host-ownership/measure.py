#!/usr/bin/env python3
"""Compare external and owned native heaps with matched paced JIT/AOT entry hosts."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import threading
import time
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--ordinary',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--profile', action='store_true')
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[3]
r=dict(passed=False,commands=[],trials=[],inputs={},scope='One owner, matched paced JIT entry host, external versus owned heap; profiling runs are separate from capacity.')
def record(path):
 path=Path(path).resolve();r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
def cpu(pid):
 raw=subprocess.check_output(['ps','-p',str(pid),'-o','time='],text=True).strip()
 parts=raw.split(':');value=0.0
 for part in parts:value=value*60+float(part)
 return raw,value
try:
 ordinary=json.loads((a.ordinary/'results.json').read_text());assert ordinary['passed']
 build=json.loads((root/'target/primitive-comparison-build/build.json').read_text())
 native=next(x['command'] for x in build['commands'] if x.get('phase')=='native-compilation')
 obj=root/'target/root-map/candidate.o';client=root/'target/primitive-comparison-refresh/client';compiler=root/'target/release/neoclr-codegen'
 configs=[]
 source=(root/'docs/experiments/jit-managed/host.c').read_text()
 # Only this experiment opts into ownership. Every established-context exit releases it.
 assert source.count('free(heap);') == 4
 source=source.replace('free(heap);', 'if (neoclr_gc_heap_release_v1(&context.text)) abort(); free(heap);')
 anchor='neoclr_aot_context context = {.text = {heap, heap_bytes, 0}};'
 assert source.count(anchor)==1
 source=source.replace(anchor, anchor+'\n    if (neoclr_gc_heap_acquire_v1(&context.text)) { free(heap); return 2; }')
 owned=out/'owned-host.c';owned.write_text(source);record(owned)
 for ownership in ['external','owned']:
  original=a.ordinary.resolve()/'paced-False.dylib'
  base=next(x['command'].copy() for x in ordinary['commands'] if x['label']=='host-build' and str(original) in x['command'])
  if ownership=='owned':
   base=[str(owned) if x.endswith('/docs/experiments/jit-managed/host.c') else x for x in base]
  host=out/f'{ownership}.dylib'
  base[base.index('-o')+1]=str(host)
  x=subprocess.run(base,capture_output=True,text=True,timeout=90);r['commands'].append(dict(command=base,exitCode=x.returncode,stderr=x.stderr));x.check_returncode()
  cmd=base.copy();cmd.remove('-dynamiclib');exe=out/f'{ownership}-aot';cmd[cmd.index('-o')+1]=str(exe);cmd += [str(a.ordinary.resolve()/'main.c'),str(obj)]
  x=subprocess.run(cmd,capture_output=True,text=True,timeout=90);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stderr=x.stderr));x.check_returncode()
  configs += [(f'aot-{ownership}',[str(exe)]),(f'jit-{ownership}',[str(compiler),'--jit-managed','--closed-world',native[2],native[3],str(host),*native[5:]])]
  record(host);record(exe)
 if a.profile: configs=[x for x in configs if x[0].endswith('owned')]
 for label,cmd in (configs if a.profile else configs+configs[::-1]):
  start=time.perf_counter();proc=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  lines,errors,events=[],[],queue.Queue()
  def read(stream,target):
   for line in stream:target.append(line);events.put(line)
  threads=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
  for thread in threads:thread.start()
  try:
   while True:
    line=events.get(timeout=60)
    if line.startswith('READY '):break
   _,port,control=line.split();ready=time.perf_counter()-start
   raw_before,before=cpu(proc.pid)
   def load(seconds):
    x=subprocess.run([str(client),port,str(seconds)],env=dict(os.environ,GOMAXPROCS='2'),capture_output=True,text=True,timeout=45);x.check_returncode();return json.loads(x.stdout)
   warmup=load(1)
   sampler=None
   if a.profile:
    profile_path=out/(label+'.sample.txt')
    sampler=subprocess.Popen(['/usr/bin/sample',str(proc.pid),'3','1','-file',str(profile_path)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
   trial=load(5)
   if sampler:
    sample_out,sample_err=sampler.communicate(timeout=30)
    assert sampler.returncode==0,(sample_out,sample_err)
    record(profile_path)
   raw_after,after=cpu(proc.pid)
   with socket.create_connection(('127.0.0.1',int(control)),timeout=5):pass
   proc.wait(timeout=30)
   for thread in threads:thread.join(timeout=2)
   assert proc.returncode==0,(lines,errors)
   count=sum(x['completed']+x['drained'] for x in [warmup,trial])
   entry=dict(label=label,command=cmd,readySeconds=ready,trial=trial,warmup=warmup,
              windowCpuSeconds=after-before,windowCpuMicrosecondsPerResponse=(after-before)*1e6/count,
              psCpuBefore=raw_before,psCpuAfter=raw_after,stdout=lines,stderr=errors)
   if label.startswith('jit'):
    entry['jit']=json.loads(next(x[len('JIT_RESULT '):] for x in errors if x.startswith('JIT_RESULT ')))
    assert entry['jit']['guestExitCode']==0
   r['trials'].append(entry)
   print(json.dumps({k:entry[k] for k in ['label','readySeconds','windowCpuMicrosecondsPerResponse']}|{'requestsPerSecond':trial['requestsPerSecond']}),flush=True)
  finally:
   if proc.poll() is None:proc.kill()
   proc.wait()
   for thread in threads:thread.join(timeout=2)
  (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
 for path in [Path(__file__),a.ordinary/'results.json',obj,client,compiler,root/'docs/experiments/jit-managed/host.c',*(root/'runtime/native').rglob('*.[ch]')]:record(path)
 r['passed']=True
finally:
 (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
