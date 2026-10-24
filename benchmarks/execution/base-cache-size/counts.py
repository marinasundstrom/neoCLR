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
parser.add_argument('--requests', type=int, nargs='+', default=[32,64])
args=parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
previous = json.loads((root/'target/native-host-ownership/results.json').read_text())
source = (root/'target/native-host-ownership/owned-host.c').read_text()
source += '\n__attribute__((destructor)) static void report_counts(void) {\n    neoclr_gc_lookup_profile s = neoclr_gc_lookup_profile_v1();\n    fprintf(stderr, "COUNTS %llu %llu %llu %llu %llu %llu %llu %llu %llu %llu %llu %llu\\n",\n        (unsigned long long)s.base_calls,\n        (unsigned long long)s.base_cache_hits,\n        (unsigned long long)s.base_walks,\n        (unsigned long long)s.base_spans,\n        (unsigned long long)s.index_build_spans,\n        (unsigned long long)s.index_queries,\n        (unsigned long long)s.index_outside,\n        (unsigned long long)s.index_probes,\n        (unsigned long long)s.mark_candidates,\n        (unsigned long long)s.payload_matches,\n        (unsigned long long)s.newly_marked,\n        (unsigned long long)s.object_words);\n}\n'
host_source = out/'counted-host.c'
host_source.write_text(source)
r = dict(passed=False, builds=[], trials=[], fields=['base_calls', 'base_cache_hits', 'base_walks', 'base_spans', 'index_build_spans', 'index_queries', 'index_outside', 'index_probes', 'mark_candidates', 'payload_matches', 'newly_marked', 'object_words'])
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
        build=[str(root/f'target/slot-maps/{args.sample}.o') if x==str(root/'target/root-map/candidate.o') else x for x in build]
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
    r['sample']=args.sample
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),host_source,args.gc_source or root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/native-gc.h',root/'runtime/native/gc/root-probe.c',root/'runtime/native/gc/execution-state.h',root/f'target/slot-maps/{args.sample}.o',root/'target/release/neoclr-codegen']}
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')
