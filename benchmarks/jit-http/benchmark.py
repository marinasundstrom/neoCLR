#!/usr/bin/env python3
"""Fresh-build, correctness-checked HTTP client/server comparison on macOS ARM64."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import queue
import signal
import socket
import statistics
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
BODY = 'Café 🌍'.encode()
REQUEST = b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
RESPONSE = b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n' + BODY


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class Guest:
    def __init__(self, command):
        self.started = time.perf_counter()
        self.process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                        start_new_session=True)
        self.lines = queue.Queue()
        self.output = []
        def read():
            for line in self.process.stdout:
                self.output.append(line)
                self.lines.put((line.rstrip(b'\r\n'), time.perf_counter()))
            self.lines.put((None, time.perf_counter()))
        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def line(self, expected=None):
        line, at = self.lines.get(timeout=120)
        if line is None or expected is not None and line != expected:
            detail = self.process.stderr.read().decode(errors='replace') if line is None else ''
            raise RuntimeError(f'Unexpected guest output: {line!r}; expected {expected!r}; {detail}')
        return line, at

    def finish(self):
        code = self.process.wait(timeout=30)
        self.reader.join(timeout=5)
        error = self.process.stderr.read()
        if code or error or self.reader.is_alive():
            raise RuntimeError(f'Guest failed: {code}, {error!r}, {self.output!r}')

    def close(self):
        if self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGKILL)
            self.process.wait(timeout=10)
        self.reader.join(timeout=5)
        self.process.stdout.close()
        self.process.stderr.close()


def response(peer):
    data = b''
    while True:
        chunk = peer.recv(4096)
        if not chunk:
            break
        data += chunk
        if len(data) > 8192:
            raise RuntimeError('Oversized response')
    headers, body = data.split(b'\r\n\r\n', 1)
    fields = dict(line.lower().split(b': ', 1) for line in headers.split(b'\r\n')[1:])
    if headers.split(b'\r\n')[0] != b'HTTP/1.1 200 OK' or body != BODY or fields.get(b'content-length') != b'10' or fields.get(b'connection') != b'close':
        raise RuntimeError('Response contract changed: '+repr(data))


def server_sample(command):
    guest = Guest(command)
    try:
        port, ready = guest.line()
        port = int(port)
        samples = []
        for index in range(32):
            start = time.perf_counter()
            with socket.create_connection(('127.0.0.1', port), timeout=20) as peer:
                peer.sendall(REQUEST)
                response(peer)
            elapsed = time.perf_counter() - start
            if index >= 8:
                samples.append(elapsed)
        guest.line(b'DONE')
        guest.finish()
        return dict(startupSeconds=ready-guest.started, requestSeconds=samples,
                    measuredSeconds=sum(samples), requestsPerSecond=24/sum(samples))
    finally:
        guest.close()


def client_sample(command, mode):
    errors = []
    received = []
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        listener.listen(4)
        listener.settimeout(120)
        url = f'http://127.0.0.1:{listener.getsockname()[1]}/greeting'
        def serve():
            try:
                for index in range(32):
                    with listener.accept()[0] as peer:
                        peer.settimeout(20)
                        data = b''
                        while not data.endswith(b'\r\n\r\n'):
                            part = peer.recv(4096)
                            if not part or len(data) + len(part) > 8192:
                                raise RuntimeError('Invalid client request')
                            data += part
                        if not data.startswith(b'GET /greeting HTTP/1.1\r\n'):
                            raise RuntimeError('Client request changed')
                        received.append(index)
                        peer.sendall(RESPONSE)
            except Exception as error:
                errors.append(str(error))
        thread = threading.Thread(target=serve, daemon=True)
        thread.start()
        guest = Guest([*command, *([] if mode == 'aot' else ['--']), url])
        try:
            _, ready = guest.line(b'READY')
            _, warmed = guest.line(b'WARMED')
            _, done = guest.line(b'DONE')
            guest.finish()
            thread.join(timeout=5)
            if errors or len(received) != 32 or thread.is_alive():
                raise RuntimeError(f'Peer failed: {errors}, {len(received)} requests')
            return dict(startupSeconds=ready-guest.started, measuredSeconds=done-warmed,
                        requestsPerSecond=24/(done-warmed))
        finally:
            guest.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--recipe', type=Path, required=True, help='Passing macOS CLI HTTP qualification report; commands are rebuilt')
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--rounds', type=int, default=5)
    p.add_argument('--compiler', type=Path, help='Updated compatible Raven compiler; library inputs remain pinned by the recipe')
    a = p.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64' or a.rounds < 3:
        p.error('Requires macOS ARM64 and at least three rounds')
    recipe = json.loads(a.recipe.read_text())
    if not recipe['passed'] or not recipe.get('jitViaRuntimeCli'):
        raise ValueError('Passing CLI recipe required')
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
                  platform=platform.platform(), rounds=a.rounds, requestsPerProcess=32, warmRequests=8,
                  scope='Loopback HTTP/1.1, close, concurrency one, 10-byte UTF-8 payload; startup separate; Python peer on same machine',
                  recipeSha256=sha(a.recipe), builds=[], artifacts={}, inputs={}, warmups=[], samples=[], summary={})
    def build(command):
        for part in command:
            if isinstance(part, str) and Path(part).is_absolute() and Path(part).is_file():
                report['inputs'].setdefault(part, sha(part))
        started = time.perf_counter()
        result = subprocess.run(command, capture_output=True, text=True, timeout=300)
        report['builds'].append(dict(command=command, seconds=time.perf_counter()-started,
                                     exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stdout+result.stderr)
    try:
        if a.compiler:
            for dependency in sorted(a.compiler.resolve().parent.iterdir()):
                if dependency.is_file() and dependency.suffix in {'.dll', '.json'}:
                    report['inputs'][str(dependency)] = sha(dependency)
        host = out/'host.dylib'
        old_host = recipe['artifacts']['server']['commands']['jit']
        old_host = old_host[old_host.index('--jit-host')+1]
        build([str(host) if part == old_host else part for part in recipe['builds'][0]['command']])
        commands = {}
        for name, offset in [('server', 1), ('client', 4)]:
            folder = out/name
            folder.mkdir()
            old = Path(recipe['artifacts'][name]['commands']['aot'][0]).parent
            source = HERE/(name.title()+'.rvn')
            old_source = recipe['artifacts'][name]['source']
            def replace(command):
                return [str(source) if part == old_source else str(host) if part == old_host else
                        str(folder/Path(part).name) if Path(part).is_absolute() and Path(part).parent == old else part for part in command]
            (folder/'main.c').write_text((old/'main.c').read_text())
            for entry in recipe['builds'][offset:offset+3]:
                command = replace(entry['command'])
                if command[0] == 'dotnet' and a.compiler:
                    command[1] = str(a.compiler.resolve())
                build(command)
            commands[name] = {mode: replace(command) for mode,command in recipe['artifacts'][name]['commands'].items()}
            for command in commands[name].values():
                for part in command:
                    if Path(part).is_absolute() and Path(part).is_file():
                        report['inputs'].setdefault(part,sha(part))
            report['artifacts'][name] = dict(commands=commands[name], sourceSha256=sha(source),
                assemblySha256=sha(folder/'app.dll'), objectSha256=sha(folder/'app.o'), executableSha256=sha(folder/'app'))
        report['hostSha256'] = sha(host)
        report['inputs'][str(Path(__file__).resolve())] = sha(__file__)
        modes = ['interpreter', 'jit', 'aot']
        for round_index in range(-1, a.rounds):
            order = modes[max(0,round_index)%3:] + modes[:max(0,round_index)%3]
            for workload in ['server','client']:
                for mode in order:
                    value = server_sample(commands[workload][mode]) if workload == 'server' else client_sample(commands[workload][mode], mode)
                    value.update(workload=workload, mode=mode, round=round_index)
                    report['warmups' if round_index < 0 else 'samples'].append(value)
                    print(f'{workload} {mode} round {round_index}: {value["requestsPerSecond"]:.2f} req/s', flush=True)
        for workload in ['server','client']:
            report['summary'][workload] = {}
            for mode in modes:
                rows = [r for r in report['samples'] if r['workload']==workload and r['mode']==mode]
                summary = {}
                for metric in ['startupSeconds','requestsPerSecond','measuredSeconds']:
                    values = [r[metric] for r in rows]
                    summary[metric] = dict(median=statistics.median(values),minimum=min(values),maximum=max(values))
                if workload == 'server':
                    latencies=sorted(t for r in rows for t in r['requestSeconds'])
                    summary['requestLatencySeconds']={f'p{q}':latencies[math.ceil(q/100*len(latencies))-1] for q in [50,95]}
                report['summary'][workload][mode]=summary
        for path, expected in report['inputs'].items():
            if sha(path) != expected:
                raise RuntimeError('Benchmark input changed: '+path)
        report['passed'] = True
    finally:
        (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report['summary'],indent=2))


if __name__ == '__main__':
    main()
