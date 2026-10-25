"""Observe the current close-per-request boundary using qualified owner hosts.

This is a protocol baseline, not pipelining support or a throughput benchmark.
"""
import argparse
import hashlib
import json
import queue
import re
import socket
import subprocess
import threading
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--server', type=Path, action='append', required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
assert not a.output.exists()
report = dict(passed=False, scope=__doc__, trials=[])
request = b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n'
try:
    for binary in a.server:
        process = subprocess.Popen([str(binary.resolve())], stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True)
        events = queue.Queue()
        stdout, stderr = [], []
        def read(stream, lines):
            for line in stream:
                lines.append(line)
                events.put(line)
        readers = [threading.Thread(target=read, args=(stream, lines))
                   for stream, lines in [(process.stdout, stdout), (process.stderr, stderr)]]
        for reader in readers:
            reader.start()
        trial = dict(server=str(binary.resolve()), sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                     stdout=stdout, stderr=stderr, cases=[])
        report['trials'].append(trial)
        try:
            ready = None
            for _ in range(60):
                try:
                    match = re.search(r'READY (\d+) (\d+)', events.get(timeout=1))
                except queue.Empty:
                    assert process.poll() is None, 'host exited before readiness'
                    continue
                if match:
                    ready = tuple(map(int, match.groups()))
                    break
            assert ready, 'host readiness timed out'
            port, control = ready
            for name, count in [('keep-alive-request', 1), ('two-pipelined-requests', 2)]:
                with socket.create_connection(('127.0.0.1', port), timeout=5) as connection:
                    connection.sendall(request * count)
                    chunks = []
                    while True:
                        block = connection.recv(4096)
                        if not block:
                            break
                        chunks.append(block)
                wire = b''.join(chunks)
                head, body = wire.split(b'\r\n\r\n', 1)
                headers = {line.split(b':', 1)[0].lower(): line.split(b':', 1)[1].strip()
                           for line in head.split(b'\r\n')[1:]}
                assert head.startswith(b'HTTP/1.1 200')
                assert headers[b'connection'].lower() == b'close'
                assert int(headers[b'content-length']) == len(body)
                assert body == b'Hello, World!'
                trial['cases'].append(dict(name=name, sentRequests=count, receivedResponses=1,
                                           eof=True, wire=wire.decode('ascii')))
            with socket.create_connection(('127.0.0.1', control), timeout=5):
                pass
            assert process.wait(timeout=30) == 0
            for reader in readers:
                reader.join(timeout=2)
            done = re.findall(r'DONE (\d+) (\d+) (\d+)', ''.join(stdout))
            assert len(done) == 1 and int(done[0][1]) == 2
            trial['exitCode'] = process.returncode
        finally:
            if process.poll() is None:
                process.kill()
            process.wait()
            for reader in readers:
                reader.join(timeout=2)
    report['passed'] = True
finally:
    a.output.write_text(json.dumps(report, indent=2) + '\n')
