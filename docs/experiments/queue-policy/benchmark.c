#define _POSIX_C_SOURCE 200809L
#include "native-workers.h"
#ifdef NEOCLR_WORKER_QUEUES
#include "native-worker-queues.h"
#endif
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if(!(x)) { fprintf(stderr,"line %d\n",__LINE__);abort(); } } while(0)
static neoclr_workers *pool;
static neoclr_worker_handle root,tickets[16];
static _Atomic unsigned next_worker;
static _Thread_local unsigned worker;
static unsigned shape;
struct task { uint64_t submitted,started,result;unsigned index,worker; };
static struct task tasks[16];
static uint64_t now(void) { struct timespec t;CHECK(!clock_gettime(CLOCK_MONOTONIC,&t));return (uint64_t)t.tv_sec*1000000000+t.tv_nsec; }
static uint64_t compute(unsigned iterations,unsigned seed) {
    uint64_t value=seed+1;
    for(unsigned n=0;n<iterations;n++) { value^=value<<13;value^=value>>7;value^=value<<17; }
    return value;
}
static int work(neoclr_worker_job *job,void *state) {
    (void)job;struct task *t=state;t->started=now();
    if(!worker) worker=atomic_fetch_add(&next_worker,1)+1;
    t->worker=worker;
    if(shape && t->index==0) {
        for(unsigned n=1;n<16;n++) {
            tasks[n].submitted=now();
            CHECK(neoclr_workers_submit(pool,root,work,&tasks[n],&tickets[n]));
        }
    }
    if(shape==3 && t->index && t->index%4==0) {
        struct timespec delay={0,500000};CHECK(!nanosleep(&delay,NULL));
    }
    unsigned iterations=shape==2 && t->index && t->index%4==0 ? 200000:32;
    t->result=compute(iterations,t->index);
    return 0;
}
static int compare(const void *a,const void *b) {
    uint64_t x=*(const uint64_t *)a,y=*(const uint64_t *)b;return (x>y)-(x<y);
}
int main(int argc,char **argv) {
    CHECK(argc==4);unsigned workers=(unsigned)atoi(argv[1]);shape=(unsigned)atoi(argv[2]);unsigned rounds=(unsigned)atoi(argv[3]);
    CHECK(workers && workers<=4 && shape<=3 && rounds && rounds<=262144);
    uint64_t *latencies=calloc(rounds*16,sizeof(uint64_t));CHECK(latencies);
    uint64_t expected[16],distribution[4]={0};
    for(unsigned n=0;n<16;n++) expected[n]=compute(shape==2 && n && n%4==0?200000:32,n);
    pool=neoclr_workers_create(workers,16,8192);CHECK(pool && neoclr_workers_root_new(pool,42,&root));
    clock_t cpu=clock();uint64_t begin=now();
    for(unsigned round=0;round<rounds;round++) {
        for(unsigned n=0;n<16;n++) tasks[n]=(struct task){.index=n};
        for(unsigned n=0;n<(shape?1:16);n++) {
            tasks[n].submitted=now();CHECK(neoclr_workers_submit(pool,root,work,&tasks[n],&tickets[n]));
        }
        for(unsigned n=0;n<16;n++) {
            int outcome;neoclr_worker_value v;neoclr_worker_handle result;
            CHECK(neoclr_workers_wait(pool,tickets[n],10000)==1);
            CHECK(neoclr_workers_take(pool,tickets[n],&outcome,&v,&result)==1 && outcome==0);
            CHECK(tasks[n].result==expected[n] && tasks[n].worker && tasks[n].worker<=workers);
            CHECK(tasks[n].started>=tasks[n].submitted);
            latencies[round*16+n]=tasks[n].started-tasks[n].submitted;distribution[tasks[n].worker-1]++;
        }
    }
    uint64_t elapsed=now()-begin;double cpu_seconds=(double)(clock()-cpu)/CLOCKS_PER_SEC;
    CHECK(neoclr_workers_root_release(pool,root));CHECK(neoclr_workers_close(pool));
    uint64_t local=0,global=0,stolen=0,misses=0;
#ifdef NEOCLR_WORKER_QUEUES
    neoclr_worker_queue_counts counts;CHECK(neoclr_workers_queue_counts(pool,&counts));
    local=counts.local;global=counts.global;stolen=counts.stolen;misses=counts.empty_steal_probes;
    CHECK(local+global+stolen==rounds*16);
#endif
    CHECK(neoclr_workers_destroy(pool));
    size_t total=rounds*16;qsort(latencies,total,sizeof(*latencies),compare);
    printf("{\"workers\":%u,\"shape\":%u,\"jobs\":%zu,\"seconds\":%.9f,\"cpuSeconds\":%.9f,\"p50ns\":%llu,\"p95ns\":%llu,\"p99ns\":%llu,\"maxNs\":%llu,\"local\":%llu,\"global\":%llu,\"stolen\":%llu,\"emptyStealProbes\":%llu,\"workerJobs\":[%llu,%llu,%llu,%llu]}\n",workers,shape,total,elapsed/1e9,cpu_seconds,(unsigned long long)latencies[total/2],(unsigned long long)latencies[(total*95+99)/100-1],(unsigned long long)latencies[(total*99+99)/100-1],(unsigned long long)latencies[total-1],(unsigned long long)local,(unsigned long long)global,(unsigned long long)stolen,(unsigned long long)misses,(unsigned long long)distribution[0],(unsigned long long)distribution[1],(unsigned long long)distribution[2],(unsigned long long)distribution[3]);
    free(latencies);
}
