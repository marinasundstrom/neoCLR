#!/usr/bin/env python3
"""Evaluate an isolated pacing hypothesis; reject it if admission changes."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import statistics
import time
ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT/'benchmarks/http-server'))
import benchmark
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--workload', choices=['plain','json'], required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
build = json.loads(a.build.read_text()); assert build['passed']
base = next(c['command'][:] for c in build['commands'] if c.get('phase') == 'host-compile-link')
probe = ROOT/'docs/experiments/aot-console/root-probe.c'
marker = '    if (neoclr_gc_collect_v1((neoclr_aot_context *)frame->context, head)) abort();'
text = probe.read_text(); assert text.count(marker) == 1
candidate = out/'paced-probe.c'
candidate.write_text('#include "'+str(HERE/'policy.h')+'"\n'+text.replace(marker,
    '    if (pacing_collect((neoclr_aot_context *)frame->context)) {\n'+marker+'\n    }'))
report = dict(passed=False, promotionEligible=False, workload=a.workload, commands=[], inputs={}, runs=[],
    scope='Isolated AOT allocation-count/reserve pacing hypothesis. Shipping stress collector unchanged. Timing cannot override the admission counterexample.')
for v in [*base, str(a.build), __file__, str(HERE/'policy.h'), str(HERE/'pressure-test.c'), str(candidate), benchmark.__file__]:
    if Path(v).is_file(): report['inputs'][str(Path(v).resolve())] = sha(v)
for directory in ['aot-console','aot-fault-details']:
    for path in (ROOT/'docs/experiments'/directory).glob('*.h'): report['inputs'][str(path)]=sha(path)
def run(cmd):
    result = subprocess.run(list(map(str,cmd)),capture_output=True,text=True,timeout=120)
    report['commands'].append(dict(command=list(map(str,cmd)),exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr))
    if result.returncode: raise RuntimeError(result.stderr or result.stdout)
try:
    for variant in ['stress','paced']:
        cmd = [str(candidate) if v == str(probe) and variant == 'paced' else v for v in base]
        cmd[cmd.index('-o')+1] = str(out/variant)
        run(cmd)
        # Same rooted allocation sequence; candidate's expected failure must be
        # recoverable by explicit collection. No hidden allocation-time collection.
        cc = base[:base.index('-DNEOCLR_NATIVE_GC')]+['-DNEOCLR_NATIVE_GC','-fsanitize=undefined,bounds',
            '-I',str(probe.parent),str(HERE/'pressure-test.c'),str(probe.parent/'native-gc.c'),
            str(candidate if variant == 'paced' else probe),'-o',str(out/(variant+'-pressure'))]
        if variant == 'paced': cc.append('-DEXPECT_PACING_GAP')
        run(cc); run([out/(variant+'-pressure')])
    report['admission'] = 'Rejected: paced boundary faults on a 3.5 MiB allocation in a 4 MiB arena; stress and explicit-collect retry succeed.'
    for round_index in range(-1,4):
        for variant in ['stress','paced'][::1 if round_index%2 == 0 else -1]:
            result = benchmark.server_sample([str(out/variant)], a.workload)
            report['runs'].append(dict(round=round_index,warmup=round_index<0,variant=variant,validation=result))
            print(round_index,variant,flush=True)
    for path,h in report['inputs'].items(): assert sha(path)==h,path
    report['passed']=True
finally:
    (out/'evaluation.json').write_text(json.dumps(report,indent=2)+'\n')
