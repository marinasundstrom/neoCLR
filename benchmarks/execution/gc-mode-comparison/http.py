#!/usr/bin/env python3
"""Warm plaintext comparison with identical native entry hosts and explicit GC policy."""
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
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[3]
r=dict(passed=False,commands=[],trials=[],inputs={},scope='One owner, identical JIT entry host, external heap, explicit forced/paced GC; not the isolated-worker optimized AOT host.')
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
 for policy in ['forced','paced']:
  host=a.ordinary.resolve()/f'{policy}-False.dylib'
  cmd=next(x['command'].copy() for x in ordinary['commands'] if x['label']=='host-build' and str(host) in x['command'])
  cmd.remove('-dynamiclib');exe=out/f'{policy}-aot';cmd[cmd.index('-o')+1]=str(exe);cmd += [str(a.ordinary.resolve()/'main.c'),str(obj)]
  x=subprocess.run(cmd,capture_output=True,text=True,timeout=90);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stderr=x.stderr));x.check_returncode()
  configs += [(f'aot-{policy}',[str(exe)]),(f'jit-{policy}',[str(compiler),'--jit-managed','--closed-world',native[2],native[3],str(host),*native[5:]])]
  record(host);record(exe)
 for label,cmd in configs+configs[::-1]:
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
   warmup=load(1);trial=load(5)
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
 for path in [Path(__file__),a.ordinary/'results.json',obj,client,compiler]:record(path)
 r['passed']=True
finally:
 (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
