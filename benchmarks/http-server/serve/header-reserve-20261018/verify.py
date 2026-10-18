#!/usr/bin/env python3
"""Verify header reservation growth and exact configured limits in three modes."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
import time
ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'benchmarks/framework'))
from process import Guest
p = argparse.ArgumentParser(description=__doc__)
for name in ('build', 'library', 'runtime', 'backend', 'host', 'output'):
    p.add_argument('--'+name, type=Path, required=True)
a = p.parse_args()
b, lib = a.build.resolve(), a.library.resolve()
context = ['--system', str(lib/'System.runtime.neox'), '--object-root', str(lib/'System.Runtime.dll')]
for name in json.loads((lib/'bundle.json').read_text())['assemblyNames']:
    context += ['--module', str(lib/(name+'.dll'))]
flags = next(c['command'] for c in json.loads((b/'build.json').read_text())['commands'] if c['phase']=='native-compilation')
flags = flags[flags.index('--compile-system'):]
commands = {'aot':[str(b/'app')],
    'interpreter':[str(a.runtime.resolve()), 'run', str(b/'app.dll'), '--execution','interpreter','--instructions','2000000000',*context],
    'jit':[str(a.backend.resolve()),'--jit-run','--closed-world',str(b/'app.dll'),'@entry',str(a.host.resolve()),*context,*flags]}
report = dict(passed=False, commands=commands, cases={}, inputs={})
for path in [Path(__file__), Path(__file__).with_name('Probe.rvn'),b/'app', b/'app.dll',lib/'bundle.json']:
    report['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
try:
    for mode,command in commands.items():
        guest=Guest(command,timeout=180)
        rows=[]
        try:
            for limit in [1,127,128,129,256,512]:
                port=int(guest.line()[0])
                for attempt in range(2):
                    prefix=b'GET / HTTP/1.1\r\nHost: localhost\r\nX-Pad: '
                    size=max(limit+attempt,len(prefix)+4)
                    wire=prefix+b'x'*(size-len(prefix)-4)+b'\r\n\r\n'
                    with socket.create_connection(('127.0.0.1',port),timeout=120) as peer:
                        # Cross the reserved prefix in separate sends for larger headers.
                        peer.sendall(wire[:120])
                        if len(wire)>120:
                            time.sleep(.03)
                            peer.sendall(wire[120:])
                        response=b''
                        while True:
                            block=peer.recv(8192)
                            if not block:break
                            response+=block
                    expected=b'OK' if limit>1 and attempt==0 else b'LIMIT'
                    guest.line(expected)
                    if expected==b'OK':assert response.split(b'\r\n\r\n',1)[1]==b'accepted'
                    else:assert response==b''
                    rows.append(dict(limit=limit,bytes=len(wire),outcome=expected.decode(),passed=True))
            guest.line(b'DONE');guest.finish()
            report['cases'][mode]=rows
            print('PASS',mode,flush=True)
        except Exception as error:
            report["failure"]=dict(mode=mode,error=str(error),stderr=guest.errors().decode(errors="replace"),stdout=b"".join(guest.output).decode(errors="replace"))
            raise
        finally:guest.close()
    report['passed']=True
finally:a.output.write_text(json.dumps(report,indent=2)+'\n')
