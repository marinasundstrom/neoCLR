#!/usr/bin/env python3
"""Prove headers-first delivery and bounded reads against independent HTTP peers."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'benchmarks/framework'))
from process import Guest


def framing(case, server):
    if case in (0, 3, 6, 7) or (case == 2 and server):
        return b'Content-Length: '+str(513 if case in (0,2) else 10).encode()+b'\r\n'
    if case == 2:
        return b''
    return b'Transfer-Encoding: chunked\r\n'


def pieces(case):
    if case == 0: return b'xxx', b'x'*510
    if case == 1: return b'201\r\nxx', b'x'*511+b'\r\n0\r\n\r\n'
    if case == 2: return b'', b'x'*513
    if case == 3: return b'', b'xxx'
    if case == 4: return b'', b'g\r\n'
    if case == 5: return b'', b'100001\r\n'
    if case == 8: return b'', b'1\r\nx\r\n0\r\n\r\n'
    return b'', b''


def run_case(command, kind):
    server = kind == 'server'
    listener = None
    if not server:
        listener = socket.socket()
        listener.bind(('127.0.0.1',0))
        listener.listen(1)
        listener.settimeout(120)
        command = command+(['--'] if len(command)>1 and command[1]=='run' else [])+['http://127.0.0.1:'+str(listener.getsockname()[1])+'/']
    guest = Guest(command, timeout=180)
    rows = []
    try:
        if server: port, _ = guest.line()
        for case in range(9):
            if server:
                peer = socket.create_connection(('127.0.0.1',int(port)),timeout=90)
            else:
                peer, _ = listener.accept()
                peer.settimeout(90)
                request = b''
                while b'\r\n\r\n' not in request: request += peer.recv(4096)
                assert request.startswith(b'GET / HTTP/1.1\r\n')
            with peer:
                first, rest = pieces(case)
                line = b'POST / HTTP/1.1\r\nHost: localhost\r\n' if server else b'HTTP/1.1 200 OK\r\n'
                if case == 8:
                    line = b'GET / HTTP/1.1\r\nHost: localhost\r\n' if server else b'HTTP/1.1 205 Reset Content\r\n'
                peer.sendall(line+framing(case,server)+b'Connection: close\r\n\r\n'+first)
                # The application has a readable stream before the withheld body
                # arrives. No timer-based guess substitutes for this handshake.
                guest.line(b'HEADERS')
                if rest: peer.sendall(rest)
                if case == 3 or (case == 2 and not server): peer.shutdown(socket.SHUT_WR)
                wire = b''
                while True:
                    part = peer.recv(4096)
                    if not part: break
                    wire += part
                    assert len(wire)<4096
                if server and case<3:
                    assert wire.startswith(b'HTTP/1.1 200 OK\r\n')
                    assert wire.endswith(b'\r\n\r\nOK')
                else: assert wire == b'',wire
                guest.line(b'CASE')
                rows.append(dict(case=case,passed=True,headersBeforeBody=True))
        guest.line(b'DONE')
        guest.finish()
        return dict(passed=True,cases=rows,command=command)
    except Exception as error:
        raise RuntimeError(f'{kind} case {case}: {error}; {guest.errors()}') from error
    finally:
        guest.close()
        if listener: listener.close()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle',type=Path,required=True)
    p.add_argument('--client',type=Path,required=True)
    p.add_argument('--server',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--mode',action='append',choices=['jit','aot','interpreter'])
    a=p.parse_args();bundle=a.bundle.resolve();manifest=json.loads((bundle/'manifest.json').read_text())
    context=['--system',str(bundle/manifest['runtimeSeed']),'--object-root',str(bundle/manifest['objectRoot'])]
    for module in manifest['modules']:context+=['--module',str(bundle/module)]
    report=dict(passed=False,commands={},cases={},inputs={},scope='Headers-first streaming request and response body contract, not a performance measurement.')
    for kind in ['client','server']:
        build=getattr(a,kind).resolve()
        for name in ['app','app.dll','build.json']:
            report['inputs'][str(build/name)]=hashlib.sha256((build/name).read_bytes()).hexdigest()
        commands={'jit':[str(bundle/manifest['runtime']),'run',str(build/'app.dll')],
                  'interpreter':[str(bundle/manifest['runtime']),'run',str(build/'app.dll'),'--execution','interpreter','--instructions','2000000000',*context],
                  'aot':[str(build/'app')]}
        for mode in a.mode or ['jit','aot','interpreter']:
            key=kind+'-'+mode
            report['commands'][key]=commands[mode]
    try:
        for key,command in report['commands'].items():
            report['cases'][key]=run_case(command,key.split('-')[0]);print('PASS',key,flush=True)
        report['passed']=True
    finally:a.output.write_text(json.dumps(report,indent=2)+'\n')

if __name__=='__main__':main()
