#!/usr/bin/env python3
"""Reproduce snapshot overlap and its conditional-root counterexample, not capacity."""
import argparse, hashlib, json, os, shutil, subprocess
from pathlib import Path
here=Path(__file__).resolve().parent
root=here.parents[2]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--thread-sanitizer',action='store_true')
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
source=root/'runtime/native/gc/native-gc.c'
assert hashlib.sha256(source.read_bytes()).hexdigest()=="fefe1387031fa7c68bf9680e7abf34ce7d6b8ae66ec7187747b5b69ec813c13c", 'Rebase and review the experiment patch before applying to a changed collector'
report=dict(passed=False,productionReady=False,commands=[],inputs={})
for path in [source,here/'native-gc.patch',here/'concurrent-snapshot.inc',here/'test.c',Path(__file__)]:
    report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
def run(command):
    r=subprocess.run(list(map(str,command)),capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=r.args,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    r.check_returncode()
    return r
try:
    shutil.copyfile(source,out/'native-gc.c')
    for name in ['concurrent-snapshot.inc','test.c']:shutil.copyfile(here/name,out/name)
    run(['patch',out/'native-gc.c',here/'native-gc.patch'])
    run(['clang','-std=c11','-O2','-Wall','-Wextra','-Werror',
         '-fsanitize=thread' if a.thread_sanitizer else '-fsanitize=undefined,bounds',
         '-I',root/'runtime/native/gc',out/'test.c',root/'runtime/native/gc/root-probe.c','-o',out/'test'])
    r=run([out/'test'])
    assert 'COUNTEREXAMPLE: three snapshot cycles retain an unrooted conditional owner' in r.stdout
    assert 'synchronous oracle expires the same owner and reclaims all memory' in r.stdout
    report['passed']=True
    print(r.stdout,end='')
finally:
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
