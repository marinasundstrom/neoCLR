#include "native-workers.h"
#include <stdatomic.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "check failed at %d: %s\n", __LINE__, #x); abort(); } } while (0)
typedef struct { atomic_uint arrived; } batch;
static _Thread_local unsigned visits;
static int work(neoclr_worker_job *job, void *state) {
    batch *b = state;
    neoclr_worker_enter(job);
    int32_t input = (int32_t)*(uint64_t *)neoclr_worker_capture(job);
    neoclr_worker_leave(job);
    /* Private invocation storage: never pass the pool's context to generated code. */
    uint64_t storage[257] = {0};
    storage[256] = 1234567;
    neoclr_aot_context context = {.text = {(unsigned char *)storage, 2048, 0}};
    CHECK(neoclr_root_probe_depth_v1() == 0);
    atomic_fetch_add(&b->arrived, 1);
    while (atomic_load(&b->arrived) < 4) sched_yield();
    for (unsigned invocation = 0; invocation < 20; invocation++) {
        int32_t argument = invocation % 2 ? input : -1;
        int32_t result = -99;
        neoclr_gc_statistics before = neoclr_gc_statistics_v1();
        int32_t status = neoclr_entry_v4(argument, &result, &context);
        neoclr_gc_statistics after = neoclr_gc_statistics_v1();
        CHECK(neoclr_root_probe_depth_v1() == 0);
        CHECK(after.collections > before.collections);
        CHECK(after.allocations >= before.allocations + 501);
        if (argument < 0) {
            CHECK(status == NEOCLR_AOT_FAULT_INVALID_CAST);
            CHECK(context.fault.code == NEOCLR_AOT_FAULT_INVALID_CAST && result == -99);
        } else {
            CHECK(status == 0 && context.fault.code == 0 && result == input);
        }
        CHECK(neoclr_gc_collect_v1(&context, NULL) == 0);
        CHECK(context.text.used == 0 && storage[256] == 1234567);
    }
    visits++;
    neoclr_worker_enter(job);
    neoclr_worker_result(job, (neoclr_worker_value){1, visits, NULL});
    neoclr_worker_leave(job);
    return 0;
}
int main(void) {
    neoclr_workers *pool = neoclr_workers_create(4, 4, 8192);
    CHECK(pool);
    for (unsigned round = 1; round <= 8; round++) {
        batch b = {0};
        neoclr_worker_handle roots[4], tickets[4];
        for (unsigned i = 0; i < 4; i++) {
            CHECK(neoclr_workers_root_new(pool, 40 + i, &roots[i]));
            CHECK(neoclr_workers_submit(pool, roots[i], work, &b, &tickets[i]));
            CHECK(neoclr_workers_root_release(pool, roots[i]));
        }
        uint64_t live;
        CHECK(neoclr_workers_collect(pool, &live));
        for (unsigned i = 0; i < 4; i++) {
            neoclr_worker_value value; neoclr_worker_handle root; int outcome;
            CHECK(neoclr_workers_wait(pool, tickets[i], 10000) == 1);
            CHECK(neoclr_workers_take(pool, tickets[i], &outcome, &value, &root) == 1);
            CHECK(outcome == NEOCLR_WORKER_OK && value.scalar == round && !root.pool);
        }
    }
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
    puts("PASS 640 generated invocations: four workers, forced GC, retained values, fault recovery, TLS reuse and reclamation");
}
