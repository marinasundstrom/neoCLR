#!/usr/bin/env python3
"""Completed HTTP owner snapshots; compare disabled/basic/timed diagnostics."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import resource
import queue
import re
import threading
import time
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--object', type=Path, required=True)
p.add_argument('--client', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--mode', choices=['off', 'basic', 'timed'], default='basic')
p.add_argument('--workers', type=int, choices=[1,4], default=4)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={}, scope='Joined isolated HTTP owner intervals, including startup/shutdown; no coordinator/pool or process totals.')
def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
try:
    build = json.loads(a.build.read_text())
    assert build['passed']
    recipe = next(c['command'] for c in build['commands'] if c.get('phase') == 'host-compile-link')
    original = next(c['command'][4] for c in build['commands'] if c.get('phase') == 'native-compilation')
    binary, stats = out/'server', out/'gc.json'
    cmd = [str(a.object.resolve()) if s == original else s for s in recipe if not s.startswith('-DNEOCLR_HTTP_WORKERS=')]
    cmd += ['-DNEOCLR_GC_PACED', '-DNEOCLR_GC_OWNED_HEAP', f'-DNEOCLR_HTTP_WORKERS={a.workers}']
    if a.mode != 'off':
        cmd += ['-DNEOCLR_RUNTIME_SNAPSHOT', '-I', str(root/'runtime/native/diagnostics'), str(root/'runtime/native/diagnostics/snapshot.c'), str(Path(__file__).with_name('report.c'))]
    if a.mode == 'timed': cmd += ['-DNEOCLR_GC_PROFILE']
    cmd[cmd.index('-o') + 1] = str(binary)
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
    result.check_returncode()
    for path in [a.build, a.object, a.client, __file__, binary, root/'benchmarks/native-web/http-host.c', Path(__file__).with_name('report.c'), *(root/'runtime/native').rglob('*.[ch]')]: record(path)
    cpu_before = resource.getrusage(resource.RUSAGE_CHILDREN)
    proc = subprocess.Popen([binary], env=dict(os.environ, NEOCLR_HTTP_RUNTIME_SNAPSHOT=str(stats)), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    lines, errors, events = [], [], queue.Queue()
    def read(stream, target):
        for line in stream:
            target.append(line)
            events.put(line)
    threads = [threading.Thread(target=read,args=(proc.stdout,lines)), threading.Thread(target=read,args=(proc.stderr,errors))]
    for thread in threads: thread.start()
    try:
        ready = []
        until = time.monotonic()+30
        while len(ready)<a.workers and time.monotonic()<until:
            try: line = events.get(timeout=1)
            except queue.Empty:
                if proc.poll() is not None: break
                continue
            ready += [(int(x),int(y)) for x,y in re.findall(r'READY (\d+) (\d+)',line)]
        assert len(ready)==a.workers and len({x for x,y in ready})==1, (lines,errors)
        port = str(ready[0][0])
        cmd = [str(a.client.resolve()), port, '4']
        result = subprocess.run(cmd, env=dict(os.environ, GOMAXPROCS='2'), capture_output=True, text=True, timeout=30)
        report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
        result.check_returncode()
        for _, control in ready:
            with socket.create_connection(('127.0.0.1', control), timeout=5): pass
        proc.wait(timeout=20)
        for thread in threads: thread.join(timeout=2)
        stdout, stderr = ''.join(lines), ''.join(errors)
        assert proc.returncode == 0, (stdout, stderr)
        report.update(load=json.loads(result.stdout), stdout=stdout, stderr=stderr, mode=a.mode, workers=a.workers)
        cpu_after = resource.getrusage(resource.RUSAGE_CHILDREN)
        report['serverCpuSeconds'] = cpu_after.ru_utime+cpu_after.ru_stime-cpu_before.ru_utime-cpu_before.ru_stime-report['load']['clientProcessCpuSeconds']
        report['cpuMicrosecondsPerResponse'] = report['serverCpuSeconds']*1e6/(report['load']['completed']+report['load']['drained'])
        if a.mode != 'off':
            report['snapshot'] = json.loads(stats.read_text())
            snapshot = report['snapshot']
            assert snapshot['version'] == 1 and len(snapshot['workers']) == a.workers
            for key, value in snapshot['total'].items():
                values = [w[key] for w in snapshot['workers']]
                assert (all(v is None for v in values) if value is None else sum(values) == value)
            assert snapshot['total']['allocations'] > 0
            assert snapshot['total']['asyncResumes'] > 0
            assert snapshot['total']['stackMappings'] == snapshot['total']['stackUnmappings']
            assert snapshot['total']['gcFailures'] == (0 if a.mode == 'timed' else None)
            count = report['load']['completed'] + report['load']['drained']
            report['perResponse'] = {k: None if v is None else v/count for k,v in snapshot['total'].items()}
        report['passed'] = True
        print(json.dumps(dict(mode=a.mode, load=report['load'], perResponse=report.get('perResponse'))))
    finally:
        if proc.poll() is None: proc.kill()
        proc.wait()
        for thread in threads: thread.join(timeout=2)
finally:
    (out/'profile.json').write_text(json.dumps(report, indent=2)+'\n')
