#!/usr/bin/env python3
"""Count transient publications and boundaries during eight forced-GC HTTP responses."""
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
p.add_argument('--build',type=Path,required=True)
p.add_argument('--object',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--json', action='store_true')
p.add_argument('--paced', action='store_true')
p.add_argument('--requests', type=int, default=4, choices=[4,12])
a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[4]
report=dict(passed=False,commands=[],responses=[],inputs={})
try:
    build=json.loads(a.build.read_text())
    recipe=next(c['command'] for c in build['commands'] if c.get('phase')=='host-compile-link')
    old=next(c['command'][4] for c in build['commands'] if c.get('phase')=='native-compilation')
    cmd=[str(a.object.resolve()) if s==old else s for s in recipe if not s.startswith('-DNEOCLR_HTTP_WORKERS=')]
    source = (root/'runtime/native/gc/root-probe.c').read_text()
    source = '#include <stdatomic.h>\n#include <stdio.h>\nstatic _Atomic unsigned long long handoffs, boundaries;\n' + source
    source = source.replace('    probe_state.calls++;', '    probe_state.calls++; atomic_fetch_add(&boundaries, 1);')
    start = source.index('void neoclr_probe_transient_v2(')
    body = source.index(' {', start) + 2
    source = source[:body] + '\n    atomic_fetch_add(&handoffs, 1);' + source[body:]
    source += '\n__attribute__((destructor)) static void counts(void) { fprintf(stderr, "COUNTS %llu %llu\\n", atomic_load(&handoffs), atomic_load(&boundaries)); }\n'
    diagnostic = out/'diagnostic-probe.c'
    diagnostic.write_text(source)
    cmd = [str(diagnostic) if x == str(root/'docs/experiments/aot-console/root-probe.c') else x for x in cmd]
    cmd += ['-I', str(root/'runtime/native/gc')]
    gc_source = '#include <stdatomic.h>\n#include <stdio.h>\nstatic _Atomic unsigned long long allocs, payload, collections;\n' + (root/'runtime/native/gc/native-gc.c').read_text()
    gc_source = gc_source.replace('statistics.allocations++;', 'statistics.allocations++; atomic_fetch_add(&allocs, 1); atomic_fetch_add(&payload, bytes);')
    gc_source = gc_source.replace('statistics.collections++;', 'statistics.collections++; atomic_fetch_add(&collections, 1);')
    gc_source += '\n__attribute__((destructor)) static void gc_counts(void) { fprintf(stderr, "GC %llu %llu %llu\\n", atomic_load(&allocs), atomic_load(&payload), atomic_load(&collections)); }\n'
    gc_diagnostic = out/'diagnostic-gc.c'
    gc_diagnostic.write_text(gc_source)
    cmd = [str(gc_diagnostic) if x == str(root/'docs/experiments/aot-console/native-gc.c') else x for x in cmd]
    if a.paced: cmd += ['-DNEOCLR_GC_PACED']

    cmd += ['-DNEOCLR_HTTP_WORKERS=1','-DNEOCLR_GC_OWNED_HEAP','-DNEOCLR_HTTP_INVOCATIONS=2','-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
    cmd[cmd.index('-o')+1]=str(out/'server')
    r=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
    report['commands'].append(dict(command=cmd,exitCode=r.returncode,stderr=r.stderr));r.check_returncode()
    inputs=[gc_diagnostic, diagnostic, a.build.resolve(),root/'benchmarks/native-web/http-host.c',a.object.resolve(),Path(__file__).resolve(),* (root/'runtime/native').rglob('*.[ch]')]
    for path in inputs:report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    proc=subprocess.Popen([out/'server'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    lines,errors,events=[],[],queue.Queue()
    def read(stream,target):
        for line in stream: target.append(line);events.put(line)
    threads=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
    for t in threads:t.start()
    try:
        for invocation in range(2):
            line=events.get(timeout=30)
            while not line.startswith('READY '): line=events.get(timeout=30)
            tag,port,control=line.split();assert tag=='READY'
            for i in range(a.requests):
                with socket.create_connection(('127.0.0.1',int(port)),timeout=30) as connection:
                    connection.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                    data=b''
                    while chunk:=connection.recv(4096): data+=chunk
                header,body=data.split(b'\r\n\r\n',1)
                assert header.startswith(b'HTTP/1.1 200'),data
                if a.json: assert json.loads(body)=={'id':42,'name':'Café team','active':True},data
                else: assert body==b'xxxxxxxxxx',data
                report['responses'].append(data.decode())
            with socket.create_connection(('127.0.0.1',int(control)),timeout=5):pass
        proc.wait(timeout=30)
        for t in threads:t.join(timeout=2)
        assert proc.returncode==0 and len(errors)==2,(lines,errors)
        _, handoffs, boundaries = next(x for x in errors if x.startswith('COUNTS ')).split()
        _, allocs, payload, collections = next(x for x in errors if x.startswith('GC ')).split()
        report.update(transientPublications=int(handoffs), boundaries=int(boundaries), allocations=int(allocs), payloadBytes=int(payload), collections=int(collections), paced=a.paced, json=a.json)
        report.update(stdout=lines,stderr=errors,exitCode=proc.returncode,passed=True)
    finally:
        if proc.poll() is None:proc.kill()
        proc.wait()
        for t in threads:t.join(timeout=2)
finally:
    (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
