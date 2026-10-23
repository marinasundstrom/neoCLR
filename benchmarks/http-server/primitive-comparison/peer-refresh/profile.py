#!/usr/bin/env python3
"""Current main-owner runtime work inventory; instrumented, not capacity evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--object', type=Path, required=True)
p.add_argument('--client', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={}, scope='Main-owner instrumented work counts; not worker throughput. Startup and shutdown included.')
def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
try:
    build = json.loads(a.build.read_text())
    assert build['passed']
    recipe = next(c['command'] for c in build['commands'] if c.get('phase') == 'host-compile-link')
    original = next(c['command'][4] for c in build['commands'] if c.get('phase') == 'native-compilation')
    binary, stats = out/'server', out/'gc.json'
    cmd = [str(a.object.resolve()) if s == original else s for s in recipe if not s.startswith('-DNEOCLR_HTTP_WORKERS=')]
    cmd += ['-DNEOCLR_GC_PACED', '-DNEOCLR_GC_OWNED_HEAP', '-DNEOCLR_GC_PROFILE', '-I', str(root/'runtime/native/gc'), str(root/'benchmarks/http-server/serve/profile-native.c')]
    cmd[cmd.index('-o') + 1] = str(binary)
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
    result.check_returncode()
    for path in [a.build, a.object, a.client, __file__, binary, root/'benchmarks/native-web/http-host.c', root/'benchmarks/http-server/serve/profile-native.c', *(root/'runtime/native').rglob('*.[ch]')]: record(path)
    proc = subprocess.Popen([binary], env=dict(os.environ, NEOCLR_HTTP_CAPACITY_PROFILE=str(stats)), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        line = proc.stdout.readline()
        tag, port, control = line.split()
        assert tag == 'READY', line
        cmd = [str(a.client.resolve()), port, '4']
        result = subprocess.run(cmd, env=dict(os.environ, GOMAXPROCS='2'), capture_output=True, text=True, timeout=30)
        report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
        result.check_returncode()
        with socket.create_connection(('127.0.0.1', int(control)), timeout=5): pass
        stdout, stderr = proc.communicate(timeout=20)
        assert proc.returncode == 0, (stdout, stderr)
        report.update(load=json.loads(result.stdout), gc=json.loads(stats.read_text()), stdout=line+stdout, stderr=stderr)
        count = report['load']['completed'] + report['load']['drained']
        report['perResponse'] = {key: report['gc'][key] / count for key in ['allocations', 'allocatedBytes', 'rootPublications', 'attempts', 'gcNanoseconds']}
        assert report['gc']['failures'] == 0
        report['passed'] = True
        print(json.dumps(report['perResponse']))
    finally:
        if proc.poll() is None: proc.kill()
        proc.wait()
finally:
    (out/'profile.json').write_text(json.dumps(report, indent=2)+'\n')
