import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--sparse-index', action='store_true')
args = parser.parse_args()
base = Path(__file__).resolve().parent
root = base.parents[2]
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
r = dict(passed=False, commands=[], trials=[])
def run(command):
    command=list(map(str,command));x=subprocess.run(command,capture_output=True,text=True,timeout=120)
    r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
    x.check_returncode();return x.stdout
try:
    source=out/'native-gc.c'
    run(['python3',base/'generate.py','--output',source])
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cc=recipe[:recipe.index('-std=c11')]
    if args.sparse_index: cc += ['-DNEOCLR_SHADOW_SPARSE_INDEX']
    exe=out/'benchmark'
    run(cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-I',base.parent/'concurrent-mutation-gate',base.parent/'concurrent-mutation-gate/gate.c','-I',root/'runtime/native/gc',base/'benchmark.c',root/'runtime/native/gc/root-probe.c',source,'-o',exe])
    for mode in ['sync','concurrent','concurrent','sync']:
        words=run([exe,mode]).split();assert words[0]==mode
        wall,cpu,peak,checksum=map(int,words[1:])
        r['trials'].append(dict(mode=mode,wallNanoseconds=wall,processCpuNanoseconds=cpu,peakArenaBytes=peak,checksum=checksum))
    assert len({row['checksum'] for row in r['trials']})==1
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source,base/'benchmark.c',base/'measure.py',base/'scan.h',base/'generate.py',base.parent/'concurrent-mutation-gate/gate.c',base.parent/'concurrent-mutation-gate/gate.h']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
