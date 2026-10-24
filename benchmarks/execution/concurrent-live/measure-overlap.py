"""Synchronous, idle-marker and overlapping-marker stores with phase distributions."""
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
    for variant in ['sync','candidate']:
        cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate',base/'store-overlap.c',root/'runtime/native/gc/root-probe.c']
        if variant=='sync':cmd+=['-DSTORE_SYNC',root/'runtime/native/gc/native-gc.c']
        else:cmd+=[base.parent/'concurrent-mutation-gate/gate.c',source]
        run(cmd+['-o',out/variant])
        run(cmd+['-fsanitize=undefined,bounds','-fno-sanitize-recover=all','-o',out/(variant+'-san')])
    # Validation precedes timing runs; no builds run concurrently with measurements.
    for mode in ['sync','idle','overlap']:
        run([out/('sync-san' if mode=='sync' else 'candidate-san'),1024,mode,100])
    for count in [1024,16384]:
        for mode in ['sync','idle','overlap','overlap','idle','sync']:
            x=run([out/('sync' if mode=='sync' else 'candidate'),count,mode,1000]);row=json.loads(x.stdout)
            assert row['stores']==32000 and row['objects']==count and row['cycles']==1000
            for phase in ['beginNs','finishNs','writeWindowNs']:
                values=sorted(row[phase+'Samples']);assert len(values)==1000
                assert row[phase]['max']==values[-1]
                for q in [50,95,99]:assert row[phase]['p'+str(q)]==values[999*q//100]
            r['runs'].append(row)
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source,base/'store.h',base/'generate.py',base/'store-overlap.c',Path(__file__),base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
