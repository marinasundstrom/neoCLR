#!/usr/bin/env python3
"""Check HTTP compilation and JIT readiness separately from capacity measurement."""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import threading
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[3]
previous = json.loads(gzip.decompress((root/'benchmarks/execution/gc-mode-comparison/http.json.gz').read_bytes()))
recipe = json.loads((root/'target/root-map/build.json').read_text())['commands'][0]['command']
r = dict(passed=False, compilation={}, trials=[])
try:
    command = recipe.copy()
    obj = (a.output/'http.o').resolve()
    command[4] = str(obj)
    started = time.perf_counter()
    x = subprocess.run(command, capture_output=True, text=True, timeout=120)
    r['compilation'] = dict(command=command, seconds=time.perf_counter()-started,
                            exitCode=x.returncode, stderr=x.stderr)
    x.check_returncode()
    r['compilation']['sha256'] = hashlib.sha256(obj.read_bytes()).hexdigest()
    r['compilation']['baselineSha256'] = hashlib.sha256(Path(recipe[4]).read_bytes()).hexdigest()
    assert r['compilation']['sha256'] == r['compilation']['baselineSha256']
    original = next(x['command'] for x in previous['trials'] if x['label']=='jit-paced')
    for variant in ['baseline', 'candidate', 'candidate', 'baseline']:
        command = original.copy()
        command[0] = str(root/('target/release/neoclr-codegen' if variant=='candidate'
                              else 'target/gc-mode-comparison-final/baseline-codegen'))
        started = time.perf_counter()
        proc = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        lines, errors, events = [], [], queue.Queue()
        def read(stream, target):
            for line in stream:
                target.append(line)
                events.put(line)
        readers = [threading.Thread(target=read, args=(proc.stdout, lines)),
                   threading.Thread(target=read, args=(proc.stderr, errors))]
        for reader in readers:
            reader.start()
        row = dict(variant=variant, command=command)
        r['trials'].append(row)
        try:
            while True:
                line = events.get(timeout=60)
                if line.startswith('READY '):
                    break
            _, port, control = line.split()
            row['readySeconds'] = time.perf_counter()-started
            client = [str(root/'target/primitive-comparison-refresh/client'), port, '1']
            x = subprocess.run(client, env=dict(os.environ, GOMAXPROCS='2'), capture_output=True, text=True, timeout=30)
            row['clientCommand'] = client
            row['clientExitCode'] = x.returncode
            row['clientStdout'], row['clientStderr'] = x.stdout, x.stderr
            x.check_returncode()
            with socket.create_connection(('127.0.0.1', int(control)), timeout=5):
                pass
            proc.wait(timeout=30)
            for reader in readers:
                reader.join(timeout=2)
            row['jit'] = json.loads(next(line[11:] for line in errors if line.startswith('JIT_RESULT ')))
            assert proc.returncode == 0 and row['jit']['guestExitCode'] == 0
        finally:
            if proc.poll() is None:
                proc.kill()
            proc.wait()
            for reader in readers:
                reader.join(timeout=2)
            row.update(exitCode=proc.returncode, stdout=lines, stderr=errors)
    r['passed'] = True
finally:
    (a.output/'results.json').write_text(json.dumps(r, indent=2)+'\n')
