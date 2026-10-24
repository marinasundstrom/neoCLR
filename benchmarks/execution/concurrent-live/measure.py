import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--sparse-index', action='store_true')
parser.add_argument('--batch-comparison', action='store_true')
parser.add_argument('--gate-profile', action='store_true')
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
    if args.gate_profile: cc += ['-DNEOCLR_LIVE_GATE_PROFILE']
    exe=out/'benchmark'
    command=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-I',base.parent/'concurrent-mutation-gate',base.parent/'concurrent-mutation-gate/gate.c','-I',root/'runtime/native/gc',base/'benchmark.c',root/'runtime/native/gc/root-probe.c',source,'-o',exe]
    run(command)
    batched=out/'batched'
    if args.batch_comparison:
        batch_command=command.copy();batch_command[-1]=batched
        batch_command+=['-DNEOCLR_LIVE_BATCH']
        run(batch_command)
    modes=['sync','concurrent','batched','batched','concurrent','sync'] if args.batch_comparison else ['sync','concurrent','concurrent','sync']
    for mode in modes:
        argument='concurrent' if mode=='batched' else mode
        words=run([batched if mode=='batched' else exe,argument]).split();assert words[0]==argument
        wall,cpu,peak,checksum=map(int,words[1:])
        r['trials'].append(dict(mode=mode,wallNanoseconds=wall,processCpuNanoseconds=cpu,peakArenaBytes=peak,checksum=checksum))
    assert len({row['checksum'] for row in r['trials']})==1
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source,base/'benchmark.c',base/'measure.py',base/'scan.h',base/'batch.h',base/'generate.py',base.parent/'concurrent-mutation-gate/gate.c',base.parent/'concurrent-mutation-gate/gate.h']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
