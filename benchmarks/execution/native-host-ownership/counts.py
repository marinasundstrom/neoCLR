#!/usr/bin/env python3
"""Fixed-request native work counts, separate from capacity and stack profiling."""
import json
from pathlib import Path
import queue
import socket
import subprocess
import threading

root = Path(__file__).resolve().parents[3]
out = root/'target/native-host-counts'
out.mkdir(exist_ok=False)
previous = json.loads((root/'target/native-host-ownership/results.json').read_text())
source = (root/'target/native-host-ownership/owned-host.c').read_text()
source += '''
__attribute__((destructor)) static void report_counts(void) {
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    fprintf(stderr, "COUNTS %llu %llu %llu\\n", (unsigned long long)s.allocations,
        (unsigned long long)s.collections, (unsigned long long)neoclr_root_probe_count_v1());
}
'''
host_source = out/'counted-host.c'
host_source.write_text(source)
r = dict(passed=False, builds=[], trials=[])
try:
    commands = {}
    for mode in ['aot', 'jit']:
        old = root/('target/native-host-ownership/owned-aot' if mode=='aot' else 'target/native-host-ownership/owned.dylib')
        build = next(x['command'].copy() for x in previous['commands'] if str(old) in x['command'])
        build = [str(host_source) if x.endswith('/owned-host.c') else x for x in build]
        artifact = out/('server' if mode=='aot' else 'host.dylib')
        build[build.index('-o')+1] = str(artifact)
        x = subprocess.run(build, capture_output=True, text=True, timeout=90)
        r['builds'].append(dict(command=build, exitCode=x.returncode, stderr=x.stderr))
        x.check_returncode()
        cmd = next(x['command'].copy() for x in previous['trials'] if x['label']==f'{mode}-owned')
        cmd = [str(artifact) if arg == str(old) else arg for arg in cmd]
        commands[mode] = cmd
    for count in [2, 4]:
        for mode, cmd in commands.items():
            proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            lines, errors, events = [], [], queue.Queue()
            def read(stream, target):
                for line in stream:
                    target.append(line)
                    events.put(line)
            readers = [threading.Thread(target=read, args=(proc.stdout, lines)), threading.Thread(target=read, args=(proc.stderr, errors))]
            for reader in readers:
                reader.start()
            row = dict(mode=mode, count=count, command=cmd)
            r['trials'].append(row)
            try:
                while True:
                    line = events.get(timeout=60)
                    if line.startswith('READY '):
                        break
                _, port, control = line.split()
                for _ in range(count):
                    with socket.create_connection(('127.0.0.1', int(port)), timeout=10) as connection:
                        connection.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                        response = b''
                        while True:
                            part = connection.recv(4096)
                            if not part:
                                break
                            response += part
                        header, body = response.split(b'\r\n\r\n', 1)
                        assert header.startswith(b'HTTP/1.1 200 ') and body == b'xxxxxxxxxx'
                with socket.create_connection(('127.0.0.1', int(control)), timeout=10):
                    pass
                proc.wait(timeout=30)
                for reader in readers:
                    reader.join(timeout=2)
                assert proc.returncode == 0
                done = next(line for line in lines if line.startswith('DONE ')).split()
                assert int(done[2]) == count and int(done[3]) == 0
                row['counters'] = list(map(int, next(line for line in errors if line.startswith('COUNTS ')).split()[1:]))
                if mode=='jit':
                    assert json.loads(next(line[11:] for line in errors if line.startswith('JIT_RESULT ')))['guestExitCode']==0
            finally:
                if proc.poll() is None:
                    proc.kill()
                proc.wait()
                for reader in readers:
                    reader.join(timeout=2)
                row.update(exitCode=proc.returncode, stdout=lines, stderr=errors)
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')
