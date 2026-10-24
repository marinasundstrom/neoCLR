"""Paired scalar-store lookup costs; lifecycle cost reported separately."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent;root=base.parents[2]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],runs=[])
def run(cmd):
    cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
    r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
    x.check_returncode();return x
try:
    source=out/'native-gc.c'
    run(['python3',base/'generate.py','--output',source])
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cc=recipe[:recipe.index('-std=c11')]
    for variant in ['linear','indexed']:
        cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base/'store-benchmark.c',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',source,'-o',out/variant]
        if variant=='linear':cmd+=['-DNEOCLR_LIVE_LINEAR_STORE']
        run(cmd)
    for count in [1024,16384]:
        for variant in ['linear','indexed','indexed','linear']:
            x=run([out/variant,count]);row=json.loads(x.stdout)
            assert row['stores']==3200 and row['objects']==count
            row['variant']=variant;r['runs'].append(row)
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source,base/'store.h',base/'generate.py',base/'store-benchmark.c',Path(__file__),base.parent/'concurrent-mutation-gate/gate.c']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
