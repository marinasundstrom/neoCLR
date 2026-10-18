#!/usr/bin/env python3
"""Observe cancellation versus timeout for a waiting response producer."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'benchmarks/framework'))
from process import Guest
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--rounds',type=int,default=5)
p.add_argument('--executable',default='app')
a=p.parse_args()
b=a.build.resolve()
r=dict(build=str(b),inputs={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in [b/a.executable,b/'app.dll',b/'build.json',Path(__file__).resolve(),Path(__file__).with_name('DeadlineProbe.rvn')]},runs=[])
try:
 for index in range(a.rounds):
  g=Guest([str(b/a.executable)])
  try:
   port,_=g.line()
   for request in range(100):
    with socket.create_connection(('127.0.0.1',int(port)),timeout=10) as peer:
     peer.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\n\r\n')
     wire=b''
     while True:
      part=peer.recv(4096)
      if not part:break
      wire+=part
     assert len(wire)<1024
   cancelled=int(g.line()[0]);timed_out=int(g.line()[0]);g.line(b'DONE');g.finish()
   assert cancelled+timed_out==100
   r['runs'].append(dict(cancelled=cancelled,timedOut=timed_out))
   print(index+1,r['runs'][-1],flush=True)
  finally:g.close()
 r['allTimedOut']=all(x['cancelled']==0 for x in r['runs'])
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')
