#!/usr/bin/env python3
"""Checked mixed HTTP qualification; closed-loop clients do not prove capacity."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import tempfile
import threading
import time
import traceback

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--modes', nargs='+', default=['baseline-sync', 'shared-sync', 'baseline-live', 'shared-live'])
p.add_argument('--clients', type=int, default=8)
p.add_argument('--cycles', type=int, default=1)
a = p.parse_args()
assert 0<a.clients<=16 and a.cycles>0
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
build_path = a.build.resolve()/'build.json'; build = json.loads(build_path.read_text())
assert build['passed'] and build['workload']=='twenty-four-route-mixed-http'
for path, digest in build['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest, ('stale input',path)
routes = [(kind,i) for kind in ('short','allocate','file-read','file-write','network','fanout') for i in range(4)]
report = dict(passed=False, kind='profile' if build.get('diagnosticOnly') else 'qualification', scope=__doc__, trials=[], clients=a.clients,
              cycles=a.cycles, downstreamDelaySeconds=[.002,.005,.010,.030],
              inputs={str(build_path):hashlib.sha256(build_path.read_bytes()).hexdigest(),
                      str(Path(__file__).resolve()):hashlib.sha256(Path(__file__).read_bytes()).hexdigest()})

class Downstream(BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    def do_GET(self):
        assert self.path in [f'/wait/{i}' for i in range(4)]
        variant = int(self.path[-1]); time.sleep(report['downstreamDelaySeconds'][variant])
        body = f'downstream:{variant}'.encode()
        self.send_response(200); self.send_header('Content-Length',str(len(body)))
        self.send_header('Connection','close'); self.end_headers(); self.wfile.write(body)
    def log_message(self, *args): pass

def traffic(port, selected):
    connection = http.client.HTTPConnection('127.0.0.1',port,timeout=30)
    rows = []
    try:
        for kind, variant in selected:
            route = f'/{kind}/{variant}'; began = time.perf_counter()
            connection.request('GET',route)
            response = connection.getresponse(); body = response.read().decode()
            expected = '10' if kind=='short' else str((256+variant*256)*(255+variant*256)//2) if kind=='allocate' else 'ok'
            assert response.status==200 and body==f'{kind}:{variant}:{expected}', (route,response.status,body)
            rows.append(dict(route=route,seconds=time.perf_counter()-began))
    finally: connection.close()
    return rows

peer = ThreadingHTTPServer(('127.0.0.1',19876),Downstream)
threading.Thread(target=peer.serve_forever,daemon=True).start()
try:
    for mode in a.modes:
        print(mode, flush=True)
        recipe=build['recipes'][mode]; lines=queue.Queue(); stdout=[]; stderr=[]
        trial=dict(mode=mode,sample='mixed',passed=False,command=recipe['command'],environment=recipe['env'])
        with tempfile.TemporaryDirectory(prefix='neoclr-mixed-http-') as directory:
            work=Path(directory); (work/'input.txt').write_text('checked file payload')
            process=subprocess.Popen(recipe['command'],cwd=work,env=dict(os.environ,**recipe['env']),
                                     text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            def drain(stream, target, notify=False):
                for line in stream:
                    target.append(line)
                    if notify: lines.put(line)
                if notify: lines.put(None)
            threads=[threading.Thread(target=drain,args=(process.stdout,stdout,True)),
                     threading.Thread(target=drain,args=(process.stderr,stderr))]
            for thread in threads: thread.start()
            try:
                deadline=time.monotonic()+90
                while True:
                    line=lines.get(timeout=max(.01,deadline-time.monotonic()))
                    assert line is not None,'server exited before readiness'
                    if line.strip()=='READY': break
                port=int(lines.get(timeout=10)); control=int(lines.get(timeout=10))
                trial['firstUse']=traffic(port,routes)
                began=time.perf_counter()
                with ThreadPoolExecutor(max_workers=a.clients) as executor:
                    jobs=[executor.submit(traffic,port,(routes[i:]+routes[:i])*a.cycles) for i in range(a.clients)]
                    trial['requests']=[row for job in jobs for row in job.result()]
                trial['batchSeconds']=time.perf_counter()-began
                for i in range(4): assert (work/f'output-{i}.txt').read_text()=='checked file payload'
                with socket.create_connection(('127.0.0.1',control),timeout=10): pass
                assert process.wait(timeout=30)==0
                for thread in threads: thread.join(timeout=5)
                expected=len(routes)*(1+a.clients*a.cycles)
                assert f'DONE {expected} 0' in ''.join(stdout)
                trial['passed']=True
            except Exception: trial['failure']=traceback.format_exc()
            finally:
                if process.poll() is None: process.kill(); process.wait(timeout=10)
                for thread in threads: thread.join(timeout=5)
                trial.update(stdout=''.join(stdout),stderr=''.join(stderr),exitCode=process.returncode)
        report['trials'].append(trial)
        (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
        assert trial['passed'],trial.get('failure')+'\n'+trial['stderr'][-3000:]
    report['passed']=True
finally:
    peer.shutdown(); peer.server_close()
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS all mixed routes, shared captures, checked I/O and shutdown')
