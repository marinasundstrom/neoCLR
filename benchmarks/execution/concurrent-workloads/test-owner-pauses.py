"""Deterministic nested timing, histogram boundary and owner-TLS checks."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import json

path = Path(__file__).with_name('owner-pauses.py')
spec = importlib.util.spec_from_file_location('pauses', path)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
fixture = r'''
#include <assert.h>
#include <pthread.h>
#include <string.h>
typedef int neoclr_aot_context;
typedef int neoclr_probe_frame;
typedef int neoclr_gc_conditional_root;
typedef int mutation_patch;
int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
static void candidate_cancel(void);
int32_t neoclr_gc_collect_conditional_v1(neoclr_aot_context *context, const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {
 return context ? experiment_gc_begin(context,head,conditional,count) : 3;
}
int32_t experiment_gc_begin(neoclr_aot_context *context, const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {
 return experiment_gc_finish(context,head,conditional,count);
}
int32_t experiment_gc_finish(neoclr_aot_context *context, const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {
 candidate_cancel(); return 0;
}
static void candidate_cancel(void) {
}
static int live_write_known(void *payload, uint64_t bytes, uint32_t kind, const mutation_patch *patches, size_t count) {
 return 0;
}
static void *other_owner(void *unused) {
 assert(owner_pauses[5].count==0);
 owner_pause_record(5,99,0);
 assert(owner_pauses[5].count==1);
 return 0;
}
int main(void) {
 int context=0;
 assert(!neoclr_gc_collect_conditional_v1(&context,0,0,0));
 assert(neoclr_gc_collect_conditional_v1(0,0,0,0)==3);
 assert(!live_write_known(0,0,0,0,0));
 candidate_cancel();
 assert(owner_pauses[0].count==2 && owner_pauses[0].total==64 && owner_pauses[0].maximum==56 && owner_pauses[0].failures==1);
 assert(owner_pauses[1].total==40 && owner_pauses[2].total==24);
 assert(owner_pauses[3].count==2 && owner_pauses[3].total==16);
 assert(owner_pauses[4].total==8);
 assert(owner_pauses[5].count==4 && owner_pauses[5].total==80 && owner_pauses[5].maximum==56 && owner_pauses[5].failures==1);
 assert(owner_pauses[6].count==3 && owner_pauses[6].total==72);
 pthread_t thread;
 assert(!pthread_create(&thread,0,other_owner,0)); assert(!pthread_join(thread,0));
 assert(owner_pauses[5].count==4);
 experiment_owner_pauses_report();
 memset(owner_pauses,0,sizeof(owner_pauses));
 uint64_t values[]={0,1,2,7,8,UINT64_C(1)<<63};
 for(unsigned i=0;i<6;i++) owner_pause_record(0,values[i],0);
 assert(owner_pauses[0].bins[0]==2 && owner_pauses[0].bins[1]==1 && owner_pauses[0].bins[2]==1 && owner_pauses[0].bins[3]==1 && owner_pauses[0].bins[63]==1);
 return 0;
}
'''
source = '#define OWNER_PAUSE_NOW() fake_now()\nstatic unsigned long long tick;\nstatic unsigned long long fake_now(void) { unsigned long long now=tick; tick+=8; return now; }\n' + m.instrument(fixture, True)
for invalid in [fixture + fixture, fixture.replace('static void candidate_cancel(void) {','static void other_cancel(void) {')]:
    try:
        m.instrument(invalid, True)
    except AssertionError:
        pass
    else:
        raise AssertionError('ambiguous/missing definition accepted')
with tempfile.TemporaryDirectory() as directory:
    c = Path(directory)/'test.c'; exe=Path(directory)/'test'
    c.write_text(source)
    root=Path(__file__).resolve().parents[3]
    recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cc=recipe[:recipe.index('-std=c11')]
    subprocess.run(cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-unused-parameter','-pthread','-fsanitize=undefined,bounds','-fno-sanitize-recover=all',str(c),'-o',str(exe)],check=True)
    x=subprocess.run([str(exe)],capture_output=True,text=True,check=True)
    lines=x.stderr.splitlines()
    assert len(lines)==7
    for line in lines:
        numbers=list(map(int,line.split()[1:])); assert len(numbers)==69
        _,count,total,maximum,failures,*bins=numbers
        assert sum(bins)==count and failures<=count and maximum<=total
spec = importlib.util.spec_from_file_location('summary', path.with_name('summarize-owner-pauses.py'))
summary = importlib.util.module_from_spec(spec)
spec.loader.exec_module(summary)
parsed = summary.parse(x.stderr)
assert parsed[6]['count'] == 3
assert summary.quantile(parsed[6]['bins'], 99) == dict(lowerNs=32, upperExclusiveNs=64)
assert summary.quantile([0]*64, 99) is None
assert summary.quantile([0]*63+[1], 99) == dict(lowerNs=1<<63, upperExclusiveNs=1<<64)
for invalid in [x.stderr + lines[0] + '\n', '\n'.join(lines[:-1]), x.stderr.replace('OWNER_PAUSE 0 2 64', 'OWNER_PAUSE 0 2 1'), x.stderr.replace('OWNER_PAUSE 0 2 64', 'OWNER_PAUSE 0 3 64')]:
    try:
        summary.parse(invalid)
    except AssertionError:
        pass
    else:
        raise AssertionError('invalid histogram accepted')
print('PASS: nested inclusive/outer timing, failure counts, bucket boundaries, TLS isolation, definition guards, summary/negative controls; UB/bounds sanitizer')
