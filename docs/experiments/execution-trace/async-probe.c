#include "native-runtime.h"
#include "native-async-trace.h"
#include "socket-listener.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while(0)
static uint64_t signal_id;
static int completed, independent, polls;
static int32_t invoke(uint64_t, neoclr_aot_context *);
static void *callback(neoclr_aot_context *c, uint64_t tag) {
    uint64_t *v=NULL;
    CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&v));
    v[0]=UINT32_MAX;v[1]=tag;
    return v;
}
void neoclr_scheduler_wait_v1(neoclr_aot_context *c) {
    int32_t done;
    CHECK(!neoclr_runtime_async_completeunit_v1(signal_id,c,&done) && done);
}
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *c,uint64_t *out,const neoclr_probe_frame *f) {
    (void)out;(void)f;
    CHECK(!neoclr_gc_idle_v1(c,NULL));
    return polls++==0 ? NEOCLR_SCHEDULER_PENDING : NEOCLR_SCHEDULER_IDLE;
}
int32_t neoclr_scheduler_complete_v1(neoclr_aot_context *c,uint64_t h,const neoclr_probe_frame *f) {
    (void)c;(void)h;(void)f;return 3;
}
static int32_t drain(neoclr_aot_context *c) { (void)c;return 0; }
static int32_t invoke(uint64_t h,neoclr_aot_context *c) {
    uint64_t *v=NULL; CHECK(!neoclr_gc_callback_read_v1(c,h,(void **)&v));
    uint64_t tag=v[1]; int32_t result;
    if (tag==2) { independent++; return 0; }
    if (tag==3) return 3;
    if (tag==4) {
        uint64_t child;
        CHECK(!neoclr_runtime_async_start_v1(callback(c,2),c,&child,invoke));
        CHECK(!neoclr_runtime_async_release_v1(child,c,&result) && result);
        return 0;
    }
    uint64_t *local=callback(c,42);
    neoclr_probe_storage slot={&local,8,0}; neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,c,1,&slot,1,"{}",2);
    CHECK(!neoclr_runtime_async_waitcompletion_v1(signal_id,c,&result) && result);
    CHECK(local[1]==42);
    neoclr_probe_leave_v1(&frame);completed++;
    return 0;
}
int main(int argc,char **argv) {
    size_t cap=argc>1 ? (size_t)atoi(argv[1]) : 65536;
    unsigned rounds=argc>2 ? (unsigned)atoi(argv[2]) : 8;
    CHECK(cap<=65536 && rounds && rounds<=2048);
    CHECK(!neoclr_async_trace_begin(65537));
    if (cap) CHECK(neoclr_async_trace_begin(cap));
    uint64_t storage[8192]={0};
    neoclr_aot_context c={.text={(unsigned char *)storage,sizeof(storage),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    clock_t cpu=clock(); struct timespec start,end;
    CHECK(!clock_gettime(CLOCK_MONOTONIC,&start));
    for (unsigned round=0;round<rounds;round++) {
        uint64_t a,b;int32_t result; polls=0;
        CHECK(!neoclr_runtime_async_create_v1(&c,&signal_id));
        neoclr_async_trace_view bad; CHECK(!neoclr_async_trace_finish(&bad));
        CHECK(!neoclr_runtime_async_start_v1(callback(&c,1),&c,&a,invoke));
        CHECK(completed==(int)round*2);
        CHECK(!neoclr_runtime_async_start_v1(callback(&c,4),&c,&b,invoke));
        CHECK(independent==(int)round+1); /* independent nested work ran while a parked */
        CHECK(!neoclr_runtime_async_release_v1(b,&c,&result) && result);
        CHECK(!neoclr_gc_idle_v1(&c,NULL));
        CHECK(!neoclr_gc_idle_v1(&c,NULL));
        CHECK(!neoclr_runtime_async_pump_v1(&c,invoke,drain));
        CHECK(completed==(int)round*2+1);
        CHECK(!neoclr_runtime_async_release_v1(a,&c,&result) && result);
        /* Completion before wait must not manufacture another park. */
        CHECK(!neoclr_runtime_async_start_v1(callback(&c,1),&c,&a,invoke));
        CHECK(completed==(int)round*2+2);
        CHECK(!neoclr_runtime_async_release_v1(a,&c,&result) && result);
        CHECK(!neoclr_runtime_async_release_v1(signal_id,&c,&result) && result);
        CHECK(!neoclr_gc_idle_v1(&c,NULL));
    }
    CHECK(!clock_gettime(CLOCK_MONOTONIC,&end));
    double wall=end.tv_sec-start.tv_sec+(end.tv_nsec-start.tv_nsec)/1e9;
    double seconds=(double)(clock()-cpu)/CLOCKS_PER_SEC;
    /* Fault and forced shutdown retain distinct terminal status/reason. */
    uint64_t failed; CHECK(neoclr_runtime_async_start_v1(callback(&c,3),&c,&failed,invoke)==3);
    CHECK(!neoclr_runtime_async_create_v1(&c,&signal_id));
    CHECK(!neoclr_runtime_async_start_v1(callback(&c,1),&c,&failed,invoke));
    CHECK(neoclr_runtime_async_shutdown_v1(&c)==3);
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
#ifdef TRACE_LIVE
    extern void experiment_gc_candidate_counts(uint64_t *);
    uint64_t counts[6]; experiment_gc_candidate_counts(counts);
    CHECK(counts[0] && counts[1]);
    fprintf(stderr,"epochs started=%llu finished=%llu\n",(unsigned long long)counts[0],(unsigned long long)counts[1]);
#endif
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    neoclr_async_trace_view view={0};
    if (cap) CHECK(neoclr_async_trace_finish(&view));
    else CHECK(!neoclr_async_trace_finish(&view));
    printf("{\"schema\":\"async-1\",\"origin_ns\":%llu,\"attempted\":%llu,\"dropped\":%llu,\"capacity\":%zu,\"rounds\":%u,\"wallSeconds\":%.9f,\"cpuSeconds\":%.9f,\"events\":[",(unsigned long long)view.origin_ns,(unsigned long long)view.attempted,(unsigned long long)view.dropped,view.capacity,rounds,wall,seconds);
    for(size_t i=0;i<view.count;i++) {
        const neoclr_async_trace_event *e=&view.events[i];
        printf("%s[%llu,%llu,%llu,%llu,%u,%u]",i?",":"",(unsigned long long)e->ns,(unsigned long long)e->sequence,(unsigned long long)e->context,(unsigned long long)e->related,e->kind,e->detail);
    }
    puts("]}");CHECK(neoclr_async_trace_dispose());
    CHECK(neoclr_async_trace_begin(1)); CHECK(neoclr_async_trace_finish(&view)); CHECK(neoclr_async_trace_dispose());
    return 0;
}
