#!/usr/bin/env python3
"""Relink the same compiled application for 1/2/4 isolated owners with paced GC."""
import argparse
import asyncio
import hashlib
import json
import math
from pathlib import Path
import queue
import re
import resource
import socket
import subprocess
import threading
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--relink-current-runtime', action='store_true', help='Allow changed runtime/native sources only; record original and current digests')
p.add_argument('--seconds', type=float, default=6)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
build = json.loads(a.build.read_text()); assert build['passed']
recipe = next(c['command'] for c in build['commands'] if c.get('phase') == 'host-compile-link')
report = dict(passed=False, build=str(a.build.resolve()), buildSha256=hashlib.sha256(a.build.read_bytes()).hexdigest(), commands=[], trials=[], inputs=build['inputs'], gc='explicit NEOCLR_GC_PACED; synchronous collector per worker', concurrency=32, windowSeconds=a.seconds, heapBytesPerWorker=build['httpHeapBytes'])
report['runtimeRelinkChanges'] = {}
runtime = Path(__file__).resolve().parents[3]/'runtime/native'
for path,digest in list(report['inputs'].items()):
    current = hashlib.sha256(Path(path).read_bytes()).hexdigest()
    if current != digest:
        assert a.relink_current_runtime and runtime in Path(path).parents, path
        report['runtimeRelinkChanges'][path] = dict(previous=digest, current=current)
        report['inputs'][path] = current
report['inputs'][str(Path(__file__).resolve())] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()

async def load(port, controls, seconds):
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
                assert int(body) in controls, body
                if time.monotonic() <= stop:
                    completed += 1
                    latencies.append((time.monotonic() - began) * 1000)
                else: drained += 1
                owners[body.decode()] += 1
            finally:
                writer.close(); await writer.wait_closed()
    await asyncio.gather(*(lane() for _ in range(32)))
    latencies.sort()
    def percentile(q):
        return latencies[min(len(latencies)-1, max(0, math.ceil(len(latencies)*q)-1))] if latencies else None
    return dict(latencyMilliseconds=dict(p50=percentile(.50), p95=percentile(.95), p99=percentile(.99)),
                latencyScope='Completed inside window, connection establishment through full body; closed-loop load',
                completedInWindow=completed, drainResponses=drained, requestsPerSecond=completed/seconds, ownerResponses=owners)

try:
    for workers in [1,2,4]:
        cmd = [s for s in recipe if not s.startswith('-DNEOCLR_HTTP_WORKERS=')]
        cmd += ['-DNEOCLR_HTTP_WORKERS='+str(workers), '-DNEOCLR_GC_PACED']
        cmd[cmd.index('-o')+1] = str(out/('app-'+str(workers)))
        r = subprocess.run(cmd,capture_output=True,text=True,timeout=60)
        report['commands'].append(dict(command=cmd,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        r.check_returncode()
    for workers in [1,2,4,4,2,1]:
        exe = out/('app-'+str(workers)); lines=[]; errors=[]; events=queue.Queue()
        before = resource.getrusage(resource.RUSAGE_CHILDREN)
        proc = subprocess.Popen([str(exe)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        def read(stream, result):
            for line in stream: result.append(line); events.put(line)
        threads=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
        for t in threads:t.start()
        try:
            ready=[]; until=time.monotonic()+30
            while len(ready)<workers and time.monotonic()<until:
                try: line=events.get(timeout=1)
                except queue.Empty:
                    if proc.poll() is not None:break
                    continue
                ready += [(int(x),int(y)) for x,y in re.findall(r'READY (\d+) (\d+)',line)]
            assert len(ready)==workers and len({x for x,y in ready})==1,(lines,errors)
            port=ready[0][0]; controls={y for x,y in ready}
            warmup=asyncio.run(load(port,controls,1))
            started=time.monotonic()
            sample=asyncio.run(load(port,controls,a.seconds))
            sample['windowAndDrainSeconds']=time.monotonic()-started
            for control in controls:
                with socket.create_connection(('127.0.0.1',control),timeout=5):pass
            proc.wait(timeout=20)
            for t in threads:t.join(timeout=2)
            after=resource.getrusage(resource.RUSAGE_CHILDREN)
            assert proc.returncode==0,(lines,errors)
            sample.update(workers=workers,executableSha256=hashlib.sha256(exe.read_bytes()).hexdigest(),processCpuSeconds=(after.ru_utime+after.ru_stime)-(before.ru_utime+before.ru_stime),cpuScope='entire process including startup, 1s warmup, measured window, drain and shutdown',stdout=lines,stderr=errors,warmup=warmup)
            assert all(sample['ownerResponses'].values()),sample
            report['trials'].append(sample)
            print(json.dumps({k:sample[k] for k in ['workers','requestsPerSecond','processCpuSeconds']}),flush=True)
        finally:
            if proc.poll() is None:proc.kill()
            proc.wait()
            for t in threads:t.join(timeout=2)
    report['passed']=True
finally:
    (out/'measurement.json').write_text(json.dumps(report,indent=2)+'\n')
