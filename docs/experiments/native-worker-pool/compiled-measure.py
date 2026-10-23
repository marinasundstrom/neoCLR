#!/usr/bin/env python3
"""Compare the same compiled graph workload with serial and concurrent heap coordination."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--validation', type=Path, required=True)
p.add_argument('--relink-current-runtime', action='store_true')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
baseline = json.loads(a.validation.read_text())
assert baseline['passed'] and baseline['scope'] == 'Concurrent compiled shared heap'
report = dict(passed=False, commands=[], trials=[], inputs={}, jobs=4096, warmupJobs=16,
              heapBytes=32768, scope='Fixed graph work, 16 queue slots, identical heap. Diagnostic timings are separate from uninstrumented timings. No HTTP throughput claim.')
def record(path):
    path = Path(path)
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
def run(cmd):
    r = subprocess.run(list(map(str, cmd)), capture_output=True, text=True, timeout=60)
    report['commands'].append(dict(command=list(map(str, cmd)), exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    return r.stdout

try:
    report['runtimeRelinkChanges'] = {}
    runtime = here.parents[2]/'runtime/native'
    for path, digest in baseline['inputs'].items():
        current = hashlib.sha256(Path(path).read_bytes()).hexdigest()
        if current != digest:
            assert a.relink_current_runtime and runtime in Path(path).parents, path
            report['runtimeRelinkChanges'][path] = dict(previous=digest, current=current)
        record(path)
    record(a.validation)
    record(__file__)
    record(here/'compiled-benchmark.c')
    codegen = next(c['command'] for c in baseline['commands'] if '--closed-world' in c['command'])
    parallel_object = Path(codegen[4])
    record(parallel_object)
    serial = [s for s in codegen if s != '--native-worker-safepoints']
    serial[4] = str(out/'serial.o')
    run(serial)
    record(out/'serial.o')
    link = next(c['command'] for c in baseline['commands'] if any(s.endswith('/parallel-host.c') for s in c['command']))
    for mode in ['serial', 'parallel']:
        for profile in [False, True]:
            exe = out/(mode + ('-profile' if profile else ''))
            cmd = [str(here/'compiled-benchmark.c') if s.endswith('/parallel-host.c') else
                   str(out/'serial.o') if s == str(parallel_object) and mode == 'serial' else s for s in link]
            cmd[cmd.index('-o')+1] = str(exe)
            if profile: cmd.append('-DNEOCLR_GC_PROFILE')
            run(cmd)
            record(exe)
    # All builds finish before measurement. Reverse order limits simple order bias.
    order = [('serial',1), ('parallel',1), ('parallel',2), ('parallel',4)]
    for mode, workers in order + order[::-1]:
        sample = json.loads(run([out/mode, str(workers), str(int(mode == 'parallel'))]))
        sample.update(mode=mode, profiled=False)
        report['trials'].append(sample)
        print(json.dumps(sample), flush=True)
    for mode, workers in [('serial',1), ('parallel',1), ('parallel',4)]:
        sample = json.loads(run([out/(mode+'-profile'), str(workers), str(int(mode == 'parallel'))]))
        sample.update(mode=mode, profiled=True)
        report['trials'].append(sample)
        print(json.dumps(sample), flush=True)
    assert len({s['allocations'] for s in report['trials']}) == 1
    assert len({s['payloadBytes'] for s in report['trials'] if s['profiled']}) == 1
    report['passed'] = True
finally:
    (out/'measurement.json').write_text(json.dumps(report, indent=2)+'\n')
