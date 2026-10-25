"""Checked HTTP execution/GC matrix, with connection policy and CPU scopes explicit.

Python loopback clients are not evidence of saturation or a framework ranking.
Prepare all artifacts first; run this tool without competing timed/build workloads.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import platform
import queue
import resource
import re
import socket
import statistics
import subprocess
import threading
import time
import traceback

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--modes',nargs='+',default=['interpreter','sync-aot','live-aot','sync-jit','live-jit'])
p.add_argument('--samples',nargs='+',choices=['plain','json'],default=['plain','json'])
p.add_argument('--protocols',nargs='+',choices=['close','keepalive','pipeline'],default=['close','keepalive','pipeline'])
p.add_argument('--requests',type=int,default=4096)
p.add_argument('--interpreter-requests',type=int,default=64)
p.add_argument('--warmup',type=int,default=16)
p.add_argument('--concurrency',type=int,default=4)
p.add_argument('--pipeline-depth',type=int,default=16)
p.add_argument('--rounds',type=int,default=3)
p.add_argument('--smoke',action='store_true',help='Label these runs as qualification, not timing evidence')
a=p.parse_args()
assert a.rounds>0 and a.requests>0 and a.interpreter_requests>0 and a.concurrency>0
assert a.requests%a.concurrency==0 and a.interpreter_requests%a.concurrency==0
assert a.pipeline_depth>0 and a.warmup>0
root=Path(__file__).resolve().parents[3]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
build_path=a.build.resolve()/'build.json';built=json.loads(build_path.read_text())
assert built['passed'] and all(mode in built['recipes'] for mode in a.modes)
for path,digest in built['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build input',path)
r=dict(passed=False,kind='profile' if built.get('diagnosticOnly') else 'qualification' if a.smoke else 'benchmark',scope=__doc__,system=platform.platform(),
       rounds=a.rounds,concurrency=a.concurrency,pipelineDepth=a.pipeline_depth,trials=[],rows=[],
       inputs={str(build_path):hashlib.sha256(build_path.read_bytes()).hexdigest()})
if built.get('diagnosticOnly'):r['diagnosticOnly']=built['diagnosticOnly']


def traffic(port,sample,protocol,count):
    expected=b'Hello, World!' if sample=='plain' else {'id':42,'name':'Café team','active':True}
    depth=a.pipeline_depth if protocol=='pipeline' else 1
    client=None
    pending=b''
    latencies=[]
    try:
        for start in range(0,count,depth):
            stop=min(start+depth,count)
            began=time.perf_counter()
            if client is None:
                client=socket.create_connection(('127.0.0.1',port),timeout=15)
                client.settimeout(15)
            request=b''
            for index in range(start,stop):
                close=protocol=='close' or index==count-1
                request+=b'GET /'+sample.encode()+b' HTTP/1.1\r\nHost: localhost\r\n'
                if close:request+=b'Connection: close\r\n'
                request+=b'\r\n'
            client.sendall(request)
            for index in range(start,stop):
                while b'\r\n\r\n' not in pending:
                    chunk=client.recv(65536);assert chunk,'EOF in headers';pending+=chunk
                    assert len(pending)<1048576,'unbounded response headers'
                header,pending=pending.split(b'\r\n\r\n',1)
                lines=header.split(b'\r\n');assert lines[0]==b'HTTP/1.1 200 OK',lines[0]
                fields={}
                for line in lines[1:]:
                    key,value=line.split(b':',1);key=key.lower()
                    assert key not in fields,'duplicate response field';fields[key]=value.strip()
                length=int(fields[b'content-length'])
                assert 0<length<=256,'unexpected response size'
                assert fields[b'content-type'].split(b';',1)[0]==(b'text/plain' if sample=='plain' else b'application/json')
                close=protocol=='close' or index==count-1
                assert (fields.get(b'connection')==b'close')==close
                while len(pending)<length:
                    chunk=client.recv(65536);assert chunk,'EOF in body';pending+=chunk
                body=pending[:length]
                assert (body if sample=='plain' else json.loads(body))==expected,body
                pending=pending[length:]
                latencies.append(time.perf_counter()-began)
            if protocol=='close' or stop==count:
                assert not pending and client.recv(1)==b'','missing expected EOF'
                client.close();client=None
        return latencies
    finally:
        if client is not None:client.close()


def trial(mode,sample,protocol,repetition):
    recipe=built['recipes'][mode]
    count=a.interpreter_requests if mode=='interpreter' else a.requests
    row=dict(mode=mode,execution=recipe['execution'],collector=recipe['collector'],sample=sample,
             protocol=protocol,round=repetition,requests=count,warmup=a.warmup,command=recipe['command'],
             environment=recipe['env'],passed=False)
    output=[];errors=[];lines=queue.Queue()
    env=dict(os.environ)
    for flag in ['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS','NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS',
                 'NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS','NEOCLR_LIVE_SSA_BYREF_ARGUMENTS']:
        env.pop(flag,None)
    env.update(recipe['env'])
    client_before=resource.getrusage(resource.RUSAGE_SELF)
    before=resource.getrusage(resource.RUSAGE_CHILDREN)
    began=time.perf_counter()
    process=subprocess.Popen(recipe['command'],cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    def drain(stream,target,notify=False):
        for line in stream:
            target.append(line)
            if notify:lines.put(line)
        if notify:lines.put(None)
    threads=[threading.Thread(target=drain,args=(process.stdout,output,True)),threading.Thread(target=drain,args=(process.stderr,errors))]
    for thread in threads:thread.start()
    try:
        deadline=time.monotonic()+90
        while True:
            line=lines.get(timeout=max(.01,deadline-time.monotonic()))
            assert line is not None,'server exited before readiness'
            if line.strip()=='READY':break
        port=int(lines.get(timeout=10));control=int(lines.get(timeout=10))
        row['readySeconds']=time.perf_counter()-began
        traffic(port,sample,protocol,a.warmup)
        started=time.perf_counter()
        with ThreadPoolExecutor(max_workers=a.concurrency) as pool:
            futures=[pool.submit(traffic,port,sample,protocol,count//a.concurrency) for _ in range(a.concurrency)]
            latencies=[latency for future in futures for latency in future.result()]
        duration=time.perf_counter()-started
        assert len(latencies)==count
        with socket.create_connection(('127.0.0.1',control),timeout=15):pass
        process.wait(timeout=30)
        for thread in threads:thread.join(timeout=5)
        assert process.returncode==0
        done=next(line.split() for line in output if line.startswith('DONE '))
        assert list(map(int,done[1:]))==[count+a.warmup,0],done
        latencies.sort()
        row.update(measuredWallSeconds=duration,requestsPerSecond=count/duration,
                   clientP50Seconds=latencies[(count-1)//2],clientP99Seconds=latencies[int((count-1)*.99)])
        profile=next((line.split()[1:] for line in errors if line.startswith('PROFILE ')),None)
        if mode!='interpreter':
            assert profile is not None
            row.update(invocationCpuSeconds=float(profile[0]),invocationWallSeconds=float(profile[1]),
                       allocations=int(profile[2]),allocatedBytes=int(profile[3]),collections=int(profile[4]),
                       peakLiveBytes=int(profile[5]),peakArenaBytes=int(profile[6]))
        if built.get('fiberSwitchProfile'):
            matches=re.findall(r'^FIBER_SWITCH (\d+) (\d+) (\d+) (\d+)$',''.join(errors),re.M)
            assert len(matches)==1,matches
            resumes,transfers,wall,cpu=map(int,matches[0]);assert transfers==2*resumes and resumes>0
            row['fiberSwitch']=dict(resumes=resumes,transfers=transfers,wallNs=wall,threadCpuNs=cpu)
        if built.get('fiberPhaseProfile'):
            matches=re.findall(r'^FIBER_PHASE (\w+) (\d+) (\d+) (\d+)$',''.join(errors),re.M)
            assert len(matches)==5,matches
            row['fiberPhases']={name:dict(count=int(count),wallNs=int(wall),threadCpuNs=int(cpu)) for name,count,wall,cpu in matches}
            assert len(row['fiberPhases'])==5
            for name in ['resume_prepare','resume_finish','yield_roots']:
                assert row['fiberPhases'][name]['count']==row['fiberSwitch']['resumes']
        if built.get('schedulerPhaseProfile'):
            matches=re.findall(r'^SCHEDULER_PHASE (\w+) (\d+) (\d+) (\d+)$',''.join(errors),re.M)
            assert len(matches)==3,matches
            phases={name:dict(count=int(count),wallNs=int(wall),threadCpuNs=int(cpu)) for name,count,wall,cpu in matches}
            assert set(phases)=={'poll','idle_gc','delivery_roots'}
            assert phases['poll']['count']==row['fiberPhases']['scheduler_acquire']['count']
            assert phases['poll']['count']==phases['idle_gc']['count']+phases['delivery_roots']['count']
            row['schedulerPhases']=phases
        if recipe['collector']=='live':
            counts=next(list(map(int,line.split()[1:])) for line in errors if line.startswith('CANDIDATE '))
            assert counts[0]==counts[1]+counts[2] and counts[5]==0
            row['candidateCounts']=counts
        if recipe['execution']=='jit':
            row['jit']=json.loads(next(line[len('JIT_RESULT '):] for line in errors if line.startswith('JIT_RESULT ')))
            assert row['jit']['guestExitCode']==0
        row['passed']=True
    except Exception:
        row['failure']=traceback.format_exc()
    finally:
        if process.poll() is None:
            process.kill();process.wait(timeout=5)
        for thread in threads:thread.join(timeout=5)
        after=resource.getrusage(resource.RUSAGE_CHILDREN)
        client_after=resource.getrusage(resource.RUSAGE_SELF)
        row.update(clientCpuSeconds=client_after.ru_utime+client_after.ru_stime-client_before.ru_utime-client_before.ru_stime,exitCode=process.returncode,stdout=''.join(output),stderr=''.join(errors),
                   wholeProcessWallSeconds=time.perf_counter()-began,
                   wholeProcessCpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime)
    return row


try:
    for repetition in range(a.rounds):
        order=a.modes if repetition%2==0 else list(reversed(a.modes))
        for sample in a.samples:
            for protocol in a.protocols:
                for mode in order:
                    print(repetition,sample,protocol,mode,flush=True)
                    row=trial(mode,sample,protocol,repetition);r['trials'].append(row)
                    (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
                    assert row['passed'],row.get('failure',row['stderr'])
    for sample in a.samples:
        for protocol in a.protocols:
            for mode in a.modes:
                trials=[t for t in r['trials'] if (t['sample'],t['protocol'],t['mode'])==(sample,protocol,mode)]
                row=dict(sample=sample,protocol=protocol,mode=mode,requestsPerSecond=statistics.median(t['requestsPerSecond'] for t in trials))
                for key in ['invocationCpuSeconds','allocations','allocatedBytes','peakLiveBytes','peakArenaBytes','wholeProcessCpuSeconds']:
                    if all(key in t for t in trials):row[key]=statistics.median(t[key] for t in trials)
                r['rows'].append(row)
    r['inputs'][str(Path(__file__).resolve())]=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    r['passed']=True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
