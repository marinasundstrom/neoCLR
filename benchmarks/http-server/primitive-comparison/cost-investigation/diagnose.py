#!/usr/bin/env python3
"""Diagnostic-only allocation search counts and runtime snapshots; not capacity evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import resource
import queue
import re
import threading
import time
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--object', type=Path, required=True)
p.add_argument('--client', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--mode', choices=['off', 'basic', 'timed'], default='basic')
p.add_argument('--workers', type=int, choices=[1,4], default=4)
p.add_argument('--paced4', action='store_true')
p.add_argument('--append-first', action='store_true')
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={}, scope='Joined isolated HTTP owner intervals, including startup/shutdown; no coordinator/pool or process totals.')
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
    gc = root/'runtime/native/gc/native-gc.c'
    generated = out/'diagnostic-gc.c'
    source = gc.read_text()
    source = source.replace('#include <stdatomic.h>', '''#include <stdatomic.h>
#include <stdio.h>
static _Thread_local struct {
    uint64_t attempts, steps, fits, appends, peak;
} allocation_search;''')
    source = source.replace('    owned_heap.arena = arena;', '    memset(&allocation_search, 0, sizeof(allocation_search));\n    owned_heap.arena = arena;', 1)
    source = source.replace('    memset(&owned_heap, 0, sizeof(owned_heap));', '''    fprintf(stderr, "ALLOCATION_SEARCH %llu %llu %llu %llu %llu\\n",
        (unsigned long long)allocation_search.attempts,
        (unsigned long long)allocation_search.steps,
        (unsigned long long)allocation_search.fits,
        (unsigned long long)allocation_search.appends,
        (unsigned long long)allocation_search.peak);
    memset(&owned_heap, 0, sizeof(owned_heap));''', 1)
    source = source.replace('    int owned = a == owned_heap.arena;', '    int owned = a == owned_heap.arena;\n    if (owned) allocation_search.attempts++;', 1)
    source = source.replace('        if (!block_valid(a, at)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;', '        if (owned) allocation_search.steps++;\n        if (!block_valid(a, at)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;', 1)
    source = source.replace('    if (chosen) {\n', '    if (chosen) {\n        if (owned) allocation_search.fits++;\n', 1)
    source = source.replace('        a->used += needed;', '        a->used += needed;\n        if (owned) allocation_search.appends++;', 1)
    source = source.replace('        owned_heap.used = a->used;', '        owned_heap.used = a->used;\n        if (a->used > allocation_search.peak) allocation_search.peak = a->used;', 1)
    if a.paced4:
        needle = 'statistics.allocations - pacing.allocations < 256 && ++pacing.boundaries < 1024'
        assert source.count(needle) == 1
        source = source.replace(needle, 'statistics.allocations - pacing.allocations < 1024 && ++pacing.boundaries < 4096')
    if a.append_first:
        search = 'for (uint64_t at = owned ? owned_heap.first_free : 0; at < a->used;)'
        assert source.count(search) == 1
        source = source.replace(search,
            'for (uint64_t at = owned && needed <= a->capacity - a->used ? a->used : (owned ? owned_heap.first_free : 0); at < a->used;)')
    generated.write_text(source)
    record(generated)
    wrapper = str(root/'docs/experiments/aot-console/native-gc.c')
    assert wrapper in cmd
    cmd = [str(generated) if item == wrapper else item for item in cmd]
    cmd += ['-I', str(root/'runtime/native/gc')]
    cmd += ['-DNEOCLR_GC_PACED', '-DNEOCLR_GC_OWNED_HEAP', f'-DNEOCLR_HTTP_WORKERS={a.workers}']
    if a.mode != 'off':
        cmd += ['-DNEOCLR_RUNTIME_SNAPSHOT', '-I', str(root/'runtime/native/diagnostics'), str(root/'runtime/native/diagnostics/snapshot.c'), str(root/'benchmarks/http-server/primitive-comparison/runtime-snapshot/report.c')]
    if a.mode == 'timed': cmd += ['-DNEOCLR_GC_PROFILE']
    cmd[cmd.index('-o') + 1] = str(binary)
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
    result.check_returncode()
    for path in [a.build, a.object, a.client, __file__, binary, root/'benchmarks/native-web/http-host.c', root/'benchmarks/http-server/primitive-comparison/runtime-snapshot/report.c', *(root/'runtime/native').rglob('*.[ch]')]: record(path)
    cpu_before = resource.getrusage(resource.RUSAGE_CHILDREN)
    proc = subprocess.Popen([binary], env=dict(os.environ, NEOCLR_HTTP_RUNTIME_SNAPSHOT=str(stats)), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    lines, errors, events = [], [], queue.Queue()
    def read(stream, target):
        for line in stream:
            target.append(line)
            events.put(line)
    threads = [threading.Thread(target=read,args=(proc.stdout,lines)), threading.Thread(target=read,args=(proc.stderr,errors))]
    for thread in threads: thread.start()
    try:
        ready = []
        until = time.monotonic()+30
        while len(ready)<a.workers and time.monotonic()<until:
            try: line = events.get(timeout=1)
            except queue.Empty:
                if proc.poll() is not None: break
                continue
            ready += [(int(x),int(y)) for x,y in re.findall(r'READY (\d+) (\d+)',line)]
        assert len(ready)==a.workers and len({x for x,y in ready})==1, (lines,errors)
        port = str(ready[0][0])
        cmd = [str(a.client.resolve()), port, '4']
        result = subprocess.run(cmd, env=dict(os.environ, GOMAXPROCS='2'), capture_output=True, text=True, timeout=30)
        report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
        result.check_returncode()
        for _, control in ready:
            with socket.create_connection(('127.0.0.1', control), timeout=5): pass
        proc.wait(timeout=20)
        for thread in threads: thread.join(timeout=2)
        stdout, stderr = ''.join(lines), ''.join(errors)
        assert proc.returncode == 0, (stdout, stderr)
        report.update(load=json.loads(result.stdout), stdout=stdout, stderr=stderr, mode=a.mode, workers=a.workers)
        cpu_after = resource.getrusage(resource.RUSAGE_CHILDREN)
        report['serverCpuSeconds'] = cpu_after.ru_utime+cpu_after.ru_stime-cpu_before.ru_utime-cpu_before.ru_stime-report['load']['clientProcessCpuSeconds']
        report['cpuMicrosecondsPerResponse'] = report['serverCpuSeconds']*1e6/(report['load']['completed']+report['load']['drained'])
        if a.mode != 'off':
            report['snapshot'] = json.loads(stats.read_text())
            snapshot = report['snapshot']
            assert snapshot['version'] == 1 and len(snapshot['workers']) == a.workers
            for key, value in snapshot['total'].items():
                values = [w[key] for w in snapshot['workers']]
                assert (all(v is None for v in values) if value is None else sum(values) == value)
            assert snapshot['total']['allocations'] > 0
            assert snapshot['total']['asyncResumes'] > 0
            assert snapshot['total']['stackMappings'] == snapshot['total']['stackUnmappings']
            assert snapshot['total']['gcFailures'] == (0 if a.mode == 'timed' else None)
            count = report['load']['completed'] + report['load']['drained']
            report['perResponse'] = {k: None if v is None else v/count for k,v in snapshot['total'].items()}
        rows = [list(map(int, line.split()[1:])) for line in errors if line.startswith('ALLOCATION_SEARCH ')]
        assert len(rows) == a.workers and len(errors) == a.workers, errors
        totals = dict(zip(['attempts', 'steps', 'fits', 'appends'], [sum(row[i] for row in rows) for i in range(4)]))
        assert totals['fits'] + totals['appends'] <= totals['attempts']
        report['allocationSearch'] = dict(owners=rows, totals=totals,
            stepsPerAttempt=totals['steps']/totals['attempts'],
            peakArenaBytesPerOwner=[row[4] for row in rows])
        report['passed'] = True
        print(json.dumps(dict(mode=a.mode, perResponse=report.get('perResponse'), allocationSearch=report['allocationSearch'])))
    finally:
        if proc.poll() is None: proc.kill()
        proc.wait()
        for thread in threads: thread.join(timeout=2)
finally:
    (out/'profile.json').write_text(json.dumps(report, indent=2)+'\n')
