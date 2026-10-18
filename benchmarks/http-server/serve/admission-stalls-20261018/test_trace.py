#!/usr/bin/env python3
"""Check diagnostic wrapper forwarding, interval recording and clock origins."""
import json, os, subprocess, tempfile
from pathlib import Path
here = Path(__file__).resolve().parent
root = here.parents[3]
with tempfile.TemporaryDirectory() as directory:
    out = Path(directory)
    (out/'stub.c').write_text(r'''#include "native-gc.h"
#include <assert.h>
#include <time.h>
void neoclr_scheduler_wait_v1(neoclr_aot_context *);
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *,uint64_t *,const neoclr_probe_frame *);
void trace_async_pause(void);
static void pause_test(void){struct timespec t={0,2000000};nanosleep(&t,0);}
int32_t trace_actual_collect(neoclr_aot_context *c,const neoclr_probe_frame *f){assert(!c&&!f);pause_test();return 7;}
void trace_actual_wait(neoclr_aot_context *c){assert(!c);pause_test();}
int32_t trace_actual_acquire(neoclr_aot_context *c,uint64_t *d,const neoclr_probe_frame *f){assert(!c&&!f);*d=42;pause_test();return 9;}
int main(void){uint64_t d=0;assert(neoclr_gc_collect_v1(0,0)==7);neoclr_scheduler_wait_v1(0);trace_async_pause();assert(neoclr_scheduler_acquire_v1(0,&d,0)==9&&d==42);}
''')
    subprocess.run(['clang','-I',str(root/'runtime/native/gc'),'-I',str(root/'docs/experiments/aot-console'),str(here/'trace.c'),str(out/'stub.c'),'-o',str(out/'test')],check=True)
    subprocess.run([str(out/'test')],env=dict(os.environ,NEOCLR_STALL_TRACE=str(out/'trace.json')),check=True)
    trace=json.loads((out/'trace.json').read_text())
    assert trace['dropped']==0
    assert [t['calls'] for t in trace['totals']]==[1,2,1]
    assert [(e[3],e[4]) for e in trace['events']]==[(0,7),(1,0),(1,1),(2,9)]
    assert all(trace['originMonoNs']<=e[0]<trace['endMonoNs'] and e[1]>0 and e[2]>=0 for e in trace['events'])
    assert trace['endUnixNs']>trace['originUnixNs']
print('Trace contracts passed')
