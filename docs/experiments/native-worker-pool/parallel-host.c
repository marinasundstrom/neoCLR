#include "native-workers.h"
#include <stdatomic.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed %d: %s\n", __LINE__, #x); abort(); } } while (0)
extern int32_t neoclr_bootstrap_callback_v1(int32_t, uint64_t *, neoclr_aot_context *);
typedef struct { atomic_uint ready, release; int fault; } batch;
static int invoke(neoclr_worker_job *job, void *state) {
    batch *b = state;
    neoclr_worker_enter(job);
    uint64_t *local = neoclr_worker_allocate(job, 8, NEOCLR_GC_BYTES);
    CHECK(local); *local = 12345;
    CHECK(neoclr_worker_local_store(job, 0, local));
    neoclr_worker_leave(job);
    atomic_fetch_add(&b->ready, 1);
    while (!atomic_load(&b->release)) sched_yield();
    int status = neoclr_worker_invoke_capture_parallel(job);
    CHECK(status == (b->fault ? NEOCLR_AOT_FAULT_USER_FAULT : 0));
    if (b->fault) CHECK(neoclr_worker_invoke_capture_parallel(job) == 3);
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_roots_parked_v1());
    neoclr_worker_enter(job);
    local = neoclr_worker_local_load(job, 0);
    CHECK(local && *local == 12345);
    neoclr_worker_result(job, (neoclr_worker_value){1, 42, local});
    neoclr_worker_leave(job);
    return 0;
}
static void run(int fault) {
    neoclr_workers *p = neoclr_workers_create(4, 4, 32768);
    neoclr_worker_handle capture;
    CHECK(p && !neoclr_workers_bootstrap(p, neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1, fault ? -1 : 40, &capture));
    for (unsigned round = 0; round < 8; round++) {
        batch b = {.fault = fault};
        neoclr_worker_handle tickets[4];
        for (unsigned i = 0; i < 4; i++) CHECK(neoclr_workers_submit(p, capture, invoke, &b, &tickets[i]));
        while (atomic_load(&b.ready) < 4) sched_yield();
        CHECK(neoclr_workers_cancel(p, tickets[0]));
        if (round == 7) CHECK(neoclr_workers_root_release(p, capture));
        atomic_store(&b.release, 1);
        for (unsigned i = 0; i < 4; i++) CHECK(neoclr_workers_wait(p, tickets[i], 30000) == 1);
        uint64_t extent;
        CHECK(neoclr_workers_collect(p, &extent));
        for (unsigned i = 0; i < 4; i++) {
            neoclr_worker_fault_info info; unsigned char message[128];
            CHECK(neoclr_workers_fault_copy(p, tickets[i], &info, message, sizeof(message)) == 1);
            if (fault && i) {
                const char *expected = "Parallel worker failure -1";
                CHECK(info.code == NEOCLR_AOT_FAULT_USER_FAULT && info.message_bytes == strlen(expected));
                CHECK(!memcmp(message, expected, info.message_bytes));
            } else CHECK(!info.code);
            neoclr_worker_value value; neoclr_worker_handle root; int outcome;
            CHECK(neoclr_workers_take(p, tickets[i], &outcome, &value, &root) == 1);
            CHECK(outcome == (i == 0 ? NEOCLR_WORKER_CANCELLED : fault ? NEOCLR_WORKER_FAILED : NEOCLR_WORKER_OK));
            if (root.pool) {
                uint64_t scalar;
                CHECK(neoclr_workers_root_read(p, root, &scalar) && scalar == 12345);
                CHECK(neoclr_workers_root_release(p, root));
            } else CHECK(i == 0 || fault);
        }
    }
    unsigned peak; uint64_t collections, extent;
    CHECK(neoclr_workers_compiled_statistics(p, &peak, &collections) && collections);
    if (!fault) CHECK(peak > 1);
    CHECK(neoclr_workers_collect(p, &extent) && !extent);
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
    uint64_t requests;
    CHECK(neoclr_workers_allocation_requests(p,&requests) && requests);
    printf("OWNER ALLOCATION %llu requests\n",(unsigned long long)requests);
#endif
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    printf("PASS compiled shared-heap %s: 32 invocations, peak %u active participants, %llu coordinated collections\n",
        fault ? "fault isolation" : "graph/GC", peak, (unsigned long long)collections);
}
static void close_running(void) {
    neoclr_workers *p=neoclr_workers_create(4,4,32768);
    neoclr_worker_handle capture,tickets[4];batch b={0};
    CHECK(p && !neoclr_workers_bootstrap(p,neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1,40,&capture));
    for(unsigned i=0;i<4;i++)CHECK(neoclr_workers_submit(p,capture,invoke,&b,&tickets[i]));
    while(atomic_load(&b.ready)<4)sched_yield();
    atomic_store(&b.release,1);
    /* Close must leave allocation service available until generated calls drain. */
    CHECK(neoclr_workers_close(p));
    for(unsigned i=0;i<4;i++) {
        neoclr_worker_value value;neoclr_worker_handle root;int outcome;
        CHECK(neoclr_workers_take(p,tickets[i],&outcome,&value,&root)==1);
        CHECK(outcome==NEOCLR_WORKER_CANCELLED || outcome==NEOCLR_WORKER_OK);
        if(root.pool)CHECK(neoclr_workers_root_release(p,root));
    }
    CHECK(neoclr_workers_root_release(p,capture));
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
    uint64_t requests;
    CHECK(neoclr_workers_allocation_requests(p,&requests) && requests);
    printf("OWNER CLOSE %llu requests drained before shutdown\n",(unsigned long long)requests);
#endif
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    puts("PASS close during four running compiled invocations and repeated close");
}
int main(void) { run(0); run(1); close_running(); }
