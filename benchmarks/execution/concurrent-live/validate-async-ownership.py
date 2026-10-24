"""Compare real fiber ownership/lifetime transitions under synchronous and live GC."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent;root=base.parents[2]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[])
def run(cmd):
    cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90)
    r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
    x.check_returncode()
try:
    original=root/'docs/experiments/runtime-async/native-owned-body-test.c'
    s=original.read_text()
    def replace(old,new):
        global s
        assert s.count(old)==1,(old,s.count(old))
        s=s.replace(old,new)
    replace('static uint64_t signal_id;', '''
#ifdef LIVE_GC
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern int32_t experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
#endif
static unsigned epochs, mutations;
static int begin(neoclr_aot_context *c) {
#ifdef LIVE_GC
    if (experiment_gc_live_active(c)) return 0;
    int status=experiment_gc_begin(c,neoclr_root_probe_head_v1(),NULL,0);
    if(status || !experiment_gc_live_active(c)) return 1;
#else
    (void)c;
#endif
    ++epochs;return 0;
}
static int mutate(neoclr_aot_context *c,uint64_t *slot,uint64_t value) {
#ifdef LIVE_GC
    if(!experiment_gc_live_active(c)) return 1;
    int status=experiment_gc_store(c,slot,value,8);
    if(status) return status;
#else
    (void)c;*slot=value;
#endif
    ++mutations;return 0;
}
static uint64_t signal_id;''')
    replace('++entered;\n    int32_t result;', '''++entered;
    CHECK(!begin(c));
    int32_t result;''')
    replace('CHECK(result && local[1] == 42);', '''CHECK(result && local[1] == 42);
    CHECK(local[3] && ((uint64_t *)(uintptr_t)local[3])[1]==99);
    CHECK(!mutate(c,local+1,43));
    CHECK(!mutate(c,local+1,42));''')
    replace('int main(int argc, char **argv) {', '''static void *allocate_payload(neoclr_aot_context *c) {
    void *p=NULL;
    return neoclr_gc_allocate_v1(&c->text,32,NEOCLR_GC_OBJECT,&p) ? NULL : p;
}
int main(int argc, char **argv) {''')
    assert s.count('payload = allocate(&c);')==1 and s.count('payload=allocate(&c);')==1
    s=s.replace('payload = allocate(&c);','payload = allocate_payload(&c);').replace('payload=allocate(&c);','payload=allocate_payload(&c);')
    replace('for (int i = 0; i < 320; ++i) {','''CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    for (int i = 0; i < 320; ++i) {''')
    replace('if (managed) {\n            CHECK(!neoclr_runtime_async_startmanaged_v1', '''CHECK(!begin(&c));
        if (managed) {
            CHECK(!neoclr_runtime_async_startmanaged_v1''')
    replace('CHECK(entered == i + 1 && resumed == i / 2);','''CHECK(entered == i + 1 && resumed == i / 2);
        CHECK(!neoclr_root_probe_head_v1());
        /* A new child becomes reachable only through the suspended local's object. */
        uint64_t *child=allocate(&c);CHECK(child);child[1]=99;
        CHECK(!mutate(&c,payload+3,(uintptr_t)child));''')
    replace('CHECK(payload[1] == 42);', '''CHECK(payload[1] == 42 && ((uint64_t *)(uintptr_t)payload[3])[1]==99);
        CHECK(!begin(&c));''')
    replace('CHECK(!neoclr_runtime_async_pump_v1(&c, invoke, drain));\n            CHECK(resumed == (i + 1) / 2);', '''CHECK(!begin(&c));
            CHECK(!neoclr_runtime_async_pump_v1(&c, invoke, drain));
            CHECK(resumed == (i + 1) / 2);''')
    replace('neoclr_async_statistics stats = neoclr_runtime_async_statistics_v1();','''CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(epochs>=640 && mutations==640);
    printf("ownership probe: beginPoints=%u mutations=%u\\n",epochs,mutations);
    neoclr_async_statistics stats = neoclr_runtime_async_statistics_v1();''')
    test=out/'async-ownership.c';test.write_text(s)
    live=out/'native-gc.c';run(['python3',base/'automatic.py','--output',live])
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
    common=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-I',root/'runtime/native/gc','-I',root/'runtime/native/async','-I',root/'docs/experiments/aot-console','-I',base.parent/'concurrent-mutation-gate',test,root/'runtime/native/async/native-runtime.c',root/'runtime/native/gc/root-probe.c',root/'docs/experiments/aot-console/native-stack.c']
    for candidate in [False,True]:
        for paced in [False,True]:
            for sanitized in [False,True]:
                flags=(['-DNEOCLR_GC_PACED'] if paced else [])+(['-fsanitize=undefined,bounds','-fno-sanitize-recover=all'] if sanitized else [])
                source=[live,base.parent/'concurrent-mutation-gate/gate.c','-DLIVE_GC','-DNEOCLR_SHADOW_SPARSE_INDEX'] if candidate else [root/'runtime/native/gc/native-gc.c']
                exe=out/f'ownership-{candidate}-{paced}-{sanitized}'
                run(common+flags+source+['-o',exe]);run([exe])
    r['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),original,test,live,base/'automatic.py',root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/async/native-runtime.c']}
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
