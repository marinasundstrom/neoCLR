#!/usr/bin/env python3
"""Check rooted file and DNS providers under synchronous and active concurrent GC."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
here=Path(__file__).resolve().parent;root=here.parents[2];native=root/'runtime/native'
legacy=root/'docs/experiments/aot-console';live=native/'gc/concurrent'
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);report=dict(passed=False,commands=[])
def run(command):
    command=list(map(str,command));r=subprocess.run(command,capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=command,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    r.check_returncode();return r.stdout
try:
    cc=[run(['xcrun','--find','clang']).strip(),'-isysroot',run(['xcrun','--show-sdk-path']).strip()]
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy=json.loads((live/'policy.json').read_text())
    for mode in ('sync','live'):
        defs=['NEOCLR_NATIVE_GC'] if mode=='sync' else policy['defines']+['FILE_LIVE','NEOCLR_CLIENT_LIVE']
        gc=[native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
        for variant,flags in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
            common=[*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread',*flags,
                    *['-D'+d for d in defs],'-I',legacy,'-I',native/'gc','-I',live,
                    *gc,native/'gc/root-probe.c',native/'memory/text-arena.c']
            for test in ('file-provider','native-gc-producer','socket-client'):
                binary=out/f'{mode}-{variant}-{test}'
                source=[here/'file-provider-test.c',legacy/'file-input.c',legacy/'file-output.c'] if test=='file-provider' else [legacy/'native-gc-producer-test.c']
                if test=='socket-client': source=[legacy/'socket-client-test.c',legacy/'socket-listener.c']; common_flags=['-DNEOCLR_CLIENT_TEST']
                else: common_flags=[]
                run([*common,*common_flags,*source,'-o',binary]);run([binary])
    files=[Path(__file__),here/'file-provider-test.c',legacy/'file-input.c',legacy/'file-output.c',legacy/'native-gc-producer-test.c']
    files += [legacy/name for name in ('socket-client-test.c','socket-client-internal.h','socket-listener.c','socket-listener.h','socket-os.h')]
    files += [f for f in native.rglob('*') if f.suffix in ('.c','.h','.inc','.py','.json')]
    report['inputs']={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in files}
    report['passed']=True
finally:(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS bounded rooted file and DNS providers in optimized/UBSan sync/live builds')
