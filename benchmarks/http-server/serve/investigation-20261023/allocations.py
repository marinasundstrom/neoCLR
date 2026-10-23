#!/usr/bin/env python3
"""Measure interpreter tracked objects by public HTTP stage; independent wire peers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import threading

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
bundle, out = a.bundle.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
report = dict(passed=False, commands=[], samples={},
              metric='Interpreter tracked managed objects; excludes host/native and some string storage. No throughput conclusion.',
              bundleManifestSha256=hashlib.sha256((bundle/'manifest.json').read_bytes()).hexdigest(),
              sources={f.name: hashlib.sha256(f.read_bytes()).hexdigest() for f in here.iterdir() if f.suffix in ('.py', '.rvn', '.rvnproj')})
lib = bundle/'lib'
def command(name):
    cmd = ['dotnet', str(bundle/'sdk/tools/rvnc/rvnc.dll'), 'neoclr', '--project', str(here/(name+'.rvnproj'))]
    r = subprocess.run(cmd, env=dict(os.environ, NeoClrBundleRoot=str(bundle)), capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(command=cmd, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    cmd = [str(bundle/'bin/neoclr'), 'run', str(here/'bin/neoclr'/(name+'.dll')), '--execution', 'interpreter',
           '--system', str(lib/'System.runtime.neox'), '--object-root', str(lib/'System.Runtime.dll'), '--instructions', '100000000']
    for name in ('System.Runtime','System.Data','System.Networking','System.Web'):
        cmd += ['--module', str(lib/(name+'.dll'))]
    return cmd
def read_headers(c):
    data = b''
    while b'\r\n\r\n' not in data:
        part = c.recv(4096)
        if not part: raise RuntimeError('Truncated request')
        data += part
    assert data.startswith(b'GET /items HTTP/1.1\r\n'), data
def samples(name, text, columns):
    values = [int(x) for x in text.splitlines()]
    assert len(values) == 9*len(columns), values
    rows = [dict(zip(columns, values[i:i+len(columns)])) for i in range(0,len(values),len(columns))]
    report['samples'][name] = dict(cold=rows[0], warm=rows[1:])
try:
    client = command('ClientAllocations')
    server = command('ServerAllocations')
    errors = []
    with socket.socket() as listener:
        listener.bind(('127.0.0.1',0)); listener.listen(); listener.settimeout(30)
        def peer():
            try:
                for _ in range(9):
                    with listener.accept()[0] as c:
                        c.settimeout(15); read_headers(c)
                        c.sendall(b'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}')
            except Exception as e: errors.append(repr(e))
        worker = threading.Thread(target=peer, daemon=True); worker.start()
        r = subprocess.run(client+['--', 'http://127.0.0.1:'+str(listener.getsockname()[1])+'/items'], capture_output=True, text=True, timeout=90)
        worker.join(1)
        report['clientExecution'] = dict(exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr, peerErrors=errors)
        r.check_returncode(); assert not errors and not worker.is_alive()
        samples('client',r.stdout,['constructRequest','sendAndReceive','firstContentType','repeatContentType','dispose'])
    guest = subprocess.Popen(server, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        port = int(guest.stdout.readline())
        for _ in range(9):
            with socket.create_connection(('127.0.0.1',port),timeout=15) as c:
                c.sendall(b'GET /items HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n')
                response = b''
                while True:
                    part = c.recv(4096)
                    if not part: break
                    response += part
                assert response.startswith(b'HTTP/1.1 200 ') and response.endswith(b'\r\n\r\n{}'), response
        stdout,stderr = guest.communicate(timeout=30)
        report['serverExecution'] = dict(exitCode=guest.returncode, stdout=stdout, stderr=stderr)
        assert guest.returncode == 0 and not stderr
        samples('server',stdout,['acceptAndRead','firstContentType','repeatContentType','configureResponse','complete','dispose'])
    finally:
        if guest.poll() is None: guest.kill(); guest.communicate()
    report['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report['samples'],indent=2))
