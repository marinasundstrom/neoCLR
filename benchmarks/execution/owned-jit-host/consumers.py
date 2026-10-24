#!/usr/bin/env python3
"""Exercise installed owned hosts with previously admitted current assemblies."""
import gzip
import hashlib
import json
from pathlib import Path
import queue
import socket
import subprocess
import threading

root = Path(__file__).resolve().parents[3]
out = root/'target/owned-jit-consumers'
out.mkdir(exist_ok=False)
ordinary = json.loads(gzip.decompress((root/'benchmarks/execution/gc-mode-comparison/ordinary.json.gz').read_bytes()))
json_recipe = json.loads((root/'target/root-map/json-build.json').read_text())['command']
r = dict(passed=False, cases=[], inputs={})
try:
    for policy, folder in [('forced', 'owned-jit-bundle'), ('paced', 'owned-jit-paced-bundle')]:
        bundle = root/'target'/folder/'bundle'
        manifest = json.loads((bundle/'manifest.json').read_text())
        assert manifest['jit']['heapOwnership'] == 'exclusive-thread'
        for relative, digest in manifest['files'].items():
            assert hashlib.sha256((bundle/relative).read_bytes()).hexdigest() == digest
        runtime = bundle/manifest['runtime']
        for case in ordinary['cases']:
            assembly = next(x['command'][2] for x in case['runs'] if x['mode']=='interpreter')
            command = [str(runtime), 'run', assembly]
            x = subprocess.run(command, capture_output=True, text=True, timeout=120)
            row = dict(policy=policy, sample=case['sample'], command=command, exitCode=x.returncode, stdout=x.stdout, stderr=x.stderr)
            r['cases'].append(row)
            assert x.returncode == 0 and x.stdout == case['expected'] and not x.stderr, row
        for sample in ['plain', 'json']:
            assembly = root/'target/primitive-comparison-build/app.dll' if sample=='plain' else Path(json_recipe[2])
            command = [str(runtime), 'run', str(assembly)]
            if sample=='json':
                command += ['--reflection-roots', json_recipe[-1]]
            proc = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            lines, errors, events = [], [], queue.Queue()
            def read(stream, target):
                for line in stream:
                    target.append(line)
                    events.put(line)
            readers = [threading.Thread(target=read, args=(proc.stdout, lines)), threading.Thread(target=read, args=(proc.stderr, errors))]
            for reader in readers:
                reader.start()
            row = dict(policy=policy, sample=sample, command=command, responses=[])
            r['cases'].append(row)
            try:
                while True:
                    line = events.get(timeout=60)
                    if line.startswith('READY '):
                        break
                _, port, control = line.split()
                for _ in range(4):
                    with socket.create_connection(('127.0.0.1', int(port)), timeout=10) as connection:
                        connection.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                        response = b''
                        while part := connection.recv(4096):
                            response += part
                        header, body = response.split(b'\r\n\r\n', 1)
                        assert header.startswith(b'HTTP/1.1 200 ')
                        if sample=='plain':
                            assert body == b'xxxxxxxxxx'
                        else:
                            assert json.loads(body) == {'id':42, 'name':'Café team', 'active':True}, body
                        row['responses'].append(response.decode())
                with socket.create_connection(('127.0.0.1', int(control)), timeout=5):
                    pass
                proc.wait(timeout=30)
                for reader in readers:
                    reader.join(timeout=2)
                assert proc.returncode == 0 and not errors
                done = next(line for line in lines if line.startswith('DONE ')).split()
                assert done[-2:] == ['4', '0'], done
            finally:
                if proc.poll() is None:
                    proc.kill()
                proc.wait()
                for reader in readers:
                    reader.join(timeout=2)
                row.update(exitCode=proc.returncode, stdout=lines, stderr=errors)
        for path in [bundle/'manifest.json', runtime, bundle/manifest['jit']['compiler'], bundle/manifest['jit']['host']]:
            r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')
