#!/usr/bin/env python3
"""Exercise scratch reuse, retained requests, receive failures and cancellation in three modes."""
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
def connect(port, body, fragmented=False):
    peer=socket.create_connection(('127.0.0.1',port),timeout=120)
    head=b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: '+str(len(body)).encode()+b'\r\n\r\n'
    if fragmented:
        peer.sendall(head+body[:100]);time.sleep(.03);peer.sendall(body[100:])
    else: peer.sendall(head+body)
    return peer
def read(peer):
    wire=b''
    while True:
        block=peer.recv(8192)
        if not block:return wire
        wire+=block
        assert len(wire)<20000
try:
    for mode,command in commands.items():
        guest=Guest(command,timeout=180)
        first=None
        try:
            port=int(guest.line()[0]);first=connect(port,b'A'*4097,True);guest.line(b'HOLD')
            for index in range(8):
                guest.line(b'READY')
                if index in (2,3,4):
                    with socket.create_connection(('127.0.0.1',port),timeout=120) as peer:
                        peer.sendall(b'GET / HTTP/1.1\r\n\r\n' if index==2 else b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 900\r\n\r\nx')
                        if index==3:peer.shutdown(socket.SHUT_WR)
                        assert read(peer)==b''
                else:
                    size=[1,4097,0,0,0,7,5000,2][index]
                    body=bytes([66+index])*size
                    with connect(port,body,size>256) as peer:
                        wire=read(peer);assert wire.split(b'\r\n\r\n',1)[1]==body,(mode,index)
                guest.line(b'CASE')
            assert read(first).split(b'\r\n\r\n',1)[1]==b'A'*4097
            guest.line(b'DONE');guest.finish()
            report['cases'][mode]=dict(passed=True, cases=9)
            print('PASS',mode,flush=True)
        except Exception as error:
            report['failure'] = dict(mode=mode, error=str(error), stderr=guest.errors().decode(errors='replace'), stdout=b''.join(guest.output).decode(errors='replace'))
            raise
        finally:
            if first:first.close()
            guest.close()
    report['passed']=True
finally:a.output.write_text(json.dumps(report,indent=2)+'\n')
