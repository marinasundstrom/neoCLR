"""Exercise a retained same-thread store-state borrow across native lifecycle tests."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import signal
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
base=Path(__file__).resolve().parent;root=base.parents[2]
r=dict(passed=False,commands=[],inputs={})
def run(cmd):
    cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
    r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
    source=out/'native-gc.c';run(['python3',base/'generate.py','--output',source])
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    common=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-DNEOCLR_SHADOW_SPARSE_INDEX','-DNEOCLR_LIVE_ALLOCATION_INDEX','-DNEOCLR_LIVE_EXACT_INDEX','-DNEOCLR_LIVE_INLINE_BASE','-DNEOCLR_LIVE_STORE_HANDLE','-I',root/'runtime/native/gc','-I',base.parent/'concurrent-mutation-gate']
    for case in ['store-test','exact-store-test']:
        original=base/(case+'.c');fixture=out/(case+'.c')
        body=original.read_text();end=body.rfind('}')
        included=out/(case+'-body.c');included.write_text(body[:end]+'    return 0;\n'+body[end:])
        fixture.write_text('''#include "native-gc.h"
#include <assert.h>
extern void *experiment_gc_store_state_borrow(void);
extern int experiment_gc_store_borrowed(void *,neoclr_aot_context *,void *,uint64_t,uint32_t);
static _Thread_local void *borrowed_state;
static _Thread_local unsigned borrowed_calls,acquisitions;
static int retained_store(neoclr_aot_context *c,void *p,uint64_t bits,uint32_t width) {
    if (!borrowed_state) { borrowed_state=experiment_gc_store_state_borrow(); acquisitions++; }
    assert(borrowed_state && acquisitions==1);
    borrowed_calls++;
    return experiment_gc_store_borrowed(borrowed_state,c,p,bits,width);
}
#define experiment_gc_store retained_store
#define main fixture_main
#include "'''+str(included)+'''"
#undef main
int main(void) {
    assert(experiment_gc_store_borrowed(NULL,NULL,NULL,0,8)==3);
    int status=fixture_main();
    assert(!status && borrowed_calls>0 && acquisitions==1);
    puts("PASS retained store borrow: one acquisition on main owner, current validation preserved");
    return status;
}
''')
        for sanitized in [False,True]:
            exe=out/f'{case}-{sanitized}'
            cmd=common+[fixture,base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',source,'-o',exe]
            if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
            run(cmd);run([exe])
        r['inputs'][str(original)]=hashlib.sha256(original.read_bytes()).hexdigest()
    # Losing current heap ownership must fail even with a retained candidate handle.
    text=source.read_text();needle='owned_heap.arena != &context->text ||'
    assert text.count(needle)==1
    broken=out/'broken-owner.c';broken.write_text(text.replace(needle,''))
    exe=out/'negative-owner'
    run(common+[out/'store-test.c',base.parent/'concurrent-mutation-gate/gate.c',root/'runtime/native/gc/root-probe.c',broken,'-o',exe])
    x=subprocess.run([exe],capture_output=True,text=True,timeout=30)
    r['negative']=dict(exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,expectedSignal='SIGABRT')
    assert x.returncode==-signal.SIGABRT
    r['passed']=True
finally:
    for f in [source,base/'store.h',base/'generate.py',Path(__file__)]:r['inputs'][str(f)]=hashlib.sha256(f.read_bytes()).hexdigest()
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
