"""Exercise real loopback transfer/root lifetimes across active live-GC epochs."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parent;root=base.parents[2];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd):
    cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90)
    r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
    original=root/'docs/experiments/aot-console/socket-transfer-test.c';s=original.read_text()
    assert s.count('neoclr_socket_scope scope;')==1
    s=s.replace('neoclr_socket_scope scope;','CHECK(!neoclr_gc_heap_acquire_v1(&c.text));\n    neoclr_socket_scope scope;')
    s=s.replace('neoclr_gc_collect_v1(&c, NULL)','collect_and_begin(&c)')
    assert s.count('    return 0;\n}')==1
    s=s.replace('    return 0;\n}', '''    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(begins>=4);
    printf("PASS socket transfer: %u collection/begin points, zero retained heap\\n",begins);
    return 0;
}''')
    s=s.replace('int main(void) {','''#include <stdio.h>
#ifdef LIVE_GC
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
#endif
static unsigned begins;
static int collect_and_begin(neoclr_aot_context *c) {
    if(neoclr_gc_collect_v1(c,NULL)) return 3;
#ifdef LIVE_GC
    if(experiment_gc_begin(c,NULL,NULL,0) || !experiment_gc_live_active(c)) return 3;
#endif
    ++begins;return 0;
}
int main(void) {''')
    test=out/'socket-transfer.c';test.write_text(s)
    live=out/'native-gc.c';run(['python3',base/'automatic.py','--output',live])
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
    network=root/'docs/experiments/aot-console'
    common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-I',root/'runtime/native/gc','-I',network,'-I',base.parent/'concurrent-mutation-gate',test,network/'socket-listener.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/memory/text-arena.c']
    for candidate in [False,True]:
        for paced in [False,True]:
            for sanitized in [False,True]:
                flags=(['-DNEOCLR_GC_PACED'] if paced else [])+(['-fsanitize=undefined,bounds','-fno-sanitize-recover=all'] if sanitized else [])
                source=[live,base.parent/'concurrent-mutation-gate/gate.c','-DLIVE_GC','-DNEOCLR_SHADOW_SPARSE_INDEX'] if candidate else [root/'runtime/native/gc/native-gc.c']
                exe=out/f'transfer-{candidate}-{paced}-{sanitized}'
                run(common+flags+source+['-o',exe]);run([exe])
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),original,test,live,base/'automatic.py',network/'socket-listener.c',root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/memory/text-arena.c']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
