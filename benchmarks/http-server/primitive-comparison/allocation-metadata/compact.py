#!/usr/bin/env python3
"""Compare compact native GC point metadata using the same source assembly."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import resource
import socket
import subprocess
import threading

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--client', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[4]
report = dict(passed=False, commands=[], inputs={}, trials=[], seconds=6, warmupSeconds=2,
              concurrency=32, clientProcessors=2, scope='Compact GC point payload comparison; same source assembly and native runtime')
def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
def run(cmd):
    r = subprocess.run(list(map(str, cmd)), capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=list(map(str, cmd)), exitCode=r.returncode, stderr=r.stderr))
    r.check_returncode()
    return r.stdout
try:
    build = json.loads(a.build.read_text())
    assert build['passed']
    report['nativeSourceChanges'] = {}
    for name, digest in build['inputs'].items():
        actual = hashlib.sha256(Path(name).read_bytes()).hexdigest()
        if actual != digest:
            assert Path(name).resolve() in [root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',root/'runtime/codegen/src/gc_probe.rs',root/'target/release/neoclr-codegen'], name
            report['nativeSourceChanges'][name] = dict(original=digest, current=actual)
        record(name)
    for path in [a.build, a.client, __file__]: record(path)
    recipe = next(c['command'] for c in build['commands'] if c.get('phase') == 'host-compile-link')
    native = next(c['command'] for c in build['commands'] if c.get('phase') == 'native-compilation')[:]
    original_object = Path(native[4])
    candidate_object = out/'candidate.o'
    native[4] = str(candidate_object)
    compilation = run(native)
    report['nativeCompilationStdoutBytes'] = len(compilation.encode())
    report['nativeCompilationStdoutSha256'] = hashlib.sha256(compilation.encode()).hexdigest()
    for path in [original_object, candidate_object, root/'runtime/codegen/src/gc_probe.rs']: record(path)
    report['objectBytes'] = dict(baseline=original_object.stat().st_size,candidate=candidate_object.stat().st_size)
    configs = []
    for label, obj in [('baseline',original_object),('candidate',candidate_object)]:
        for owners in [1,4]:
            name = f'{label}-w{owners}'
            cmd = [str(obj) if s == str(original_object) else s for s in recipe if not s.startswith('-DNEOCLR_HTTP_WORKERS=')]
            cmd += [f'-DNEOCLR_HTTP_WORKERS={owners}', '-DNEOCLR_GC_PACED']
            cmd[cmd.index('-o')+1] = str(out/name)
            run(cmd)
            record(out/name)
            configs.append((name, owners))
    for name, owners in configs + configs[::-1]:
        before = resource.getrusage(resource.RUSAGE_CHILDREN)
        proc = subprocess.Popen([out/name], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        lines, errors, events = [], [], queue.Queue()
        def read(stream, target):
            for line in stream:
                target.append(line)
                events.put(line)
        threads = [threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
        for thread in threads: thread.start()
        try:
            ready = []
            while len(ready) < owners:
                line = events.get(timeout=30)
                ready += [(int(x),int(y)) for x,y in re.findall(r'READY (\d+) (\d+)',line)]
            assert len({x for x,y in ready}) == 1
            def drive(seconds):
                r = subprocess.run([a.client.resolve(),str(ready[0][0]),str(seconds)],env=dict(os.environ,GOMAXPROCS='2'),capture_output=True,text=True,timeout=30)
                assert r.returncode == 0, r.stderr
                return json.loads(r.stdout)
            warmup, trial = drive(2), drive(6)
            for _, control in ready:
                with socket.create_connection(('127.0.0.1',control),timeout=5): pass
            proc.wait(timeout=20)
            for thread in threads: thread.join(timeout=2)
            assert proc.returncode == 0, (lines,errors)
            after = resource.getrusage(resource.RUSAGE_CHILDREN)
            cpu = after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime-warmup['clientProcessCpuSeconds']-trial['clientProcessCpuSeconds']
            count = sum(t['completed']+t['drained'] for t in [warmup,trial])
            trial.update(name=name,warmup=warmup,processCpuSeconds=cpu,cpuMicrosecondsPerResponse=cpu*1e6/count,stdout=lines,stderr=errors)
            report['trials'].append(trial)
            print(json.dumps({k:trial[k] for k in ['name','requestsPerSecond','cpuMicrosecondsPerResponse','latencyMilliseconds']}),flush=True)
        finally:
            if proc.poll() is None: proc.kill()
            proc.wait()
            for thread in threads: thread.join(timeout=2)
    report['passed'] = True
finally:
    (out/'measurement.json').write_text(json.dumps(report,indent=2)+'\n')
