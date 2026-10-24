#define _DARWIN_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>
#include <sys/resource.h>
extern int experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_live_wait(void);
extern void experiment_gc_time_counts(uint64_t *);
extern void experiment_gc_live_counts(uint64_t *);
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
#define CHECK(x) do { if (!(x)) abort(); } while(0)
static uint64_t now(clockid_t clock) { struct timespec t; CHECK(!clock_gettime(clock,&t)); return (uint64_t)t.tv_sec*1000000000+t.tv_nsec; }
static int compare(const void *a,const void *b) {
    uint64_t x=*(const uint64_t *)a,y=*(const uint64_t *)b; return (x>y)-(x<y);
}
static void distribution(const char *name,uint64_t *values,unsigned count) {
    printf(",\"%sSamples\":[",name);
    for(unsigned i=0;i<count;i++) printf("%s%llu",i?",":"",(unsigned long long)values[i]);
    printf("]");
    qsort(values,count,sizeof(*values),compare);
    printf(",\"%s\":{\"p50\":%llu,\"p95\":%llu,\"p99\":%llu,\"max\":%llu}",name,
        (unsigned long long)values[(count-1)*50/100],(unsigned long long)values[(count-1)*95/100],
        (unsigned long long)values[(count-1)*99/100],(unsigned long long)values[count-1]);
}
int main(int argc,char **argv) {
    CHECK(argc==4);
    const char *mode=argv[2]; unsigned cycles=(unsigned)strtoul(argv[3],NULL,10);
    CHECK(cycles>=100 && cycles<=10000);
#ifdef STORE_SYNC
    CHECK(!strcmp(mode,"sync"));
#else
    CHECK(!strcmp(mode,"idle") || !strcmp(mode,"overlap"));
#endif
    uint64_t *begin=calloc(cycles,sizeof(uint64_t)),*finish=calloc(cycles,sizeof(uint64_t)),*writes=calloc(cycles,sizeof(uint64_t));
    CHECK(begin && finish && writes); unsigned count=(unsigned)strtoul(argv[1],NULL,10); CHECK(count>=32 && count<=16384);
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
    uint64_t cpu=0,wall=0,elapsed=now(CLOCK_MONOTONIC),start=now(CLOCK_PROCESS_CPUTIME_ID); unsigned random=123;
    for(unsigned round=0;round<cycles;round++) {
#ifndef STORE_SYNC
        uint64_t b=now(CLOCK_MONOTONIC);
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        if (!strcmp(mode,"idle")) experiment_gc_live_wait();
        begin[round]=now(CLOCK_MONOTONIC)-b;
#endif
        uint64_t t=now(CLOCK_THREAD_CPUTIME_ID),w=now(CLOCK_MONOTONIC);
        for(unsigned j=0;j<32;j++) {
            random=random*1664525u+1013904223u; unsigned i=random%count;
#ifdef STORE_SYNC
            nodes[i][2]=round+1;
#else
            CHECK(!experiment_gc_store(&c,&nodes[i][2],round+1,8));
#endif
            CHECK(nodes[i][0]==17 && nodes[i][2]==round+1);
        }
        writes[round]=now(CLOCK_MONOTONIC)-w;
        wall+=writes[round]; cpu+=now(CLOCK_THREAD_CPUTIME_ID)-t;
        uint64_t f=now(CLOCK_MONOTONIC);
#ifdef STORE_SYNC
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
#else
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
#endif
        finish[round]=now(CLOCK_MONOTONIC)-f;
    }
    uint64_t process=now(CLOCK_PROCESS_CPUTIME_ID)-start; elapsed=now(CLOCK_MONOTONIC)-elapsed;
    uint64_t timing[5]={0}, work[5]={0};
#ifndef STORE_SYNC
    experiment_gc_time_counts(timing); experiment_gc_live_counts(work);
    CHECK(work[3]==cycles && !work[4] && work[2]==(uint64_t)cycles*64);
#endif
    for(unsigned i=0;i<count;i++) CHECK(nodes[i][0]==17 && nodes[i][1]==(i?(uintptr_t)nodes[i-1]:0));
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text)); free(nodes); free(c.text.data);
    printf("{\"objects\":%u,\"mode\":\"%s\",\"cycles\":%u,\"stores\":%u,\"storeCpuNs\":%llu,\"storeWallNs\":%llu,\"processCpuNs\":%llu,\"wallNs\":%llu",count,mode,cycles,cycles*32,(unsigned long long)cpu,(unsigned long long)wall,(unsigned long long)process,(unsigned long long)elapsed);
    printf(",\"markerCpuNs\":%llu,\"copiedHeaderBytes\":%llu,\"payloadReadBytes\":%llu,\"barrierWords\":%llu,\"validEpochs\":%llu",(unsigned long long)timing[0],(unsigned long long)work[0],(unsigned long long)work[1],(unsigned long long)work[2],(unsigned long long)work[3]);
    distribution("beginNs",begin,cycles); distribution("finishNs",finish,cycles); distribution("writeWindowNs",writes,cycles);
    struct rusage usage; CHECK(!getrusage(RUSAGE_SELF,&usage));
    printf(",\"maxRssBytes\":%ld}\n",usage.ru_maxrss); /* macOS fixture */
    free(begin); free(finish); free(writes);
}
