#!/usr/bin/env python3
"""Same-host minimal HTTP/1.1 connection-close comparison; not maximum-capacity certification."""
import argparse
import asyncio
import hashlib
import json
import math
import os
from pathlib import Path
import queue
import re
import resource
import socket
import subprocess
import threading
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--neoclr-build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--client-cpus', type=int, default=2)
p.add_argument('--client', choices=['go','python'], default='go')
p.add_argument('--seconds', type=float, default=6)
a = p.parse_args()
assert a.seconds > 0 and a.client_cpus > 0
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
report = dict(client=a.client, clientProcessors=a.client_cpus, passed=False, commands=[], trials=[], inputs={}, concurrency=32, warmupSeconds=2,
              windowSeconds=a.seconds, contract='GET /, HTTP/1.1, connection close, ten x bytes, no extra Task.Run/goroutine hop in handlers. Same client; runtime defaults differ. Closed-loop observations, not maximum capacities.')
def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
def run(command, env=None):
    r = subprocess.run(list(map(str, command)), env=env, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=list(map(str, command)), exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    return r.stdout.strip()
async def load(port, seconds):
    stop = time.monotonic() + seconds
    latencies = []
    drained = 0
    headers = None
    async def lane():
        nonlocal drained, headers
        while time.monotonic() < stop:
            began = time.monotonic()
            reader, writer = await asyncio.wait_for(asyncio.open_connection('127.0.0.1', port), 10)
            try:
                writer.write(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                await writer.drain()
                head = await asyncio.wait_for(reader.readuntil(b'\r\n\r\n'), 10)
                assert head.startswith(b'HTTP/1.1 200'), head
                length = int(re.search(rb'content-length:\s*(\d+)', head, re.I)[1])
                assert length == 10, head
                body = await asyncio.wait_for(reader.readexactly(length), 10)
                assert body == b'xxxxxxxxxx', body
                assert await asyncio.wait_for(reader.read(1), 10) == b''
                finished = time.monotonic()
                if finished <= stop: latencies.append((finished-began)*1000)
                else: drained += 1
                if headers is None: headers = head.decode()
            finally:
                writer.close()
                await writer.wait_closed()
    cpu = time.process_time()
    began = time.monotonic()
    await asyncio.gather(*(lane() for _ in range(32)))
    cpu = time.process_time()-cpu
    elapsed = time.monotonic()-began
    latencies.sort()
    assert latencies
    percentiles = {name:latencies[math.ceil(len(latencies)*q)-1] for name,q in [('p50',.50),('p95',.95),('p99',.99)]}
    return dict(completed=len(latencies), drained=drained, requestsPerSecond=len(latencies)/seconds,
                latencyMilliseconds=percentiles, clientCpuSeconds=cpu, windowAndDrainSeconds=elapsed,
                clientCpuPerWall=cpu/elapsed, exampleHeaders=headers)

try:
    build = json.loads(a.neoclr_build.read_text())
    assert build['passed']
    for name, digest in build['inputs'].items():
        assert hashlib.sha256(Path(name).read_bytes()).hexdigest() == digest, name
        record(name)
    for source in [*here.glob('*.rvn*'), here/'server.go', here/'client.go', here/'server.mjs', * (here/'kestrel').glob('*.cs*'), Path(__file__)]:
        record(source)
    record(a.neoclr_build)
    report['versions'] = dict(go=run(['go','version']), node=run(['node','--version']), dotnet=run(['dotnet','--list-runtimes']))
    env = dict(os.environ, GOTOOLCHAIN='local', CGO_ENABLED='0')
    run(['go','build','-trimpath','-o',out/'go-server',here/'server.go'],env)
    run(['go','build','-trimpath','-o',out/'client',here/'client.go'],env)
    run(['dotnet','build',here/'kestrel/Server.csproj','-c','Release','-o',out/'kestrel','--nologo'])
    recipe = next(c['command'] for c in build['commands'] if c.get('phase') == 'host-compile-link')
    for owners in [1,4]:
        cmd = [s for s in recipe if not s.startswith('-DNEOCLR_HTTP_WORKERS=')]
        cmd += [f'-DNEOCLR_HTTP_WORKERS={owners}','-DNEOCLR_GC_PACED']
        cmd[cmd.index('-o')+1] = str(out/f'neoclr-{owners}')
        run(cmd)
    for binary in [out/'go-server',out/'client',out/'kestrel/Server.dll',out/'neoclr-1',out/'neoclr-4']:
        record(binary)
    configs = [
        ('neoclr-1',[str(out/'neoclr-1')],1,{}),
        ('go-1',[str(out/'go-server')],1,dict(GOMAXPROCS='1',GOGC='100')),
        ('node-1',['node',str(here/'server.mjs')],1,{}),
        ('kestrel-1',['dotnet',str(out/'kestrel/Server.dll')],1,dict(DOTNET_PROCESSOR_COUNT='1')),
        ('neoclr-4',[str(out/'neoclr-4')],4,{}),
        ('go-4',[str(out/'go-server')],1,dict(GOMAXPROCS='4',GOGC='100')),
        ('kestrel-4',['dotnet',str(out/'kestrel/Server.dll')],1,dict(DOTNET_PROCESSOR_COUNT='4')),
    ]
    # No builds or other benchmarks run during these serial trials.
    for name, command, ready_count, overrides in configs + configs[::-1]:
        env = dict(os.environ)
        for key in ['GOMAXPROCS','GOGC','GOMEMLIMIT','DOTNET_PROCESSOR_COUNT']:
            env.pop(key,None)
        env.update(overrides)
        before = resource.getrusage(resource.RUSAGE_CHILDREN)
        proc = subprocess.Popen(command, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        lines, errors, events = [], [], queue.Queue()
        def read(stream, target):
            for line in stream: target.append(line); events.put(line)
        threads = [threading.Thread(target=read,args=(proc.stdout,lines)), threading.Thread(target=read,args=(proc.stderr,errors))]
        for t in threads: t.start()
        try:
            ready = []
            until = time.monotonic()+30
            while len(ready)<ready_count and time.monotonic()<until:
                try: line = events.get(timeout=1)
                except queue.Empty:
                    if proc.poll() is not None: break
                    continue
                ready += [(int(x),int(y)) for x,y in re.findall(r'READY (\d+) (\d+)',line)]
            assert len(ready)==ready_count and len({x for x,y in ready})==1, (lines,errors)
            port = ready[0][0]
            def drive(seconds):
                if a.client == 'python': return asyncio.run(load(port,seconds))
                return json.loads(run([out/'client',str(port),str(seconds)],dict(os.environ,GOMAXPROCS=str(a.client_cpus))))
            warmup = drive(2)
            trial = drive(a.seconds)
            for _, control in ready:
                with socket.create_connection(('127.0.0.1',control),timeout=5): pass
            proc.wait(timeout=20)
            for t in threads: t.join(timeout=2)
            assert proc.returncode == 0, (lines,errors)
            after = resource.getrusage(resource.RUSAGE_CHILDREN)
            trial.update(name=name,command=command,environment=overrides,warmup=warmup,stdout=lines,stderr=errors,
                         processCpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime
                         - warmup.get('clientProcessCpuSeconds',0) - trial.get('clientProcessCpuSeconds',0),
                         cpuScope='Server lifetime including startup, warmup, measurement, drain and shutdown')
            report['trials'].append(trial)
            print(json.dumps({k:trial[k] for k in ['name','requestsPerSecond','latencyMilliseconds','clientCpuPerWall']}),flush=True)
        finally:
            if proc.poll() is None: proc.kill()
            proc.wait()
            for t in threads: t.join(timeout=2)
    report['passed'] = True
finally:
    (out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
