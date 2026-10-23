#!/usr/bin/env python3
"""Separate main-owner GC diagnostics; never treat instrumented runs as capacity."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--client',type=Path,required=True)
p.add_argument('--object',type=Path,required=True)
p.add_argument('--experiment',type=Path,required=True)
a=p.parse_args()
root=Path(__file__).resolve().parents[4]
out=a.experiment.resolve()
report=dict(passed=False,commands=[],trials=[],inputs={},scope='Main-owner diagnostic only; GC statistics are thread-local')
def run(cmd):
    r=subprocess.run(list(map(str,cmd)),capture_output=True,text=True,timeout=120)
    report['commands'].append(dict(command=list(map(str,cmd)),exitCode=r.returncode,stderr=r.stderr))
    r.check_returncode()
try:
    build=json.loads(a.build.read_text())
    report['nativeSourceChanges']={}
    for name,digest in build['inputs'].items():
        actual=hashlib.sha256(Path(name).read_bytes()).hexdigest()
        if actual != digest:
            assert Path(name).resolve() in [root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',root/'target/release/neoclr-codegen'],name
            report['nativeSourceChanges'][name]=dict(original=digest,current=actual)
        report['inputs'][name]=actual
    recipe=next(c['command'] for c in build['commands'] if c.get('phase')=='host-compile-link')
    original=next(c['command'][4] for c in build['commands'] if c.get('phase')=='native-compilation')
    recipe=[str(a.object.resolve()) if s==original else s for s in recipe]
    report['inputs'][str(a.object.resolve())]=hashlib.sha256(a.object.read_bytes()).hexdigest()
    for allocations in ['baseline','candidate']:
        variant=out/f'{allocations}-native-gc.c'
        binary=out/f'profile-{allocations}'
        cmd=[str(variant) if s.endswith('/aot-console/native-gc.c') else s for s in recipe if not s.startswith('-DNEOCLR_HTTP_WORKERS=')]
        cmd+=['-DNEOCLR_GC_PACED','-DNEOCLR_GC_PROFILE','-I',str(root/'runtime/native/gc'),str(root/'benchmarks/http-server/serve/profile-native.c')]
        cmd[cmd.index('-o')+1]=str(binary)
        run(cmd)
        for path in [variant,binary,Path(__file__).resolve(),a.client.resolve(),root/'benchmarks/http-server/serve/profile-native.c']:
            report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    for allocations in ['baseline','candidate']:
        stats=out/f'profile-{allocations}.json'
        proc=subprocess.Popen([out/f'profile-{allocations}'],env=dict(os.environ,NEOCLR_HTTP_CAPACITY_PROFILE=str(stats)),stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        try:
            line=proc.stdout.readline()
            tag,port,control=line.split()
            assert tag=='READY',line
            client=subprocess.run([a.client.resolve(),port,'4'],env=dict(os.environ,GOMAXPROCS='2'),capture_output=True,text=True,timeout=30)
            assert client.returncode==0,client.stderr
            with socket.create_connection(('127.0.0.1',int(control)),timeout=5): pass
            stdout,stderr=proc.communicate(timeout=20)
            assert proc.returncode==0,(stdout,stderr)
            trial=dict(allocations=allocations,load=json.loads(client.stdout),gc=json.loads(stats.read_text()),stdout=line+stdout,stderr=stderr)
            report['trials'].append(trial)
            print(json.dumps(trial),flush=True)
        finally:
            if proc.poll() is None: proc.kill()
            proc.wait()
    report['passed']=True
finally:
    (out/'profiles.json').write_text(json.dumps(report,indent=2)+'\n')
