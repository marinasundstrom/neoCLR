/* Diagnostic only, owner-thread capture. Durations overlap (acquire includes GC).
 * Fixed storage, no allocation or I/O inside hooks; overflow is explicit. */
#include "native-gc.h"
#include "socket-os.h"
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#define CAPACITY 131072
static struct event { uint64_t start, wall, cpu; unsigned kind; int status; } events[CAPACITY];
static uint64_t count, dropped, calls[3], wall_total[3], cpu_total[3], maximum[3];
static uint64_t origin_mono, origin_real;
static uint64_t stamp(clockid_t clock) {
    struct timespec t; if (clock_gettime(clock,&t)) abort();
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
static void record(unsigned kind, uint64_t start, uint64_t cpu, int status) {
    uint64_t wall=stamp(CLOCK_MONOTONIC)-start;
    cpu=stamp(CLOCK_THREAD_CPUTIME_ID)-cpu;
    calls[kind]++;wall_total[kind]+=wall;cpu_total[kind]+=cpu;
    if(wall>maximum[kind])maximum[kind]=wall;
    if(kind!=1 && wall<UINT64_C(100000))return;
    if(count==CAPACITY){dropped++;return;}
    events[count++]=(struct event){start,wall,cpu,kind,status};
}
int32_t trace_actual_collect(neoclr_aot_context *, const neoclr_probe_frame *);
void trace_actual_wait(neoclr_aot_context *);
int32_t trace_actual_acquire(neoclr_aot_context *, uint64_t *, const neoclr_probe_frame *);
int32_t neoclr_gc_collect_v1(neoclr_aot_context *c,const neoclr_probe_frame *f){
    uint64_t start=stamp(CLOCK_MONOTONIC),cpu=stamp(CLOCK_THREAD_CPUTIME_ID);
    int32_t result=trace_actual_collect(c,f);record(0,start,cpu,result);return result;
}
void neoclr_scheduler_wait_v1(neoclr_aot_context *c){
    uint64_t start=stamp(CLOCK_MONOTONIC),cpu=stamp(CLOCK_THREAD_CPUTIME_ID);
    trace_actual_wait(c);record(1,start,cpu,0);
}
void trace_async_pause(void){
    uint64_t start=stamp(CLOCK_MONOTONIC),cpu=stamp(CLOCK_THREAD_CPUTIME_ID);
    neoclr_os_pause();record(1,start,cpu,1);
}
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *c,uint64_t *d,const neoclr_probe_frame *f){
    uint64_t start=stamp(CLOCK_MONOTONIC),cpu=stamp(CLOCK_THREAD_CPUTIME_ID);
    int32_t result=trace_actual_acquire(c,d,f);record(2,start,cpu,result);return result;
}
static void report(void){
    const char *path=getenv("NEOCLR_STALL_TRACE");if(!path)abort();
    FILE *file=fopen(path,"w");if(!file)abort();
    fprintf(file,"{\"originMonoNs\":%"PRIu64",\"originUnixNs\":%"PRIu64",\"endMonoNs\":%"PRIu64",\"endUnixNs\":%"PRIu64",\"dropped\":%"PRIu64",\"totals\":[",origin_mono,origin_real,stamp(CLOCK_MONOTONIC),stamp(CLOCK_REALTIME),dropped);
    for(unsigned i=0;i<3;i++)fprintf(file,"%s{\"kind\":%u,\"calls\":%"PRIu64",\"wallNs\":%"PRIu64",\"cpuNs\":%"PRIu64",\"maxNs\":%"PRIu64"}",i?",":"",i,calls[i],wall_total[i],cpu_total[i],maximum[i]);
    fputs("],\"events\":[",file);
    for(uint64_t i=0;i<count;i++){
        struct event e=events[i];fprintf(file,"%s[%"PRIu64",%"PRIu64",%"PRIu64",%u,%d]",i?",":"",e.start,e.wall,e.cpu,e.kind,e.status);
    }
    fputs("]}\n",file);if(fclose(file))abort();
}
__attribute__((constructor))static void initialize(void){
    origin_mono=stamp(CLOCK_MONOTONIC);origin_real=stamp(CLOCK_REALTIME);
    if(atexit(report))abort();
}
