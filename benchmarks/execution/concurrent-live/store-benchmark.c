#define _POSIX_C_SOURCE 200809L
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
extern int experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_live_wait(void);
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
#define CHECK(x) do { if (!(x)) abort(); } while(0)
static uint64_t now(clockid_t clock) { struct timespec t; CHECK(!clock_gettime(clock,&t)); return (uint64_t)t.tv_sec*1000000000+t.tv_nsec; }
int main(int argc,char **argv) {
    CHECK(argc==2); unsigned count=(unsigned)strtoul(argv[1],NULL,10); CHECK(count>=32 && count<=16384);
    neoclr_aot_context c={0}; c.text.capacity=(uint64_t)count*128;
    c.text.data=calloc(1,c.text.capacity); CHECK(c.text.data);
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t **nodes=calloc(count,sizeof(*nodes)); CHECK(nodes);
    for(unsigned i=0;i<count;i++) {
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&nodes[i]));
        nodes[i][0]=17; nodes[i][1]=i?(uintptr_t)nodes[i-1]:0;
    }
    void *root=nodes[count-1]; neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    uint64_t cpu=0,wall=0,start=now(CLOCK_PROCESS_CPUTIME_ID); unsigned random=123;
    for(unsigned round=0;round<100;round++) {
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0)); experiment_gc_live_wait();
        uint64_t t=now(CLOCK_THREAD_CPUTIME_ID),w=now(CLOCK_MONOTONIC);
        for(unsigned j=0;j<32;j++) {
            random=random*1664525u+1013904223u; unsigned i=random%count;
            CHECK(!experiment_gc_store(&c,&nodes[i][2],round+1,8));
            CHECK(nodes[i][0]==17 && nodes[i][2]==round+1);
        }
        wall+=now(CLOCK_MONOTONIC)-w; cpu+=now(CLOCK_THREAD_CPUTIME_ID)-t;
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    }
    uint64_t process=now(CLOCK_PROCESS_CPUTIME_ID)-start;
    for(unsigned i=0;i<count;i++) CHECK(nodes[i][0]==17 && nodes[i][1]==(i?(uintptr_t)nodes[i-1]:0));
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text)); free(nodes); free(c.text.data);
    printf("{\"objects\":%u,\"stores\":3200,\"storeCpuNs\":%llu,\"storeWallNs\":%llu,\"processCpuNs\":%llu}\n",count,(unsigned long long)cpu,(unsigned long long)wall,(unsigned long long)process);
}
