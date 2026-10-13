#!/usr/bin/env python3
"""Check response bytes for HEAD, status rules and invalid headers in all modes."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'benchmarks/framework'))
from process import Guest
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--bundle',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();b=a.bundle.resolve();manifest=json.loads((b/'manifest.json').read_text())
context=['--system',str(b/manifest['runtimeSeed']),'--object-root',str(b/manifest['objectRoot'])]
for module in manifest['modules']:context+=['--module',str(b/module)]
assembly=str(a.build.resolve()/'app.dll')
commands={'interpreter':[str(b/manifest['runtime']),'run',assembly,'--execution','interpreter','--instructions','200000000',*context],
 'jit':[str(b/manifest['runtime']),'run',assembly], 'aot':[str(a.build.resolve()/'app')]}
report=dict(passed=False,commands=commands,cases=[],inputs={})
for path in [Path(__file__),Path(__file__).with_name('Server.rvn'),a.build/'app.dll',a.build/'app',b/'manifest.json']:
 report['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
try:
 for mode,command in commands.items():
  guest=Guest(command)
  try:
   port,_=guest.line()
   cases=[('GET','/',200,b'Caf\xc3\xa9 \xf0\x9f\x8c\x8d',10),('HEAD','/',200,b'',10),('GET','/204',204,b'',None),('GET','/205',205,b'',0),('GET','/304',304,b'',None),('GET','/299',299,b'Caf\xc3\xa9 \xf0\x9f\x8c\x8d',10),('GET','/reserved',None,b'',None),('GET','/injection',None,b'',None)]
   for method,target,status,body,length in cases:
    with socket.create_connection(('127.0.0.1',int(port)),timeout=60) as peer:
     peer.sendall(f'{method} {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'.encode())
     wire=b''
     while True:
      data=peer.recv(4096)
      if not data:break
      wire+=data
    if status is None:assert not wire,(mode,target,wire)
    else:
     head,content=wire.split(b'\r\n\r\n',1);lines=head.split(b'\r\n')
     assert lines[0]==f'HTTP/1.1 {status} '.encode()+(b'OK' if status==200 else b''),lines[0]
     fields=dict(line.lower().split(b': ',1) for line in lines[1:])
     assert fields==({b'x-test':b'ok',b'connection':b'close'}|({b'content-length':str(length).encode()} if length is not None else {})),fields
     assert content==body,(mode,target,content)
    report['cases'].append(dict(mode=mode,method=method,target=target,passed=True))
   guest.line(b'DONE');guest.finish()
  finally:guest.close()
 report['passed']=True
finally:a.output.write_text(json.dumps(report,indent=2)+'\n')
print('PASS',len(report['cases']),'response wire cases')
