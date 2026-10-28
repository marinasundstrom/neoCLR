#!/usr/bin/env python3
"""Qualify private exclusive live-arena transfer; no generated worker admission."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
root = here.parents[2]
native = root/'runtime/native'
live = native/'gc/concurrent'
report = dict(passed=False, commands=[], inputs={}, samples=[])

def run(command):
    command = list(map(str, command))
    r = subprocess.run(command, capture_output=True, text=True, timeout=30)
    report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    return r.stdout

try:
    cc = [run(['xcrun','--find','clang']).strip(), '-isysroot', run(['xcrun','--show-sdk-path']).strip()]
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy = json.loads((live/'policy.json').read_text())
    for mode in ('sync','concurrent'):
        defs = ['-DNEOCLR_NATIVE_GC'] if mode=='sync' else ['-D'+d for d in policy['defines']]+['-DDELIVERY_CONCURRENT','-I',live]
        gc = [native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
        for variant, flags in [('optimized', []), ('ubsan', ['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
            binary = out/f'{mode}-{variant}'
            command = [*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations',
                 '-pthread','-Wl,-dead_strip','-DNEOCLR_EXCLUSIVE_HEAP_ADOPTION','-DNEOCLR_SHADOW_TEST',
                 *defs,*flags,'-I',native/'gc',here/'heap-adoption.c',native/'gc/root-probe.c',*gc,'-o',binary]
            run(command)
            run([binary])
            if variant=='optimized':
                command[command.index(here/'heap-adoption.c')]=here/'adoption-cost.c'
                command[command.index('-o')+1]=out/f'{mode}-cost'
                run(command)
    for trial in range(5):
        for mode in (('sync','concurrent') if trial%2==0 else ('concurrent','sync')):
            for line in run([out/f'{mode}-cost']).splitlines():
                sample=json.loads(line); sample.update(trial=trial,mode=mode)
                report['samples'].append(sample)
    paths = [Path(__file__),here/'heap-adoption.c',here/'adoption-cost.c',out/'live-gc.c']
    paths += [f for f in native.rglob('*') if f.suffix in ('.c','.h','.inc','.json','.py')]
    report['inputs'] = {str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in paths}
    report['passed'] = True
finally:
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS exclusive adoption contracts under sync/concurrent GC, optimized/UBSan')
