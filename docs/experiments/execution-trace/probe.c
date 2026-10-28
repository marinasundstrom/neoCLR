#define _POSIX_C_SOURCE 200809L
#include "native-worker-trace.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"check line %d\n",__LINE__); abort(); } } while (0)
static pthread_mutex_t gate = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t changed = PTHREAD_COND_INITIALIZER;
static int started, released;
static int work(neoclr_worker_job *j, void *state) {
    (void)state;
    neoclr_worker_enter(j);
    uint64_t value = *(uint64_t *)neoclr_worker_capture(j);
    neoclr_worker_result(j, (neoclr_worker_value){1, (int64_t)value, NULL});
    neoclr_worker_leave(j);
    return 0;
}
static int fail(neoclr_worker_job *j, void *state) { (void)j; (void)state; return 1; }
static int block(neoclr_worker_job *j, void *state) {
    CHECK(!pthread_mutex_lock(&gate)); started = 1;
    CHECK(!pthread_cond_broadcast(&changed));
    while (!released) CHECK(!pthread_cond_wait(&changed, &gate));
    CHECK(!pthread_mutex_unlock(&gate));
    return work(j, state);
}
static void take(neoclr_workers *p, neoclr_worker_handle h, int expected) {
    neoclr_worker_value value; neoclr_worker_handle root; int outcome;
    CHECK(neoclr_workers_wait(p,h,10000) == 1);
    CHECK(neoclr_workers_take(p,h,&outcome,&value,&root) == 1);
    CHECK(outcome == expected && !root.pool);
    if (!outcome) CHECK(value.tag == 1 && value.scalar == 42);
}
static void dump(neoclr_workers *p, unsigned workers) {
    printf("{\"schema\":1,\"clock\":\"CLOCK_MONOTONIC\",\"units\":\"ns\",\"workers\":%u,\"lanes\":[",workers);
    for (unsigned lane=0; lane<=workers; lane++) {
        neoclr_worker_trace_view v; CHECK(neoclr_workers_trace_view(p,lane,&v));
        printf("%s{\"lane\":%u,\"pool\":%llu,\"origin_ns\":%llu,\"attempted\":%llu,\"dropped\":%llu,\"capacity\":%zu,\"events\":[",
            lane ? "," : "",lane,(unsigned long long)v.pool,(unsigned long long)v.origin_ns,
            (unsigned long long)v.attempted,(unsigned long long)v.dropped,v.capacity);
        for (size_t i=0;i<v.count;i++) {
            const neoclr_worker_trace_event *e=&v.events[i];
            printf("%s[%llu,%llu,%llu,%u,%u,%u]",i ? "," : "",(unsigned long long)e->ns,
                (unsigned long long)e->sequence,(unsigned long long)e->generation,e->kind,e->slot,e->detail);
        }
        printf("]}");
    }
    printf("]}\n");
}
static double now(void) { struct timespec t; CHECK(!clock_gettime(CLOCK_MONOTONIC,&t)); return t.tv_sec+t.tv_nsec/1e9; }
int main(int argc,char **argv) {
    unsigned workers = argc>1 ? (unsigned)atoi(argv[1]) : 1;
    unsigned capacity = argc>2 ? (unsigned)atoi(argv[2]) : 1024;
    int benchmark = argc>3;
    CHECK(!neoclr_workers_create_traced(1,2,8192,65537));
    neoclr_workers *p=neoclr_workers_create_traced(workers,16,8192,capacity);
    neoclr_worker_handle root,tickets[16]; CHECK(p && neoclr_workers_root_new(p,42,&root));
    neoclr_worker_trace_view v; CHECK(!neoclr_workers_trace_view(p,0,&v));
    clock_t cpu=clock(); double start=now();
    unsigned batches=benchmark ? (argc>4 ? (unsigned)atoi(argv[4]) : 512) : 3;
    CHECK(batches && batches<=32768);
    for (unsigned batch=0;batch<batches;batch++) {
        for (unsigned i=0;i<16;i++) CHECK(neoclr_workers_submit(p,root,work,NULL,&tickets[i]));
        for (unsigned i=0;i<16;i++) take(p,tickets[i],0);
    }
    double elapsed=now()-start, cpu_seconds=(double)(clock()-cpu)/CLOCKS_PER_SEC;
    if (!benchmark && workers==1) {
        CHECK(neoclr_workers_submit(p,root,block,NULL,&tickets[0]));
        CHECK(!pthread_mutex_lock(&gate));
        while (!started) CHECK(!pthread_cond_wait(&changed,&gate));
        CHECK(!pthread_mutex_unlock(&gate));
        CHECK(neoclr_workers_submit(p,root,work,NULL,&tickets[1]));
        CHECK(neoclr_workers_cancel(p,tickets[1]));
        take(p,tickets[1],NEOCLR_WORKER_CANCELLED);
        CHECK(neoclr_workers_cancel(p,tickets[0]));
        CHECK(!pthread_mutex_lock(&gate)); released=1; CHECK(!pthread_cond_broadcast(&changed));
        CHECK(!pthread_mutex_unlock(&gate));
        take(p,tickets[0],NEOCLR_WORKER_CANCELLED);
    }
    if (!benchmark) {
        CHECK(neoclr_workers_submit(p,root,fail,NULL,&tickets[0]));
        take(p,tickets[0],NEOCLR_WORKER_FAILED);
    }
    CHECK(neoclr_workers_root_release(p,root));
    CHECK(neoclr_workers_close(p)); CHECK(neoclr_workers_close(p));
    if (benchmark) {
        uint64_t events=0,dropped=0;
        if (capacity) for (unsigned lane=0;lane<=workers;lane++) {
            CHECK(neoclr_workers_trace_view(p,lane,&v)); events+=v.count; dropped+=v.dropped;
        }
        printf("{\"workers\":%u,\"enabled\":%s,\"jobs\":%u,\"wallSeconds\":%.9f,\"cpuSeconds\":%.9f,\"events\":%llu,\"dropped\":%llu,\"bufferBytes\":%zu}\n",
            workers,capacity ? "true":"false",batches*16,elapsed,cpu_seconds,(unsigned long long)events,
            (unsigned long long)dropped,(workers+1)*capacity*sizeof(neoclr_worker_trace_event));
    } else if (capacity) dump(p,workers);
    else CHECK(!neoclr_workers_trace_view(p,0,&v));
    CHECK(neoclr_workers_destroy(p));
}
