#include "native-runtime.h"
#include "native-worker-delivery.h"
#include "socket-listener.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
extern int32_t neoclr_bootstrap_callback_v1(int32_t, uint64_t *, neoclr_aot_context *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
static neoclr_workers *pool;
static neoclr_worker_handle capture;
static uint64_t signal_id;
static int pending, finished, faults;
static atomic_uint_fast64_t allocations, collections, epochs_started, epochs_finished, epochs_cancelled;
#ifdef DELIVERY_CONCURRENT
extern void experiment_gc_candidate_counts(uint64_t *);
#endif
static int work(neoclr_worker_job *j,void *state) {
    (void)state;
#ifdef DELIVERY_CONCURRENT
    uint64_t epochs_before[6], epochs_after[6];
    experiment_gc_candidate_counts(epochs_before);
#endif
    neoclr_gc_statistics before=neoclr_gc_statistics_v1();
    int status = neoclr_worker_invoke_capture(j);
    neoclr_gc_statistics after=neoclr_gc_statistics_v1();
    CHECK(after.allocations>=before.allocations+2500);
    CHECK(after.collections>before.collections);
    atomic_fetch_add(&allocations,after.allocations-before.allocations);
    atomic_fetch_add(&collections,after.collections-before.collections);
#ifdef DELIVERY_CONCURRENT
    experiment_gc_candidate_counts(epochs_after);
#ifdef NEOCLR_WORKER_OWNED_CALLBACKS
    CHECK(!epochs_after[5] && epochs_after[0]==epochs_after[1]+epochs_after[2]);
#endif
    atomic_fetch_add(&epochs_started,epochs_after[0]-epochs_before[0]);
    atomic_fetch_add(&epochs_finished,epochs_after[1]-epochs_before[1]);
    atomic_fetch_add(&epochs_cancelled,epochs_after[2]-epochs_before[2]);
#endif
    neoclr_worker_enter(j);
    neoclr_worker_result(j,(neoclr_worker_value){3,status,neoclr_worker_capture(j)});
    neoclr_worker_leave(j);
    return 0;
}
static void completion(neoclr_workers *p,neoclr_worker_handle ticket,void *state) {
    neoclr_aot_context *c=state;
    neoclr_worker_handle root; neoclr_worker_value value; int outcome;
    uint64_t extent;
    CHECK(neoclr_workers_collect(p,&extent));
    neoclr_worker_fault_info info;
    unsigned char message[128];
    CHECK(neoclr_workers_fault_copy(p,ticket,&info,message,sizeof(message))==1);
    CHECK(neoclr_workers_take(p,ticket,&outcome,&value,&root)==1);
    CHECK(!value.reference); /* Only rooted handles cross the pool boundary. */
    if (finished==32) {
        CHECK(outcome==NEOCLR_WORKER_FAILED && !root.pool && info.code==NEOCLR_AOT_FAULT_USER_FAULT);
        CHECK(info.message_bytes==17 && !memcmp(message,"Worker failure 33",17));
        faults++;
    } else {
        CHECK(outcome==NEOCLR_WORKER_OK && root.pool && !value.scalar && !info.code);
        capture=root; /* Adopt the very same closure, never an owner-heap copy. */
    }
    int32_t done; CHECK(!neoclr_runtime_async_completeunit_v1(signal_id,c,&done) && done);
    pending=0;
}
static int poll_source(neoclr_aot_context *c,void *state) {
    CHECK(state==pool && neoclr_workers_dispatch(pool,1,completion,c)>=0);
    return pending;
}
static int32_t invoke(uint64_t h,neoclr_aot_context *c) {
    void *callback; CHECK(!neoclr_gc_callback_read_v1(c,h,&callback));
    for (unsigned i=0;i<64;i++) {
        neoclr_worker_handle ticket;
        CHECK(!neoclr_runtime_async_create_v1(c,&signal_id));
        pending=1; CHECK(neoclr_workers_submit(pool,capture,work,NULL,&ticket));
        /* Keep a recovery root for the deliberately faulting job: failure discards results. */
        if (i!=32) CHECK(neoclr_workers_root_release(pool,capture));
        int32_t done;
        CHECK(!neoclr_runtime_async_waitcompletion_v1(signal_id,c,&done) && done);
        CHECK(!neoclr_runtime_async_release_v1(signal_id,c,&done) && done);
        finished++;
    }
    return 0;
}
static int32_t drain(neoclr_aot_context *c) { (void)c; return 0; }
int main(void) {
    #ifdef NEOCLR_WORKER_OWNED_CALLBACKS
    pool=neoclr_workers_create(2,1,131072);
#else
    pool=neoclr_workers_create(2,1,8192);
#endif
#ifdef NEOCLR_EXPECT_IMAGE_REJECTION
    CHECK(pool && neoclr_workers_bootstrap(pool,neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1,32,&capture)==3 && !capture.pool);
    uint64_t untouched;
    CHECK(neoclr_workers_collect(pool,&untouched) && !untouched);
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
    puts("PASS ordinary callback image rejected before owned live bootstrap");
    return 0;
#endif
    CHECK(pool && !neoclr_workers_bootstrap(pool,neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1,32,&capture));
    neoclr_async_external_source source={neoclr_workers_completion_descriptor(pool),poll_source,pool};
    CHECK(source.descriptor>=0);
    uint64_t storage[8192]={0}; neoclr_aot_context c={.text={(unsigned char *)storage,sizeof(storage),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    neoclr_socket_scope scope; CHECK(!neoclr_socket_scope_enter_v1(&scope,&c));
    uint64_t *callback=NULL; CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&callback));
    callback[0]=UINT32_MAX; callback[1]=1; uint64_t body;
    CHECK(!neoclr_runtime_async_start_v1(callback,&c,&body,invoke));
    CHECK(!neoclr_runtime_async_pump_external_v1(&c,invoke,drain,&source));
    CHECK(finished==64 && faults==1 && !pending);
    uint64_t extent; CHECK(neoclr_workers_collect(pool,&extent));
    int32_t done; CHECK(!neoclr_runtime_async_release_v1(body,&c,&done) && done);
    CHECK(!neoclr_runtime_async_shutdown_v1(&c));
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(neoclr_workers_root_release(pool,capture));
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
#ifdef DELIVERY_CONCURRENT
    /* Borrowed pool arenas have no owned-heap registration: synchronous fallback. */
#ifdef NEOCLR_WORKER_OWNED_CALLBACKS
    CHECK(atomic_load(&epochs_started)>0);
#else
    CHECK(!atomic_load(&epochs_started));
#endif
    uint64_t owner_epochs[6];
    experiment_gc_candidate_counts(owner_epochs);
    CHECK(owner_epochs[0] && !owner_epochs[5]);
    CHECK(owner_epochs[0]==owner_epochs[1]+owner_epochs[2]);
    printf("owner epochsStarted=%llu finished=%llu cancelled=%llu\n",
        (unsigned long long)owner_epochs[0],(unsigned long long)owner_epochs[1],
        (unsigned long long)owner_epochs[2]);
#endif
    printf("worker allocations=%llu collections=%llu epochsStarted=%llu finished=%llu cancelled=%llu\n",
        (unsigned long long)atomic_load(&allocations),(unsigned long long)atomic_load(&collections),
        (unsigned long long)atomic_load(&epochs_started),
        (unsigned long long)atomic_load(&epochs_finished),(unsigned long long)atomic_load(&epochs_cancelled));
    puts("PASS 64 generated shared closures: owner resume, ticket-only roots, collections, fault delivery and recovery");
}
