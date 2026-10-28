#!/usr/bin/env python3
"""Qualify the shared ready service and existing owner scheduler contracts."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
here=Path(__file__).resolve().parent;root=here.parents[2]
native=root/'runtime/native';legacy=root/'docs/experiments/aot-console';live=native/'gc/concurrent'
report=dict(passed=False,commands=[])
def run(command):
    command=list(map(str,command));r=subprocess.run(command,capture_output=True,text=True,timeout=30)
    report['commands'].append(dict(command=command,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr));r.check_returncode();return r.stdout
try:
    cc=[run(['xcrun','--find','clang']).strip(),'-isysroot',run(['xcrun','--show-sdk-path']).strip(),'-std=c11','-O2','-Wall','-Wextra','-Werror']
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy=json.loads((live/'policy.json').read_text())
    for variant,flags in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
        for diagnostic in (0,1):
            defs=['-DNEOCLR_SCHEDULER_DIAGNOSTICS'] if diagnostic else []
            binary=out/f'queue-{variant}-{diagnostic}'
            run([*cc,*flags,*defs,'-I',native/'scheduling',here/'queue-test.c','-o',binary]);run([binary])
        for mode in ('sync','concurrent'):
            defs=['-DNEOCLR_NATIVE_GC'] if mode=='sync' else ['-D'+d for d in policy['defines']]
            gc=[native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
            binary=out/f'scheduler-{mode}-{variant}'
            run([*cc,*flags,*defs,'-DNEOCLR_SHARED_READY_QUEUE','-DNEOCLR_SCHEDULER_DIAGNOSTICS','-pthread',
                 '-I',legacy,'-I',native/'gc','-I',live,legacy/'socket-scheduler-test.c',legacy/'socket-listener.c',
                 legacy/'task-queue.c',native/'memory/text-arena.c',native/'gc/root-probe.c',*gc,'-o',binary]);run([binary])
    files=[Path(__file__),here/'queue-test.c',legacy/'socket-listener.c',legacy/'socket-listener.h',legacy/'socket-scheduler-test.c',legacy/'task-queue.c',out/'live-gc.c']
    files += [p for p in native.rglob('*') if p.suffix in ('.c','.h','.inc','.py','.json')]
    report['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    report['passed']=True
finally:(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS shared queue and existing scheduler lifetime/fairness contracts')
