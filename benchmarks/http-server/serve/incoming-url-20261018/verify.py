#!/usr/bin/env python3
"""Compare wire admission, exact errors, request fields and responses across modes/cohorts."""
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
p.add_argument('--oracle', type=Path)
p.add_argument('--modes', nargs='+', default=['aot','interpreter','jit'])
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

cases = []
for host in ['localhost', 'EXample.COM', '127.0.0.1:00080', 'a:65535', 'a:1',
             'a:', 'a:0', 'a:65536', 'a:9999999999999999', 'a:abc', 'a:+1',
             'a:1:2', 'user@host', 'x@y@z', 'user:pass@host', '[::1]',
             'a_b', 'a%20b', 'a/b', 'a?b', 'a#b', 'a#b#c', 'a/', 'a?',
             'a:0/path', 'a:abc/path', 'a:80/path', '', 'a'*253, 'a'*254]:
    cases.append(dict(host=host, target='/path?q=1', method='GET', body=''))
for target in ['/', '//double', '/a/../b', '/x?y=%2f&z=1', '/%00', '/%ff',
               '/%', '/%0', '/%gg', '/[x]', '/x#y', '/x\\y', '/é',
               "/!$&'()*+,;=:@_~.-", '/'+'x'*1007, '/'+'x'*1008,
               '/'+'x'*4080, '/'+'x'*4081]:
    cases.append(dict(host='localhost', target=target, method='GET', body=''))
cases.append(dict(host='EXample.COM:0080', target='/upload?x=%20', method='POST', body='owned payload'))
cases.append(dict(host='localhost', target='/stop', method='GET', body=''))
report = dict(passed=False, commands=commands, cases={}, inputs={}, contract='Public Accept wire oracle; exact fields, error kind/message and echoed body/content type; includes fragmented input.')
for path in [Path(__file__), Path(__file__).with_name('Probe.rvn'),b/'app', b/'app.dll',lib/'bundle.json']:
    report['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
oracle = json.loads(a.oracle.read_text())['cases']['aot'] if a.oracle else None
try:
    for mode in a.modes:
        guest = Guest(commands[mode], timeout=180)
        rows=[]
        try:
            port=int(guest.line()[0])
            for index,case in enumerate(cases):
                body=case['body'].encode()
                wire=(f"{case['method']} {case['target']} HTTP/1.1\r\nHost: {case['host']}\r\n" +
                      (f"Content-Length: {len(body)}\r\nContent-Type: text/custom\r\n" if body else '') + '\r\n').encode()+body
                with socket.create_connection(('127.0.0.1',port),timeout=120) as peer:
                    peer.sendall(wire[:17])
                    time.sleep(.005)
                    peer.sendall(wire[17:])
                    response=b''
                    while True:
                        block=peer.recv(8192)
                        if not block:break
                        response+=block
                kind=guest.line()[0].decode()
                fields=[guest.line()[0].decode() for _ in range(5 if kind=='OK' else 1)]
                if kind=='OK':
                    head, echoed=response.split(b'\r\n\r\n',1)
                    assert echoed==body
                    if body: assert b'content-type: text/custom' in head.lower()
                else: assert not response
                row=dict(case=case,kind=kind,fields=fields,response=response.decode())
                if oracle is not None: assert row==oracle[index], (mode,index,row,oracle[index])
                rows.append(row)
            guest.line(b'DONE'); guest.finish()
            report['cases'][mode]=rows
            if oracle is None: oracle=rows
            print('PASS', mode,len(rows),flush=True)
        except Exception as error:
            report['failure']=dict(mode=mode,index=index,error=str(error),stderr=guest.errors().decode(errors='replace'))
            raise
        finally: guest.close()
    report['passed']=True
finally: a.output.write_text(json.dumps(report,indent=2)+'\n')
