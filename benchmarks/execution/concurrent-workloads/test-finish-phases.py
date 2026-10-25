"""Validate paired finish CPU/wall accounting and same-event outlier attribution."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

path=Path(__file__).with_name('finish-phases.py')
spec=importlib.util.spec_from_file_location('finish',path)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
prefix='''static unsigned long long fake_wall,fake_cpu;
#define FINISH_WALL_NOW() fake_wall
#define FINISH_CPU_NOW() fake_cpu
'''
body=r'''
#include <assert.h>
static void phase(unsigned i, uint64_t w, uint64_t c, finish_stamp *event) {
 finish_stamp start=finish_start(); fake_wall+=w; fake_cpu+=c; finish_end(i,start,event);
}
int main(void) {
 finish_stamp event[6]={{0,0}}, start=finish_start();
 phase(0,100,5,event); phase(1,20,20,event); phase(3,40,40,event); phase(4,10,5,event);
 finish_end(5,start,event); finish_event(event,59);
 assert(finish_rows[5].wall==170 && finish_rows[5].cpu==70);
 finish_stamp next[6]={{0,0}}; start=finish_start();
 phase(0,1,1,next); phase(1,1,1,next); phase(2,300,2,next); phase(3,5,5,next); phase(4,1,1,next);
 finish_end(5,start,next); finish_event(next,63);
 assert(finish_rows[5].wall==478 && finish_rows[5].cpu==80);
 assert(finish_rows[0].wall_max==100 && finish_slowest[0].wall==1);
 assert(finish_slowest[2].wall==300 && finish_slowest[5].wall==308);
 assert(finish_slowest[5].cpu==10 && finish_slowest_mask==63);
 assert(finish_rows[2].count==1 && finish_rows[2].bins[8]==1);
 experiment_finish_phases_report();
 return 0;
}
'''
root=Path(__file__).resolve().parents[3]
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
with tempfile.TemporaryDirectory() as directory:
 c=Path(directory)/'test.c';exe=Path(directory)/'test';c.write_text(prefix+m.HEADER+body)
 subprocess.run(recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-fsanitize=undefined,bounds','-fno-sanitize-recover=all',str(c),'-o',str(exe)],check=True)
 result=subprocess.run([str(exe)],capture_output=True,text=True,check=True)
 assert len(result.stderr.splitlines())==7
 print('PASS: paired owner CPU/wall, optional phase count, histogram, same-event maximum; UB/bounds sanitizer')

spec=importlib.util.spec_from_file_location('summary',path.with_name('summarize-finish-phases.py'))
summary=importlib.util.module_from_spec(spec);spec.loader.exec_module(summary)
rows,event=summary.parse(result.stderr)
assert event['phases'][0]['wallNs']==1 and rows[0]['maximumWallNs']==100
for invalid in [result.stderr+result.stderr, '\n'.join(result.stderr.splitlines()[:-1]), result.stderr.replace('FINISH_PHASE 0 2', 'FINISH_PHASE 0 3'), result.stderr.replace('FINISH_MAX 63 1 1', 'FINISH_MAX 63 101 1')]:
    try: summary.parse(invalid)
    except AssertionError: pass
    else: raise AssertionError('invalid attribution accepted')
for invalid in ['', 'int32_t experiment_gc_finish(neoclr_aot_context *context, int count) {\n}\n'*2]:
    try: m.instrument(invalid)
    except AssertionError: pass
    else: raise AssertionError('missing/duplicate finish accepted')
print('PASS: parser/attribution negative controls and definition guards')
