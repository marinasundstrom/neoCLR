"""Check active-phase gating, argument/result preservation and part accounting."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

def module(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+'.py'))
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
parts=module('collection-parts');finish=module('finish-phases')
source=r'''
#include <assert.h>
typedef int neoclr_aot_text_arena;
typedef int neoclr_aot_context;
typedef int neoclr_probe_frame;
typedef int block_index;
typedef int block;
static struct { struct { unsigned collections; } statistics; } gc_state;
static int build_index(neoclr_aot_text_arena *a, block_index *index) {
 fake_wall+=11;fake_cpu+=7;return *a+*index;
}
static void mark_frames(neoclr_aot_context *context, block_index *index, const neoclr_probe_frame *head, block **pending) {
 fake_wall+=13;fake_cpu+=9;**pending=*context+*index+*head;
}
static int candidate_seed(neoclr_aot_context *context, block_index *index, block **pending) {
 fake_wall+=7;fake_cpu+=5;
    size_t previous = 0;
    assert(*context+*index+**pending==19 && previous==0);
    fake_wall+=10;fake_cpu+=6;
    return 0;
}
static void sweep(void) {
    /* Sweep and coalesce in one forward walk. */
    fake_wall+=19;fake_cpu+=13;
    gc_state.statistics.collections++;
}
int main(void) {
 int context=3,index=4,head=5,value=0;int *pending=&value;
 for(unsigned active=0;active<2;active++) {
  finish_parts_active=active;
  assert(build_index(&context,&index)==7);
  mark_frames(&context,&index,&head,&pending);assert(value==12);
  assert(candidate_seed(&context,&index,&pending)==0);sweep();
  for(unsigned i=0;i<6;i++)assert(finish_parts[i].count==active);
 }
 finish_parts_active=0;
 uint64_t walls[]={11,13,17,19,7,10},cpus[]={7,9,11,13,5,6};
 for(unsigned i=0;i<6;i++)assert(finish_parts[i].wall==walls[i] && finish_parts[i].cpu==cpus[i]);
 assert(gc_state.statistics.collections==2);
 experiment_collection_parts_report();return 0;
}
'''
for invalid in [source+source,source.replace('gc_state.statistics.collections++;','')]:
    try:parts.instrument(invalid)
    except AssertionError:pass
    else:raise AssertionError('invalid source accepted')
prefix='''static unsigned long long fake_wall,fake_cpu;
#define FINISH_WALL_NOW() fake_wall
#define FINISH_CPU_NOW() fake_cpu
'''
root=Path(__file__).resolve().parents[3];recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
with tempfile.TemporaryDirectory() as directory:
    c=Path(directory)/'test.c';exe=Path(directory)/'test'
    c.write_text(prefix+finish.HEADER+parts.HEADER+parts.instrument(source))
    subprocess.run(recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-unused-function','-fsanitize=undefined,bounds','-fno-sanitize-recover=all',str(c),'-o',str(exe)],check=True)
    x=subprocess.run([str(exe)],capture_output=True,text=True,check=True)
    assert len(x.stderr.splitlines())==6
print('PASS: final-phase gating, argument/result preservation, paired accounting and source guards; UB/bounds sanitizer')

summary=module('summarize-collection-parts')
final=dict(count=1,wallNs=60,cpuNs=40)
rows,remainder=summary.parse(x.stderr,final)
assert remainder==dict(wallNs=0,cpuNs=0)
for invalid in [x.stderr+x.stderr, '\n'.join(x.stderr.splitlines()[1:]),x.stderr.replace('COLLECTION_PART 0 1','COLLECTION_PART 0 2')]:
    try:summary.parse(invalid,final)
    except AssertionError:pass
    else:raise AssertionError('invalid part report accepted')
try:summary.parse(x.stderr,dict(count=1,wallNs=59,cpuNs=40))
except AssertionError:pass
else:raise AssertionError('overlapping/excess attribution accepted')
print('PASS: part parser and negative accounting controls')

seed, remainder=summary.parse_seed(x.stderr,rows[2])
assert seed['validation']['wallNs']==7 and seed['application']['cpuNs']==6
assert remainder==dict(wallNs=0,cpuNs=0)
for bad in [x.stderr+x.stderr, x.stderr.replace('SEED_PART 1','SEED_PART 0'),
            '\n'.join(x.stderr.splitlines()[:-1]),
            x.stderr.replace('SEED_PART 0 1','SEED_PART 0 2')]:
    try:summary.parse_seed(bad,rows[2])
    except AssertionError:pass
    else:raise AssertionError('invalid nested seed accounting accepted')
try:summary.parse_seed(x.stderr,dict(count=1,wallNs=16,cpuNs=11))
except AssertionError:pass
else:raise AssertionError('excess seed accounting accepted')
print('PASS: nested seed parser and negative accounting controls')

assert summary.parse_seed(x.stderr.splitlines(),rows[2])==summary.parse_seed(x.stderr,rows[2])
