#!/usr/bin/env python3
"""Capture macOS stacks during checked HTTP load using a verified existing binary."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import queue
import re
import threading
import time
import subprocess
p = argparse.ArgumentParser(description='macOS stack samples during checked HTTP load; not capacity or exact CPU attribution.')
p.add_argument('--build-report', type=Path, required=True)
p.add_argument('--client', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={}, scope='macOS all-thread wall-clock stack samples under load; blocked threads included; not capacity evidence.')
build = json.loads(a.build_report.read_text())
assert build['passed']
a.workers = build['workers']
binary = Path(next(c['command'][c['command'].index('-o')+1] for c in build['commands'] if '-o' in c['command']))
assert hashlib.sha256(binary.read_bytes()).hexdigest() == build['inputs'][str(binary)]
for path in [a.build_report.resolve(), binary, a.client.resolve(), Path(__file__).resolve()]:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
stats = out/'snapshot.json'
try:
    proc = subprocess.Popen([binary], env=dict(os.environ, NEOCLR_HTTP_RUNTIME_SNAPSHOT=str(stats)), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    lines, errors, events = [], [], queue.Queue()
    def read(stream, target):
        for line in stream:
            target.append(line)
            events.put(line)
    threads = [threading.Thread(target=read,args=(proc.stdout,lines)), threading.Thread(target=read,args=(proc.stderr,errors))]
    for thread in threads: thread.start()
    try:
        ready = []
        until = time.monotonic()+30
        while len(ready)<a.workers and time.monotonic()<until:
            try: line = events.get(timeout=1)
            except queue.Empty:
                if proc.poll() is not None: break
                continue
            ready += [(int(x),int(y)) for x,y in re.findall(r'READY (\d+) (\d+)',line)]
        assert len(ready)==a.workers and len({x for x,y in ready})==1, (lines,errors)
        port = str(ready[0][0])
        cmd = [str(a.client.resolve()), port, '8']
        client = subprocess.Popen(cmd, env=dict(os.environ, GOMAXPROCS='2'), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        sample_command = ['/usr/bin/sample', str(proc.pid), '5', '1', '-file', str(out/'sample.txt')]
        try:
            sampled = subprocess.run(sample_command, capture_output=True, text=True, timeout=20)
            report['commands'].append(dict(command=sample_command, exitCode=sampled.returncode, stdout=sampled.stdout, stderr=sampled.stderr))
            sampled.check_returncode()
            client_stdout, client_stderr = client.communicate(timeout=20)
            result = subprocess.CompletedProcess(cmd, client.returncode, client_stdout, client_stderr)
        finally:
            if client.poll() is None: client.kill()
            client.wait()
        report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
        result.check_returncode()
        for _, control in ready:
            with socket.create_connection(('127.0.0.1', control), timeout=5): pass
        proc.wait(timeout=20)
        for thread in threads: thread.join(timeout=2)
        stdout, stderr = ''.join(lines), ''.join(errors)
        assert proc.returncode == 0, (stdout, stderr)
        report.update(load=json.loads(result.stdout), stdout=stdout, stderr=stderr, workers=a.workers)
        report['sampleSha256'] = hashlib.sha256((out/'sample.txt').read_bytes()).hexdigest()
        report['passed'] = True
        print(json.dumps(dict(passed=True, workers=a.workers, responses=report['load']['completed']+report['load']['drained'])))
    finally:
        if proc.poll() is None: proc.kill()
        proc.wait()
        for thread in threads: thread.join(timeout=2)
finally:
    (out/'capture.json').write_text(json.dumps(report, indent=2)+'\n')
