#!/usr/bin/env python3
"""Focused native lookup-counter validation with optimized and sanitized builds."""
import json
from pathlib import Path
import subprocess
root=Path(__file__).resolve().parents[3]
out=root/'target/lookup-profile'
out.mkdir(parents=True,exist_ok=True)
report=[]
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
cc=recipe[:recipe.index('-std=c11')]
for profile in [False,True]:
    for sanitized in [False,True]:
        exe=out/f'test-{profile}-{sanitized}'
        cmd=cc+['-pthread','-DNEOCLR_TEST_THREADS','-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC','-I',str(root/'docs/experiments/aot-console')]
        if profile:cmd+=['-DNEOCLR_GC_PROFILE']
        if sanitized:cmd+=['-fsanitize=undefined,bounds']
        cmd += [str(root/'docs/experiments/aot-console'/(n+'.c')) for n in ['native-gc-lookup-profile-test','native-gc','root-probe']]+['-o',str(exe)]
        for command in [cmd,[str(exe)]]:
            p=subprocess.run(command,capture_output=True,text=True)
            report.append(dict(command=command,exitCode=p.returncode,stdout=p.stdout,stderr=p.stderr))
            (out/'tests.json').write_text(json.dumps(report,indent=2)+'\n')
            p.check_returncode()
print('Four optimized/sanitized enabled/disabled tests passed')
