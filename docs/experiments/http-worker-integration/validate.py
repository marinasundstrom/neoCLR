#!/usr/bin/env python3
"""Exercise one shared HTTP endpoint backed by isolated native application owners."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import http.client
import json
from pathlib import Path
import queue
import re
import socket
import subprocess
import threading
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--executable', type=Path, required=True)
p.add_argument('--workers', type=int, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--fault-exit', type=int, default=2)
p.add_argument('--requests', type=int, default=400)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
exe = a.executable.resolve()
report = dict(passed=False, executable=str(exe), executableSha256=hashlib.sha256(exe.read_bytes()).hexdigest(), workers=a.workers, runs=[])

def trial(fault=False):
    proc = subprocess.Popen([str(exe)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    lines, errors, events = [], [], queue.Queue()
    def read(stream, output):
        for line in stream:
            output.append(line); events.put(line)
    readers = [threading.Thread(target=read, args=(proc.stdout, lines)), threading.Thread(target=read, args=(proc.stderr, errors))]
    for t in readers: t.start()
    partial = []
    record = dict(stdout=lines, stderr=errors, injectedFault=fault)
    report['runs'].append(record)
    try:
        ready = []
        deadline = time.monotonic() + 30
        while len(ready) < a.workers and time.monotonic() < deadline:
            try: line = events.get(timeout=1)
            except queue.Empty:
                if proc.poll() is not None: break
                continue
            ready += [(int(x), int(y)) for x,y in re.findall(r'READY (\d+) (\d+)', line)]
        assert len(ready) == a.workers, (ready, lines, errors)
        ports = {x for x,y in ready}; controls = {y for x,y in ready}
        assert len(ports) == 1 and len(controls) == a.workers, ready
        port = next(iter(ports)); record['ready'] = ready
        if fault:
            c = http.client.HTTPConnection('127.0.0.1',port,timeout=5)
            try:
                c.request('GET', '/fault', headers={'Connection':'close'})
                try: c.getresponse().read()
                except (OSError, http.client.HTTPException): pass
            finally: c.close()
            proc.wait(timeout=20)
            for t in readers: t.join(timeout=2)
            record['exitCode'] = proc.returncode
            assert proc.returncode == a.fault_exit and 'Injected HTTP worker failure' in ''.join(errors), record
            return
        # Incomplete request reads stay pending while unrelated requests finish.
        for _ in range(8):
            s = socket.create_connection(('127.0.0.1',port), timeout=10)
            s.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\n'); partial.append(s)
        def request(index):
            c = http.client.HTTPConnection('127.0.0.1',port,timeout=15)
            try:
                c.request('GET', '/', headers={'Connection':'close'})
                r = c.getresponse(); body = r.read()
                assert r.status == 200 and int(body) in controls, (r.status,body)
                return int(body)
            finally: c.close()
        start = time.monotonic()
        with ThreadPoolExecutor(max_workers=32) as executor:
            owners = list(executor.map(request, range(a.requests)))
        record['seconds'] = time.monotonic()-start
        record['responses'] = len(owners)
        record['ownerResponses'] = {str(owner):owners.count(owner) for owner in controls}
        assert set(owners) == controls, record
        # Request shutdown while the incomplete reads are still pending.
        for control in controls:
            with socket.create_connection(('127.0.0.1',control),timeout=5): pass
        proc.wait(timeout=20)
        for t in readers: t.join(timeout=2)
        record['exitCode'] = proc.returncode
        done = re.findall(r'DONE (\d+) (\d+) (\d+)', ''.join(lines))
        record['done'] = done
        assert proc.returncode == 0 and len(done) == a.workers, record
        assert {int(row[0]) for row in done} == controls, record
        assert sum(int(row[1]) for row in done) == a.requests, record
    finally:
        for s in partial: s.close()
        if proc.poll() is None: proc.kill()
        proc.wait()
        for t in readers: t.join(timeout=2)
try:
    for _ in range(3): trial()
    trial(fault=True)
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps([dict(responses=r['responses'],owners=r['ownerResponses'],seconds=r['seconds']) for r in report['runs'] if not r['injectedFault']],indent=2))
