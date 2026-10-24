"""Qualify native byte-buffer operations against sync and live collectors."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent;root=base.parents[2]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd,negative=False):
    cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90)
    r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
    if negative:
        assert x.returncode != 0 and 'after[1]==before[1]' in x.stderr, x.stderr
    else: x.check_returncode()
try:
    source=out/'native-gc.c'
    run(['python3',base.parent/'concurrent-live/automatic.py','--output',source])
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cc=recipe[:recipe.index('-std=c11')]
    common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-I',root/'runtime/native/gc','-I',root/'runtime/native/memory','-I',base.parent/'concurrent-mutation-gate',root/'runtime/native/gc/root-probe.c',root/'runtime/native/memory/text-arena.c']
    for live in [False,True]:
        collector=[source,base.parent/'concurrent-mutation-gate/gate.c','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-DLIVE'] if live else [root/'runtime/native/gc/native-gc.c']
        for sanitized in [False,True]:
            flags=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all'] if sanitized else []
            for test in [base/'byte-views-consumer.c',root/'runtime/native/memory/memory-views-test.c']:
                exe=out/f'{test.stem}-{live}-{sanitized}'
                run(common+collector+flags+[test,'-o',exe]);run([exe])
    negative=out/'negative-gc.c'
    text=source.read_text();needle='static int shadow_scan(shadow_snapshot *s, const block *b) {'
    assert text.count(needle)==1
    text=text.replace(needle,needle+'\n    if (b->kind == NEOCLR_GC_BYTES) return live_words(s,b,16,b->bytes & ~UINT64_C(7));')
    negative.write_text(text)
    exe=out/'negative'
    run(common+[negative,base.parent/'concurrent-mutation-gate/gate.c','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-DLIVE',base/'byte-views-consumer.c','-o',exe]);run([exe],negative=True)
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),base/'byte-views-consumer.c',source,negative,root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/memory/text-arena.c',root/'runtime/native/memory/memory-views.h',root/'runtime/native/memory/memory-views-test.c',base.parent/'concurrent-live/scan.h',base.parent/'concurrent-live/automatic.py']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
