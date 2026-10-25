"""Diagnostic finish subphases: owner wall/CPU and components of the same slowest finish."""
import re

HEADER = r'''
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>
#ifndef FINISH_WALL_NOW
static uint64_t finish_clock(clockid_t id) {
    struct timespec t;
    if(clock_gettime(id,&t)) abort();
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
#define FINISH_WALL_NOW() finish_clock(CLOCK_MONOTONIC)
#define FINISH_CPU_NOW() finish_clock(CLOCK_THREAD_CPUTIME_ID)
#endif
typedef struct { uint64_t wall, cpu; } finish_stamp;
typedef struct { uint64_t count, wall, wall_max, cpu, cpu_max, bins[64]; } finish_row;
static _Thread_local finish_row finish_rows[6];
static _Thread_local finish_stamp finish_slowest[6];
static _Thread_local unsigned finish_slowest_mask;
static finish_stamp finish_start(void) {
    finish_stamp s; s.wall=FINISH_WALL_NOW(); s.cpu=FINISH_CPU_NOW(); return s;
}
static void finish_end(unsigned kind, finish_stamp start, finish_stamp *event) {
    uint64_t cpu=FINISH_CPU_NOW(),wall=FINISH_WALL_NOW();
    if(cpu<start.cpu || wall<start.wall || kind>=6) abort();
    finish_stamp elapsed={wall-start.wall,cpu-start.cpu};
    finish_row *r=&finish_rows[kind];
    if(r->count==UINT64_MAX || elapsed.wall>UINT64_MAX-r->wall || elapsed.cpu>UINT64_MAX-r->cpu) abort();
    r->count++; r->wall+=elapsed.wall; r->cpu+=elapsed.cpu;
    if(elapsed.wall>r->wall_max) r->wall_max=elapsed.wall;
    if(elapsed.cpu>r->cpu_max) r->cpu_max=elapsed.cpu;
    unsigned b=0; for(uint64_t n=elapsed.wall;n>1;n>>=1) b++;
    r->bins[b]++; event[kind]=elapsed;
}
static void finish_event(finish_stamp *event, unsigned mask) {
    if(finish_rows[5].count==1 || event[5].wall>finish_slowest[5].wall) {
        for(unsigned i=0;i<6;i++) finish_slowest[i]=event[i];
        finish_slowest_mask=mask;
    }
}
void experiment_finish_phases_report(void) {
    for(unsigned i=0;i<6;i++) {
        finish_row *r=&finish_rows[i];
        fprintf(stderr,"FINISH_PHASE %u %llu %llu %llu %llu %llu",i,
            (unsigned long long)r->count,(unsigned long long)r->wall,
            (unsigned long long)r->wall_max,(unsigned long long)r->cpu,(unsigned long long)r->cpu_max);
        for(unsigned j=0;j<64;j++) fprintf(stderr," %llu",(unsigned long long)r->bins[j]);
        fprintf(stderr,"\n");
    }
    fprintf(stderr,"FINISH_MAX %u",finish_slowest_mask);
    for(unsigned i=0;i<6;i++) fprintf(stderr," %llu %llu",(unsigned long long)finish_slowest[i].wall,(unsigned long long)finish_slowest[i].cpu);
    fprintf(stderr,"\n");
}
'''

def instrument(source):
    assert 'experiment_finish_phases_report' not in source
    matches=list(re.finditer(r'(?m)^int32_t experiment_gc_finish\(neoclr_aot_context \*context,[^;{]*\) \{',source))
    assert len(matches)==1, ('finish definitions',len(matches))
    start = matches[0].start()
    end = source.index('\n}\n', start) + 3
    body = source[start:end]
    def replace(old, new):
        nonlocal body
        assert body.count(old)==1, (old,body.count(old))
        body=body.replace(old,new)
    replace('    candidate_wait();\n    shadow_worker', '''    finish_stamp event[6]={{0,0}}, whole=finish_start(), phase=finish_start();
    unsigned phase_mask=59; /* all except optional remark wait */
    candidate_wait();
    finish_end(0,phase,event); phase=finish_start();
    shadow_worker''')
    replace('    if (s->valid && s->pending_count) {', '''    finish_end(1,phase,event);
    if (s->valid && s->pending_count) {
        phase=finish_start(); phase_mask|=4;''')
    replace('        candidate_wait();\n    }', '''        candidate_wait();
        finish_end(2,phase,event);
    }''')
    replace('    candidate_phase = 2;', '    phase=finish_start();\n    candidate_phase = 2;')
    replace('    candidate_phase = 0;', '    finish_end(3,phase,event); phase=finish_start();\n    candidate_phase = 0;')
    replace('    if (!status) candidate_counts[1]++;', '''    finish_end(4,phase,event);
    if (!status) candidate_counts[1]++;
    finish_end(5,whole,event); finish_event(event,phase_mask);''')
    return HEADER+'\n'+source[:start]+body+source[end:]
