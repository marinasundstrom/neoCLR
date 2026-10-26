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
import shutil
import statistics
import subprocess
import threading
import time
import traceback
from process_memory import snapshot as memory_snapshot

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
p.add_argument('--sample-stacks',action='store_true',help='macOS one-second stack capture after warmup; diagnostic timings only')
p.add_argument('--cpu-trace',action='store_true',help='macOS five-second CPU Profiler trace after warmup; diagnostic only')
p.add_argument('--header-gap-ms',type=float,default=0,help='Split request line/headers with client delay to exercise pending server reads; included in latency')
p.add_argument('--io-gap-ms',type=float,default=0,help='Client wait between request batches, simulating I/O slack; included in throughput wall time')
p.add_argument('--process-memory',action='store_true',help='macOS whole-process RSS/virtual snapshots; diagnostic timings only')
a=p.parse_args()
assert a.io_gap_ms>=0 and a.header_gap_ms>=0
if a.cpu_trace:
    assert platform.system()=='Darwin' and shutil.which('xcrun')
    assert not a.sample_stacks and a.requests>=65536 and 'interpreter' not in a.modes
if a.process_memory:assert platform.system()=='Darwin', 'macOS ps memory semantics required'
if a.sample_stacks:
    assert platform.system()=='Darwin' and shutil.which('sample'), 'macOS sample is required'
    assert a.requests>=32768 and 'interpreter' not in a.modes, 'use at least 32768 native requests for sampling'
assert a.rounds>0 and a.requests>0 and a.interpreter_requests>0 and a.concurrency>0
assert a.requests%a.concurrency==0 and a.interpreter_requests%a.concurrency==0
assert a.pipeline_depth>0 and a.warmup>0
root=Path(__file__).resolve().parents[3]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
build_path=a.build.resolve()/'build.json';built=json.loads(build_path.read_text())
assert built['passed'] and all(mode in built['recipes'] for mode in a.modes)
for path,digest in built['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build input',path)
r=dict(passed=False,kind='profile' if a.process_memory or a.sample_stacks or a.cpu_trace or built.get('diagnosticOnly') else 'qualification' if a.smoke else 'benchmark',scope=__doc__,system=platform.platform(),
       workload=built.get('workload','standard-http'),backendOptimization=built.get('backendOptimization','explicit retained recipe'),rounds=a.rounds,concurrency=a.concurrency,ioGapMs=a.io_gap_ms,headerGapMs=a.header_gap_ms,pipelineDepth=a.pipeline_depth,trials=[],rows=[],
       inputs={str(build_path):hashlib.sha256(build_path.read_bytes()).hexdigest(),str(Path(__file__).resolve()):hashlib.sha256(Path(__file__).read_bytes()).hexdigest()})
if built.get('diagnosticOnly'):r['diagnosticOnly']=built['diagnosticOnly']
if a.sample_stacks:
    r['diagnosticOnly']='Wall-clock stack samples include sleeping threads, not CPU percentages. Sampling perturbs timing. wholeProcessCpuSeconds includes the sampler child; invocationCpuSeconds remains server-reported.'

if a.cpu_trace:
    r['diagnosticOnly']='CPU Profiler recording perturbs timing. Invocation CPU includes all server threads; tracing is a separate diagnostic cohort.'

if a.process_memory:
    r['diagnosticOnly']=r.get('diagnosticOnly','')+' Whole-process RSS/virtual snapshots include compiler, metadata and libraries; not allocator attribution or peaks. Child CPU includes ps; timings are diagnostic.'
    source=Path(__file__).with_name('process_memory.py').resolve()
    r['inputs'][str(source)]=hashlib.sha256(source.read_bytes()).hexdigest()


def traffic(port,sample,protocol,count,depth_override=None):
    expected=b'Hello, World!' if sample=='plain' else {'id':42,'name':'Café team','active':True}
    depth=depth_override or (a.pipeline_depth if protocol=='pipeline' else 1)
    client=None
    pending=b''
    latencies=[]
    try:
        for start in range(0,count,depth):
            if start and a.io_gap_ms:time.sleep(a.io_gap_ms/1000)
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
            if a.header_gap_ms:
                split=request.index(b'\r\n')+2
                client.sendall(request[:split])
                time.sleep(a.header_gap_ms/1000)
                client.sendall(request[split:])
            else:
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
    profiler=None
    env=dict(os.environ)
    for flag in ['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS','NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS',
                 'NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS','NEOCLR_LIVE_SSA_BYREF_ARGUMENTS',
                 'NEOCLR_LIVE_SCALAR_ARRAY_STORES','NEOCLR_LIVE_GENERIC_SCALAR_ARRAY_STORES','NEOCLR_LIVE_STACK_POLICY',
                 'NEOCLR_JIT_LAZY','NEOCLR_TEST_LAZY_BIND_COMPILED','NEOCLR_TEST_LAZY_PANIC_AFTER','NEOCLR_TEST_LAZY_FAIL_AFTER','NEOCLR_TEST_JIT_ARENA','NEOCLR_TEST_KEEP_JIT_COMPILATION','NEOCLR_TEST_NATIVE_MEMORY']:
        env.pop(flag,None)
    env.update(recipe['env'])
    if a.process_memory and recipe['execution']=='jit':env['NEOCLR_TEST_NATIVE_MEMORY']='1'
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
        if a.process_memory:row['processMemory']={'ready':memory_snapshot(process.pid)}
        cold_started=time.perf_counter()
        cold=traffic(port,sample,protocol,a.warmup,depth_override=1)
        row['coldWarmupSeconds']=time.perf_counter()-cold_started
        row['coldRequestLatenciesSeconds']=cold
        row['firstResponseSeconds']=cold[0]
        # Observed nearest-rank p99 over this small cold sample, not a population estimate.
        row['coldObservedP99Seconds']=sorted(cold)[max(0,(99*len(cold)+99)//100-1)]
        row['readyThroughFirstResponseSeconds']=row['readySeconds']+row['firstResponseSeconds']
        if a.process_memory:row['processMemory']['afterWarmup']=memory_snapshot(process.pid)
        if a.sample_stacks:
            profile_path=out/f'{sample}-{protocol}-{mode}-{repetition}-stacks.txt'
            command=['sample',str(process.pid),'1','1','-file',str(profile_path)]
            row['stackProfile']=dict(command=command,path=str(profile_path))
            profiler=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        if a.cpu_trace:
            profile_path=out/f'{sample}-{protocol}-{mode}-{repetition}.trace'
            command=['xcrun','xctrace','record','--template','CPU Profiler','--attach',str(process.pid),'--time-limit','5s','--output',str(profile_path)]
            row['cpuTrace']=dict(command=command,path=str(profile_path))
            profiler=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        started=time.perf_counter()
        with ThreadPoolExecutor(max_workers=a.concurrency) as pool:
            futures=[pool.submit(traffic,port,sample,protocol,count//a.concurrency) for _ in range(a.concurrency)]
            latencies=[latency for future in futures for latency in future.result()]
        duration=time.perf_counter()-started
        assert len(latencies)==count
        if a.process_memory:row['processMemory']['afterTraffic']=memory_snapshot(process.pid)
        if profiler is not None:
            profile_stdout,profile_stderr=profiler.communicate(timeout=30)
            profile=row['cpuTrace'] if a.cpu_trace else row['stackProfile']
            profile.update(exitCode=profiler.returncode,stdout=profile_stdout,stderr=profile_stderr)
            assert profiler.returncode==0 and profile_path.exists(), profile
            if a.sample_stacks:
                assert 'Call graph:' in profile_path.read_text(), 'missing stack capture'
                profile['sha256']=hashlib.sha256(profile_path.read_bytes()).hexdigest()
            else:
                profile['files']={str(p.relative_to(profile_path)):hashlib.sha256(p.read_bytes()).hexdigest() for p in profile_path.rglob('*') if p.is_file()}
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
            if a.process_memory:
                assert all(stage in row['jit']['nativeMemory'] for stage in ['beforePrepare','beforeRelease','afterRelease','afterInvoke'])
            if 'compilationStateReleased' in recipe:
                assert row['jit']['compilationStateReleased']==recipe['compilationStateReleased']
        row['passed']=True
    except Exception:
        row['failure']=traceback.format_exc()
    finally:
        if profiler is not None and profiler.poll() is None:
            profiler.kill();profiler.communicate(timeout=10)
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
                for key in ['readySeconds','firstResponseSeconds','readyThroughFirstResponseSeconds','coldObservedP99Seconds','invocationCpuSeconds','allocations','allocatedBytes','peakLiveBytes','peakArenaBytes','wholeProcessCpuSeconds']:
                    if all(key in t for t in trials):row[key]=statistics.median(t[key] for t in trials)
                r['rows'].append(row)
    r['inputs'][str(Path(__file__).resolve())]=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    r['passed']=True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
