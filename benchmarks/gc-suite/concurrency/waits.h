/* Diagnostic-only wall/thread-CPU time inside waits. Never link into timings.
 * Aggregate thread wait intervals may exceed elapsed process time. */
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
static const char *wait_names[]={"compiled_allocate","compiled_collect_on_owner",
    "compiled_allocation_owner","compiled_boundary","neoclr_worker_invoke_capture_parallel",
    "run","heap_lock","other","mutex"};
static _Atomic uint64_t wait_calls[9],wait_wall[9],wait_cpu[9];
static uint64_t wait_clock(clockid_t clock) {
    struct timespec t;if(clock_gettime(clock,&t)) abort();
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
static int observe_wait(pthread_cond_t *condition,pthread_mutex_t *mutex,const char *function) {
    unsigned i=0;while(i<7 && strcmp(wait_names[i],function)) i++;
    uint64_t wall=wait_clock(CLOCK_MONOTONIC),cpu=wait_clock(CLOCK_THREAD_CPUTIME_ID);
    int status=pthread_cond_wait(condition,mutex);
    cpu=wait_clock(CLOCK_THREAD_CPUTIME_ID)-cpu;wall=wait_clock(CLOCK_MONOTONIC)-wall;
    atomic_fetch_add_explicit(&wait_calls[i],1,memory_order_relaxed);
    atomic_fetch_add_explicit(&wait_wall[i],wall,memory_order_relaxed);
    atomic_fetch_add_explicit(&wait_cpu[i],cpu,memory_order_relaxed);
    return status;
}
static int observe_lock(pthread_mutex_t *mutex) {
    uint64_t wall=wait_clock(CLOCK_MONOTONIC),cpu=wait_clock(CLOCK_THREAD_CPUTIME_ID);
    int status=pthread_mutex_lock(mutex);
    cpu=wait_clock(CLOCK_THREAD_CPUTIME_ID)-cpu;wall=wait_clock(CLOCK_MONOTONIC)-wall;
    atomic_fetch_add_explicit(&wait_calls[8],1,memory_order_relaxed);
    atomic_fetch_add_explicit(&wait_wall[8],wall,memory_order_relaxed);
    atomic_fetch_add_explicit(&wait_cpu[8],cpu,memory_order_relaxed);
    return status;
}
__attribute__((destructor)) static void dump_waits(void) {
    for(unsigned i=0;i<9;i++) fprintf(stderr,"WAIT %s %llu %llu %llu\n",wait_names[i],
        (unsigned long long)atomic_load(&wait_calls[i]),
        (unsigned long long)atomic_load(&wait_wall[i]),
        (unsigned long long)atomic_load(&wait_cpu[i]));
}
#define pthread_cond_wait(c,m) observe_wait(c,m,__func__)
#define pthread_mutex_lock(m) observe_lock(m)
