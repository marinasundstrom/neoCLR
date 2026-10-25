"""Diagnostic owner begin/capture CPU and completed marker CPU; no per-object clocks."""
import re

HEADER = r'''
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#ifndef BEGIN_WALL_NOW
static uint64_t begin_clock(clockid_t id) {
    struct timespec t;
    if (clock_gettime(id, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
#define BEGIN_WALL_NOW() begin_clock(CLOCK_MONOTONIC)
#define BEGIN_CPU_NOW() begin_clock(CLOCK_THREAD_CPUTIME_ID)
#endif
typedef struct { uint64_t wall, cpu; } begin_stamp;
typedef struct { uint64_t count, wall, cpu, failures; } begin_row;
static _Thread_local begin_row begin_rows[2];
static begin_stamp begin_start(void) {
    begin_stamp s = {BEGIN_WALL_NOW(), BEGIN_CPU_NOW()};
    return s;
}
static void begin_end(unsigned kind, begin_stamp start, int failed) {
    uint64_t cpu=BEGIN_CPU_NOW(), wall=BEGIN_WALL_NOW();
    if (kind>=2 || wall<start.wall || cpu<start.cpu) abort();
    begin_row *r=&begin_rows[kind];
    if (r->count==UINT64_MAX || wall-start.wall>UINT64_MAX-r->wall || cpu-start.cpu>UINT64_MAX-r->cpu) abort();
    r->count++;r->wall+=wall-start.wall;r->cpu+=cpu-start.cpu;r->failures+=failed!=0;
}
void experiment_begin_phases_report(void) {
    for (unsigned i=0;i<2;i++) {
        begin_row *r=&begin_rows[i];
        fprintf(stderr,"BEGIN_PHASE %u %llu %llu %llu %llu\n",i,
            (unsigned long long)r->count,(unsigned long long)r->wall,
            (unsigned long long)r->cpu,(unsigned long long)r->failures);
    }
    /* Owner-side totals are transferred only after a marker pass has joined.
     * Include completed and cancelled jobs; never read the active worker's job. */
    extern void experiment_gc_time_counts(uint64_t *);
    extern void experiment_gc_live_counts(uint64_t *);
    uint64_t times[5], live[5];
    experiment_gc_time_counts(times);experiment_gc_live_counts(live);
    fprintf(stderr,"MARKER_WORK %llu %llu %llu %llu %llu %llu\n",
        (unsigned long long)times[0],(unsigned long long)live[0],
        (unsigned long long)live[1],(unsigned long long)live[2],
        (unsigned long long)live[3],(unsigned long long)live[4]);
}
'''

def instrument(source):
    assert 'begin_impl_' not in source
    wrappers=[]
    for name,args,kind,void in [
        ('experiment_gc_begin','context, head, conditional, count',0,False),
        ('shadow_begin','context, conditional, count',1,True),
    ]:
        matches=list(re.finditer(r'(?m)^((?:static )?(?:int32_t|void) )'+name+r'(\([^;{}]*\)) \{',source))
        assert len(matches)==1,(name,len(matches))
        m=matches[0];signature=m[1]+name+m[2]
        source=source[:m.start()]+signature+';\n'+m[1]+'begin_impl_'+name+m[2]+' {'+source[m.end():]
        call='begin_impl_'+name+'('+args+')'
        body=call+';\n    begin_end(%d,start,0);'%kind if void else 'int32_t result='+call+';\n    begin_end(%d,start,result!=0);\n    return result;'%kind
        wrappers.append(signature+' {\n    begin_stamp start=begin_start();\n    '+body+'\n}\n')
    return HEADER+'\n'+source+'\n'+'\n'.join(wrappers)
