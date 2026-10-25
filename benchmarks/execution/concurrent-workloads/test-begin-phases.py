"""Deterministic wrapper/accounting checks plus missing/corrupt data controls."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
base=Path(__file__).resolve().parent

def load(name):
    spec=importlib.util.spec_from_file_location(name,base/(name+'.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
m=load('begin-phases');summary=load('summarize-begin-phases')
prefix='''static unsigned long long fake_wall,fake_cpu;
#define BEGIN_WALL_NOW() fake_wall
#define BEGIN_CPU_NOW() fake_cpu
'''
source=r'''
#include <assert.h>
typedef struct { int fail; } neoclr_aot_context;
typedef int neoclr_probe_frame;
typedef int neoclr_gc_conditional_root;
static void shadow_begin(neoclr_aot_context *context,
    const neoclr_gc_conditional_root *conditional, uint32_t count) {
    assert(context && conditional && count==7); fake_wall+=30;fake_cpu+=10;
}
int32_t experiment_gc_begin(neoclr_aot_context *context, const neoclr_probe_frame *head,
    neoclr_gc_conditional_root *conditional, uint32_t count) {
    assert(head && *head==42);fake_wall+=10;fake_cpu+=3;
    if (context->fail) return 3;
    shadow_begin(context,conditional,count);fake_wall+=20;fake_cpu+=7;return 0;
}
void experiment_gc_time_counts(uint64_t *out) {out[0]=17;}
void experiment_gc_live_counts(uint64_t *out) {for(unsigned i=0;i<5;i++)out[i]=i<3?100+i:i==3?2:0;}
'''
main=r'''
int main(void) {
    neoclr_aot_context context={0};int head=42,conditional=1;
    for(unsigned i=0;i<2;i++)assert(!experiment_gc_begin(&context,&head,&conditional,7));
    assert(begin_rows[0].wall==120 && begin_rows[0].cpu==40);
    assert(begin_rows[1].wall==60 && begin_rows[1].cpu==20);
    experiment_begin_phases_report();
    context.fail=1;assert(experiment_gc_begin(&context,&head,&conditional,7)==3);
    assert(begin_rows[0].count==3 && begin_rows[0].failures==1 && begin_rows[1].count==2);
    return 0;
}
'''
root=base.parents[2];recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
with tempfile.TemporaryDirectory() as temp:
    c=Path(temp)/'test.c';exe=Path(temp)/'test';c.write_text(prefix+m.instrument(source)+main)
    subprocess.run(recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-fsanitize=undefined,bounds','-fno-sanitize-recover=all',c,'-o',exe],check=True)
    result=subprocess.run([exe],capture_output=True,text=True,check=True)
rows=summary.parse(result.stderr,[2,1,1,0,0,0]);assert rows['beginRemainder']['cpuNs']==20 and rows['marker']['cpuNs']==17
for bad in [result.stderr+result.stderr,result.stderr.replace('MARKER_WORK','OMITTED'),result.stderr.replace('BEGIN_PHASE 1 2 60 20','BEGIN_PHASE 1 2 60 50'),result.stderr.replace('102 2 0','102 1 0')]:
    try:summary.parse(bad,[2,1,1,0,0,0])
    except AssertionError:pass
    else:raise AssertionError('invalid counters accepted')
for bad in ['',source+source,m.instrument(source)]:
    try:m.instrument(bad)
    except AssertionError:pass
    else:raise AssertionError('invalid instrumentation accepted')
print('PASS deterministic CPU/wall nesting, argument/result preservation, failures, joined marker counts and negative controls; UB/bounds')
