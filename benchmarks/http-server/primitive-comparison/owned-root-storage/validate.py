#!/usr/bin/env python3
"""Optimized root checks, complementing the sanitizer/native GC suite."""
import argparse
import hashlib
import json
from pathlib import Path
import signal
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
out=a.output.resolve()
out.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[4]
base=root/'docs/experiments/aot-console'
report=dict(passed=False,inputs={},commands=[])
def run(cmd,expected=0):
    r=subprocess.run(list(map(str,cmd)),capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=list(map(str,cmd)),exitCode=r.returncode,expected=expected,stderr=r.stderr))
    assert r.returncode==expected,report['commands'][-1]
    return r.stdout.strip()
try:
    cc=run(['xcrun','--sdk','macosx','--find','clang'])
    sdk=run(['xcrun','--sdk','macosx','--show-sdk-path'])
    for gc in [False,True]:
        binary=out/('gc' if gc else 'probe')
        sources=[base/'root-validation-test.c',base/'root-probe.c']
        if gc: sources += [base/'native-gc.c']
        cmd=[cc,'-isysroot',sdk,'-std=c11','-O2','-Wall','-Wextra','-Werror',*sources,'-o',binary]
        if gc: cmd+=['-DNEOCLR_NATIVE_GC']
        run(cmd)
        for case in ['address','width','flags','fault','transient','plan-missing','plan-length','plan-short','batch-address','batch-width','batch-flags','batch-transient','context-mutated','context-null','context-ancestor-null']:
            run([binary,case],-signal.SIGABRT)
        for case in ['plan-moved','batch-valid','context-valid','state-valid']: run([binary,case])
        if gc:
            for case in ['owned-entry-invalid', 'owned-mixed-invalid', 'owned-transient-invalid']:
                run([binary, case], -signal.SIGABRT)
            run([binary, 'owned-valid'])
        for position in ['vector', 'tail']:
            for width in [0, 1, 3, 4, 5, 7, 8, 9, 12, 16, 2147483648, 4294967292, 4294967295]:
                run([binary, f'width-{position}-{width}'], 0 if width in [4, 8] else -signal.SIGABRT)
    inputs=[Path(__file__).resolve(),base/'root-validation-test.c',base/'root-probe.c',base/'native-gc.c',* (root/'runtime/native/gc').glob('*.[ch]')]
    for path in inputs: report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    report['passed']=True
finally:
    (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
