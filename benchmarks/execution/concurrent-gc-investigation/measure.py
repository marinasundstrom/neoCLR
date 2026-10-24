#!/usr/bin/env python3
"""Fixed-request native work counts, separate from capacity and stack profiling."""
import argparse
import hashlib
import json
from pathlib import Path
import queue
import socket
import subprocess
import threading

root = Path(__file__).resolve().parents[3]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--sample', choices=['plain','json'], default='plain')
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--gc-source', type=Path)
parser.add_argument('--forced', action='store_true')
parser.add_argument('--requests', type=int, nargs='+', default=[1024,1024,1024])
args=parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
previous = json.loads((root/'target/native-host-ownership/results.json').read_text())
source = (root/'target/native-host-ownership/owned-host.c').read_text()
source += '\n__attribute__((destructor)) static void report_counts(void) {\n    neoclr_gc_trigger_profile t = neoclr_gc_trigger_profile_v1();\n    neoclr_gc_profile p = neoclr_gc_profile_v1();\n    neoclr_gc_statistics s = neoclr_gc_statistics_v1();\n    fprintf(stderr, "TRIGGERS");\n    for (unsigned i = 0; i < NEOCLR_GC_TRIGGER_COUNT; i++) fprintf(stderr, " %llu", (unsigned long long)t.requests[i]);\n    for (unsigned i = 0; i < NEOCLR_GC_TRIGGER_COUNT; i++) fprintf(stderr, " %llu", (unsigned long long)t.failures[i]);\n    fprintf(stderr, "\\nGC %llu %llu %llu %llu %llu %llu %llu %llu\\n",\n        (unsigned long long)p.attempts, (unsigned long long)p.failures,\n        (unsigned long long)s.allocations, (unsigned long long)neoclr_root_probe_count_v1(),\n        (unsigned long long)p.nanoseconds, (unsigned long long)p.peak_live_bytes,\n        (unsigned long long)p.peak_arena_used, (unsigned long long)p.allocated_bytes);\n}\n'
host_source = out/'counted-host.c'
host_source.write_text(source)
r = dict(passed=False, builds=[], trials=[], fields=['direct','idle','pressure','managed','context','epoch','allocation','publication'], gcFields=['attempts','failures','allocations','publications','nanoseconds','peak_live_bytes','peak_arena_used','allocated_bytes'])
try:
    commands = {}
    for mode in ['aot', 'jit']:
        old = root/('target/native-host-ownership/owned-aot' if mode=='aot' else 'target/native-host-ownership/owned.dylib')
        build = next(x['command'].copy() for x in previous['commands'] if str(old) in x['command'])
        build = [str(host_source) if x.endswith('/owned-host.c') else x for x in build]
        artifact = out/('server' if mode=='aot' else 'host.dylib')
        build[build.index('-o')+1] = str(artifact)
        build.append('-DNEOCLR_GC_PROFILE')
        if args.forced:
            build.remove('-DNEOCLR_GC_PACED')
        if args.gc_source:
            build=[str(args.gc_source.resolve()) if x.endswith('/native-gc.c') else x for x in build]
        build=[str(root/f'target/state-integration/{args.sample}.o') if x==str(root/'target/root-map/candidate.o') else x for x in build]
        x = subprocess.run(build, capture_output=True, text=True, timeout=90)
        r['builds'].append(dict(command=build, exitCode=x.returncode, stderr=x.stderr))
        x.check_returncode()
        cmd = next(x['command'].copy() for x in previous['trials'] if x['label']==f'{mode}-owned')
        cmd = [str(artifact) if arg == str(old) else arg for arg in cmd]
        if args.sample=='json' and mode=='jit':
            recipe=json.loads((root/'target/root-map/json-build.json').read_text())['command']
            cmd=[recipe[0],'--jit-managed',*recipe[1:4],str(artifact),*recipe[5:]]
        commands[mode] = cmd
    for count in args.requests:
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
                        assert header.startswith(b'HTTP/1.1 200 ')
                        if args.sample=='plain':
                            assert body == b'xxxxxxxxxx'
                        else:
                            assert json.loads(body)==dict(id=42,name='Café team',active=True)
                with socket.create_connection(('127.0.0.1', int(control)), timeout=10):
                    pass
                proc.wait(timeout=30)
                for reader in readers:
                    reader.join(timeout=2)
                assert proc.returncode == 0
                done = next(line for line in lines if line.startswith('DONE ')).split()
                assert int(done[2]) == count and int(done[3]) == 0
                values = list(map(int, next(line for line in errors if line.startswith('TRIGGERS ')).split()[1:]))
                assert len(values)==16
                row['requests'],row['failures']=values[:8],values[8:]
                row['gc']=list(map(int,next(line for line in errors if line.startswith('GC ')).split()[1:]))
                assert sum(row['requests'])==row['gc'][0] and not sum(row['failures']) and row['gc'][1]==0
                if mode=='jit':
                    assert json.loads(next(line[11:] for line in errors if line.startswith('JIT_RESULT ')))['guestExitCode']==0
            finally:
                if proc.poll() is None:
                    proc.kill()
                proc.wait()
                for reader in readers:
                    reader.join(timeout=2)
                row.update(exitCode=proc.returncode, stdout=lines, stderr=errors)
    r['sample']=args.sample
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),host_source,root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/native-gc.h',root/'runtime/native/gc/root-probe.c',root/'runtime/native/gc/root-probe.h',root/'runtime/native/gc/execution-state.h',root/f'target/state-integration/{args.sample}.o',root/'target/release/neoclr-codegen']}
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')
