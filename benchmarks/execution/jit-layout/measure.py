#!/usr/bin/env python3
"""Measure placement with unchanged JIT lowering; profile in separate invocations."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import threading

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--profile', action='store_true')
p.add_argument('--output', type=Path, required=True)
a=p.parse_args()
root=Path(__file__).resolve().parents[3]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
prior=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())
command=next(x['command'].copy() for x in prior['trials'] if x['label']=='jit')
command[0]=str(root/'target/jit-layout/experiment-codegen')
client=root/'target/primitive-comparison-refresh/client'
r=dict(passed=False,profile=a.profile,trials=[],inputs={})

def record(path):
    path=Path(path);r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()

def save():
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

def cpu(pid):
    raw=subprocess.check_output(['ps','-p',str(pid),'-o','time='],text=True).strip()
    seconds=0
    for part in raw.split(':'):seconds=seconds*60+float(part)
    return raw,seconds

try:
    labels=['system','arena'] if a.profile else ['system','arena','arena','system']
    for index,label in enumerate(labels):
        env=dict(os.environ);env.pop('NEOCLR_JIT_ARENA_EXPERIMENT',None);env.pop('PERF_BUILDID_DIR',None)
        if label=='arena':env['NEOCLR_JIT_ARENA_EXPERIMENT']='1'
        if a.profile:env['PERF_BUILDID_DIR']=str(out)
        row=dict(label=label,command=command,environment={k:env[k] for k in ['NEOCLR_JIT_ARENA_EXPERIMENT','PERF_BUILDID_DIR'] if k in env})
        r['trials'].append(row)
        proc=subprocess.Popen(command,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        events=queue.Queue();lines=[];errors=[]
        def read(stream,dest):
            for line in stream:dest.append(line);events.put(line)
        readers=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
        for reader in readers:reader.start()
        sampler=None
        try:
            while True:
                line=events.get(timeout=60)
                if line.startswith('READY '):break
            _,port,control=line.split()
            if a.profile:
                source=Path(f'/tmp/perf-{proc.pid}.map')
                target=out/f'{label}.map';target.write_bytes(source.read_bytes());record(target)
            def load(seconds):
                x=subprocess.run([str(client),port,str(seconds)],env=dict(os.environ,GOMAXPROCS='2'),capture_output=True,text=True,timeout=45)
                x.check_returncode();return json.loads(x.stdout)
            before_raw,before=cpu(proc.pid)
            warmup=load(1)
            if a.profile:
                sample=out/f'{label}.sample.txt'
                sampler=subprocess.Popen(['/usr/bin/sample',str(proc.pid),'3','1','-file',str(sample)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            trial=load(5)
            if sampler:
                stdout,stderr=sampler.communicate(timeout=30)
                row['sampler']=dict(exitCode=sampler.returncode,stdout=stdout,stderr=stderr)
                assert sampler.returncode==0
                record(sample)
            after_raw,after=cpu(proc.pid)
            with socket.create_connection(('127.0.0.1',int(control)),timeout=5):pass
            proc.wait(timeout=30)
            for reader in readers:reader.join(timeout=2)
            assert proc.returncode==0
            result=json.loads(next(x[11:] for x in errors if x.startswith('JIT_RESULT ')))
            assert result['guestExitCode']==0
            count=sum(x['completed']+x['drained'] for x in [warmup,trial])
            row.update(warmup=warmup,trial=trial,jit=result,psCpuBefore=before_raw,psCpuAfter=after_raw,cpuMicrosecondsPerResponse=(after-before)*1e6/count)
            print(json.dumps(dict(label=label,requestsPerSecond=trial['requestsPerSecond'],cpuMicrosecondsPerResponse=row['cpuMicrosecondsPerResponse'])),flush=True)
        finally:
            if sampler and sampler.poll() is None:sampler.kill();sampler.wait()
            if proc.poll() is None:proc.kill()
            proc.wait()
            for reader in readers:reader.join(timeout=2)
            row.update(exitCode=proc.returncode,stdout=lines,stderr=errors);save()
    for path in [Path(__file__),client,root/'benchmarks/execution/jit-layout/experiment.patch',root/'target/root-map/codegen.json',*[Path(x) for x in command if Path(x).is_file()]]:record(path)
    r['passed']=True
finally:
    save()
