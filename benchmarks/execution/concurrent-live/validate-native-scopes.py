"""Validate bounded synchronous native scopes without enabling compiler admission."""
import argparse,gzip,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd,negative=False):
    cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90)
    r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
    if negative: assert x.returncode!=0 and '!experiment_gc_live_active(&c)' in x.stderr,x.stderr
    else:x.check_returncode()
try:
    source=out/'native-gc.c';run(['python3',base/'native-scopes.py','--output',source])
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
    common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',root/'runtime/native/gc','-I',root/'runtime/native/memory','-I',base.parent/'concurrent-mutation-gate',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/memory/text-arena.c']
    for paced in [False,True]:
        for san in [False,True]:
            flags=(['-DNEOCLR_GC_PACED'] if paced else [])+(['-fsanitize=undefined,bounds','-fno-sanitize-recover=all'] if san else [])
            exe=out/f'scopes-{paced}-{san}'
            run(common+flags+[source,base/'native-scopes-test.c','-o',exe]);run([exe])
    # Existing collector lifetimes and previously generated allocation consumers
    # must still work when no fallback scope is active.
    for test in [root/'docs/experiments/aot-console/native-gc-test.c',root/'docs/experiments/aot-console/native-gc-seed-test.c',base/'automatic-test.c']:
        exe=out/test.stem
        run(common+(['-DNEOCLR_GC_PACED'] if test.name=='automatic-test.c' else [])+['-fsanitize=undefined,bounds','-fno-sanitize-recover=all',source,test,'-o',exe]);run([exe])
    store=base.parent/'concurrent-store';obj=out/'primitive-arrays.o';obj.write_bytes(gzip.decompress((store/'primitive-arrays.o.gz').read_bytes()))
    exe=out/'managed-regression'
    run(common+['-DNEOCLR_GC_PACED','-fsanitize=undefined,bounds','-fno-sanitize-recover=all',source,store/'primitive-arrays-consumer.c',obj,'-DMANAGED_MAIN','-o',exe]);run([exe])
    negative=out/'negative-gc.c';text=source.read_text()
    needle='''    if (candidate_active) {
        candidate_cancel();
        native_scope_counts[1]++;
    }'''
    assert text.count(needle)==1
    negative.write_text(text.replace(needle,'    /* Negative control: enter without cancelling marker. */'))
    exe=out/'negative';run(common+['-DNEOCLR_GC_PACED',negative,base/'native-scopes-test.c','-o',exe]);run([exe],negative=True)
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),base/'native-scopes.py',base/'native-scopes-test.c',base/'automatic.py',source,negative,obj,store/'primitive-arrays-consumer.c',root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/memory/text-arena.c']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
