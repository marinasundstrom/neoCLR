"""Paired bounded HTTP owner measurement; whole-process CPU includes warmup/cleanup."""
import argparse,asyncio,os,hashlib,json,math,queue,re,resource,socket,statistics,subprocess,threading,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--seconds',type=float,default=3);p.add_argument('--rounds',type=int,default=3);p.add_argument('--concurrency',type=int,default=32);p.add_argument('--go-client',type=Path);p.add_argument('--owners',type=int,nargs='+',choices=[1,4],default=[1,4]);p.add_argument('--modes',nargs='+',choices=['sync','live'],default=['sync','live']);a=p.parse_args()
assert a.seconds>0 and a.rounds>0 and a.concurrency>0
assert len(set(a.owners))==len(a.owners) and len(set(a.modes))==len(a.modes)
build=a.build.resolve();out=a.output.resolve();assert not out.exists()
manifest=json.loads((build/'results.json').read_text());assert manifest['passed']
sample=manifest.get('sample','task')
assert not a.go_client or (sample=='plain' and a.concurrency==32)
r={'passed':False,'sample':sample,'scope':'HTTP owner fixture, closed-loop connections. Whole-process CPU/allocations include startup/warmup/drain/shutdown.','seconds':a.seconds,'concurrency':a.concurrency,'trials':[],'inputs':{}}
programs={(mode,workers):build/f'{mode}-{workers}' for mode in a.modes for workers in a.owners}
for path in [Path(__file__),build/'results.json',*programs.values(),*([a.go_client] if a.go_client else [])]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
async def load(port, controls, seconds):
    if a.go_client:
        x=subprocess.run([str(a.go_client.resolve()),str(port),str(seconds),'Hello, World!'],capture_output=True,text=True,timeout=seconds+20)
        r.setdefault('clientRuns',[]).append(dict(command=x.args,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
        x.check_returncode();result=json.loads(x.stdout)
        result['completedInWindow']=result.pop('completed');result['drainResponses']=result.pop('drained')
        result['ownerResponses']={str(owner):0 for owner in controls}
        return result
    latencies = []
    stop = time.monotonic() + seconds
    owners = {str(x):0 for x in controls}; completed = drained = 0
    async def lane():
        nonlocal completed, drained
        while time.monotonic() < stop:
            began = time.monotonic()
            reader, writer = await asyncio.wait_for(asyncio.open_connection('127.0.0.1',port), 10)
            try:
                writer.write(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                await writer.drain()
                head = await asyncio.wait_for(reader.readuntil(b'\r\n\r\n'),10)
                assert head.startswith(b'HTTP/1.1 200'), head
                length = int(re.search(rb'content-length:\s*(\d+)',head,re.I)[1])
                body = await asyncio.wait_for(reader.readexactly(length),10)
                assert (body==b'Hello, World!' if sample=='plain' else int(body) in controls), body
                if time.monotonic() <= stop:
                    completed += 1
                    latencies.append((time.monotonic() - began) * 1000)
                else: drained += 1
                if sample=='task':owners[body.decode()] += 1
            finally:
                writer.close(); await writer.wait_closed()
    await asyncio.gather(*(lane() for _ in range(a.concurrency)))
    latencies.sort()
    def percentile(q):
        return latencies[min(len(latencies)-1, max(0, math.ceil(len(latencies)*q)-1))] if latencies else None
    return dict(latencyMilliseconds=dict(p50=percentile(.50), p95=percentile(.95), p99=percentile(.99)),
                latencyScope='Completed inside window, connection establishment through full body; closed-loop load',
                completedInWindow=completed, drainResponses=drained, requestsPerSecond=completed/seconds, ownerResponses=owners)

try:
 for repeat in range(a.rounds):
  order=list(programs);offset=repeat%len(order);order=order[offset:]+order[:offset]
  if repeat%2:order.reverse()
  for mode,workers in order:
   before=resource.getrusage(resource.RUSAGE_CHILDREN);client_before=resource.getrusage(resource.RUSAGE_SELF)
   proc=subprocess.Popen([str(programs[mode,workers])],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
   lines=[];errors=[];events=queue.Queue()
   def read(stream,result):
    for line in stream:result.append(line);events.put(line)
   readers=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
   for reader in readers:reader.start()
   trial=dict(mode=mode,workers=workers,repeat=repeat,stdout=lines,stderr=errors);r['trials'].append(trial)
   try:
    ready=[];deadline=time.monotonic()+30
    while len(ready)<workers and time.monotonic()<deadline:
     try:line=events.get(timeout=1)
     except queue.Empty:
      if proc.poll() is not None:break
      continue
     ready.extend((int(x),int(y)) for x,y in re.findall(r'READY (\d+) (\d+)',line))
    assert len(ready)==workers and len({x for x,y in ready})==1
    port=ready[0][0];controls={y for x,y in ready};assert len(controls)==workers
    warmup=asyncio.run(load(port,controls,1));trial['warmup']=warmup
    trial.update(asyncio.run(load(port,controls,a.seconds)))
    for control in controls:
     with socket.create_connection(('127.0.0.1',control),timeout=5):pass
    deadline=time.monotonic()+30
    while True:
     pid,status,server_usage=os.wait4(proc.pid,os.WNOHANG)
     if pid:
      proc.returncode=os.waitstatus_to_exitcode(status);break
     if time.monotonic()>deadline:raise TimeoutError('server shutdown')
     time.sleep(.01)
    for reader in readers:reader.join(timeout=2)
    assert proc.returncode==0
    if sample=='task':assert all(trial['ownerResponses'].values())
    after=resource.getrusage(resource.RUSAGE_CHILDREN);client_after=resource.getrusage(resource.RUSAGE_SELF)
    cpu=server_usage.ru_utime+server_usage.ru_stime
    total=sum(x['completedInWindow']+x['drainResponses'] for x in [warmup,trial])
    done=re.findall(r'DONE (\d+) (\d+) (\d+)', ''.join(lines))
    assert all(int(row[1])>0 for row in done)
    assert len(done)==workers and {int(row[0]) for row in done}==controls and sum(int(row[1]) for row in done)==total
    stderr=''.join(errors);allocations=list(map(int,re.findall(r'^ALLOCATIONS (\d+)$',stderr,re.M)))
    assert len(allocations)==workers
    process=[line.split() for line in re.findall(r'^PROCESS (.+)$',stderr,re.M)];assert len(process)==workers
    profiles=[list(map(int,line.split())) for line in re.findall(r'^PROFILE ([\d ]+)$',stderr,re.M)];assert len(profiles)==workers and all(len(row)==7 for row in profiles)
    trial.update(loadClient='go' if a.go_client else 'python',totalResponses=total,processCpuSeconds=cpu,cpuMicrosecondsPerResponse=cpu/total*1e6,allocations=sum(allocations),allocationsPerResponse=sum(allocations)/total,maxRssBytes=max(int(row[2]) for row in process),profiles=profiles,clientCpuSeconds=client_after.ru_utime+client_after.ru_stime-client_before.ru_utime-client_before.ru_stime)
    if a.go_client:trial['clientCpuSeconds']=warmup['clientProcessCpuSeconds']+trial['clientProcessCpuSeconds']
    if mode=='live':
     counts=[list(map(int,line.split())) for line in re.findall(r'^CANDIDATE ([\d ]+)$',stderr,re.M)]
     indices=[list(map(int,line.split())) for line in re.findall(r'^INDEX_METADATA ([\d ]+)$',stderr,re.M)]
     assert len(counts)==len(indices)==workers and all(len(c)==6 and c[0]==c[1]+c[2] and c[1]>0 and c[5]==0 for c in counts)
     assert all(len(i)==3 and i[0]>0 and i[1]==0 for i in indices)
     trial.update(candidateCounts=counts,indexMetadata=indices)
    print(json.dumps({k:trial[k] for k in ['mode','workers','repeat','requestsPerSecond','cpuMicrosecondsPerResponse','allocationsPerResponse']}),flush=True)
   finally:
    if proc.poll() is None:proc.kill()
    proc.wait()
    for reader in readers:reader.join(timeout=2)
 metrics=['requestsPerSecond','cpuMicrosecondsPerResponse','allocationsPerResponse','maxRssBytes','clientCpuSeconds']
 r['medians']={mode+'-'+str(workers):{metric:statistics.median(t[metric] for t in r['trials'] if t['mode']==mode and t['workers']==workers) for metric in metrics} for mode,workers in programs}
 r['passed']=True
finally:out.write_text(json.dumps(r,indent=2)+'\n')
