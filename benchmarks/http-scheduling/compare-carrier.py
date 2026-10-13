#!/usr/bin/env python3
"""Alternate uninstrumented retained-stack hosts using identical HTTP guest objects."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import resource
import statistics
import subprocess
import time
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('concurrency', ROOT/'benchmarks/http-concurrency/validate.py')
validation = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validation)
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--baseline', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = json.loads(a.build.read_text())
command = next(c['command'] for c in build['commands'] if c.get('phase') == 'host-compile-link')
carrier = ROOT/'docs/experiments/runtime-async/native-runtime.c'
assert str(carrier) in command
report = dict(passed=False, inputs={}, commands=[], runs=[], summaries=[],
              scope='Uninstrumented AOT matched carrier comparison, identical runtime-async application and libraries. Process CPU/wall include startup and teardown; phase durations are client-observed, not OS readiness latency. Four rounds alternate order after warmups.')
for value in [*command, str(a.build), str(a.baseline), __file__, validation.__file__]:
    if Path(value).is_file(): report['inputs'][str(Path(value).resolve())] = sha(value)
try:
    for name, source in [('baseline', a.baseline.resolve()), ('candidate', carrier)]:
        cmd = [str(source) if v == str(carrier) else v for v in command]
        cmd[cmd.index('-o')+1] = str(out/name)
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        report['commands'].append(cmd)
        report['inputs'][str(out/name)] = sha(out/name)
    for round_index in range(-1, 4):
        for hold in [0, .1]:
            for name in ['baseline', 'candidate'][::1 if round_index % 2 == 0 else -1]:
                before = resource.getrusage(resource.RUSAGE_CHILDREN)
                started = time.perf_counter()
                result = validation.run_case([str(out/name)], slow_hold_seconds=hold)
                seconds = time.perf_counter() - started
                after = resource.getrusage(resource.RUSAGE_CHILDREN)
                cpu = after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime
                report['runs'].append(dict(round=round_index, warmup=round_index < 0, variant=name,
                    holdSecondsPerPhase=hold, processSeconds=seconds, processCpuSeconds=cpu, validation=result))
                print(round_index, hold, name, round(seconds, 4), flush=True)
    for hold in [0, .1]:
        summary = dict(holdSecondsPerPhase=hold)
        for name in ['baseline', 'candidate']:
            rows = [r for r in report['runs'] if not r['warmup'] and r['variant'] == name and r['holdSecondsPerPhase'] == hold]
            summary[name] = {}
            for field in ['processSeconds', 'processCpuSeconds']:
                values = [r[field] for r in rows]
                summary[name][field] = dict(median=statistics.median(values), minimum=min(values), maximum=max(values))
        report['summaries'].append(summary)
    for path, digest in report['inputs'].items(): assert sha(path) == digest, path
    report['passed'] = True
finally:
    (out/'comparison.json').write_text(json.dumps(report, indent=2)+'\n')
