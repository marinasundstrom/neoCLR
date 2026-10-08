#!/usr/bin/env python3
"""Check the successful greeting exchange; no performance timing.
Server must print its ephemeral loopback port or ASP.NET's 'Now listening on' line.
The harness starts and stops only its own server process.
"""
import argparse
import http.client
import json
import os
import re
import selectors
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--command', nargs=argparse.REMAINDER, required=True)
a = p.parse_args()
if not a.command:
    p.error('a server command is required')
server = subprocess.Popen(a.command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
log = b''
try:
    with selectors.DefaultSelector() as selector:
        selector.register(server.stdout, selectors.EVENT_READ)
        deadline = time.monotonic() + 30
        port = None
        while port is None:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise RuntimeError(f'server readiness timeout: {log!r}')
            if not selector.select(remaining):
                continue
            chunk = os.read(server.stdout.fileno(), 4096)
            if not chunk:
                raise RuntimeError(f'server exited before readiness: {log!r}')
            log += chunk
            match = re.search(rb'(?:^|\n)(\d+)\r?\n|Now listening on: http://127\.0\.0\.1:(\d+)\r?\n', log)
            if match:
                port = int(match.group(1) or match.group(2))
    connection = http.client.HTTPConnection('127.0.0.1', port, timeout=10)
    try:
        connection.request('GET', '/greeting', headers={'Connection': 'close'})
        response = connection.getresponse()
        body = response.read()
        expected = 'Café 🌍'.encode('utf-8')
        if (response.version != 11 or response.status != 200 or body != expected
                or response.getheader('Content-Type') != 'text/plain; charset=utf-8'
                or response.getheader('Content-Length') != str(len(expected))
                or response.getheader('Connection', '').lower() != 'close'):
            raise RuntimeError(f'Unexpected greeting: {response.status}, {response.getheaders()}, {body!r}')
        print(json.dumps({'command': a.command, 'protocol': 'HTTP/1.1', 'status': response.status,
                          'headers': response.getheaders(), 'bodyUtf8': body.decode('utf-8'),
                          'scope': 'one successful exchange only; no timing or malformed-request parity'}, indent=2))
    finally:
        connection.close()
finally:
    if server.poll() is None:
        server.terminate()
    try:
        server.wait(timeout=5)
    except subprocess.TimeoutExpired:
        server.kill()
        server.wait()
    server.stdout.close()
