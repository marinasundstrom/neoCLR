#include "native-workers.h"
#include <stdatomic.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed %d: %s\n", __LINE__, #x); abort(); } } while (0)
extern int32_t neoclr_bootstrap_callback_v1(int32_t, uint64_t *, neoclr_aot_context *);
typedef struct { atomic_uint arrived; int expected; } batch;
static int work(neoclr_worker_job *job, void *state) {
    batch *b = state;
    neoclr_worker_enter(job);
    uint64_t *temporary = neoclr_worker_allocate(job, 8, NEOCLR_GC_BYTES);
    CHECK(temporary); *temporary = 12345;
    CHECK(neoclr_worker_local_store(job, 0, temporary));
    neoclr_worker_leave(job);
    atomic_fetch_add(&b->arrived, 1);
    while (atomic_load(&b->arrived) < 4) sched_yield();
    neoclr_gc_statistics before = neoclr_gc_statistics_v1();
    int32_t status = neoclr_worker_invoke_capture(job);
    neoclr_gc_statistics after = neoclr_gc_statistics_v1();
    if (!b->expected) {
        CHECK(!status && after.collections > before.collections);
        CHECK(after.allocations >= before.allocations + 500);
    } else CHECK(status == NEOCLR_AOT_FAULT_USER_FAULT || status == 0);
    if (status) CHECK(neoclr_worker_invoke_capture(job) == 3);
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_roots_parked_v1());
    neoclr_worker_enter(job);
    temporary = neoclr_worker_local_load(job, 0);
    CHECK(temporary && *temporary == 12345);
    CHECK(neoclr_worker_local_store(job, 0, temporary));
    neoclr_worker_result(job, (neoclr_worker_value){1, status, temporary});
    neoclr_worker_leave(job);
    return 0; /* Test reports guest status as scalar to inspect fault rejection. */
}
static int rejected(neoclr_worker_job *job, void *unused) {
    (void)unused;
    CHECK(neoclr_worker_invoke_capture(job) == 3);
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_roots_parked_v1());
    return 0;
}
static void check_rejected(neoclr_workers *pool, neoclr_worker_handle capture) {
    neoclr_worker_handle ticket, root; neoclr_worker_value value; int outcome;
    CHECK(neoclr_workers_submit(pool, capture, rejected, NULL, &ticket));
    CHECK(neoclr_workers_wait(pool, ticket, 10000) == 1);
    CHECK(neoclr_workers_take(pool, ticket, &outcome, &value, &root) == 1 && !outcome && !root.pool);
}
typedef struct { atomic_int ready, release; } cancellation;
static int cancel_fault(neoclr_worker_job *job, void *state) {
    cancellation *c = state;
    CHECK(neoclr_worker_invoke_capture(job) == NEOCLR_AOT_FAULT_USER_FAULT);
    atomic_store(&c->ready, 1);
    while (!atomic_load(&c->release)) sched_yield();
    return 0;
}
static void cancellation_wins(void) {
    neoclr_workers *pool = neoclr_workers_create(1, 1, 8192);
    neoclr_worker_handle callback, ticket, root;
    CHECK(pool && !neoclr_workers_bootstrap(pool, neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1, 0, &callback));
    cancellation c = {0};
    CHECK(neoclr_workers_submit(pool, callback, cancel_fault, &c, &ticket));
    while (!atomic_load(&c.ready)) sched_yield();
    neoclr_worker_fault_info info = {99,99,99}; unsigned char message[128];
    CHECK(neoclr_workers_fault_copy(pool, ticket, &info, message, sizeof(message)) == 0 && info.code == 99);
    uint64_t extent;
    CHECK(neoclr_workers_collect(pool, &extent));
    CHECK(neoclr_workers_cancel(pool, ticket));
    atomic_store(&c.release, 1);
    CHECK(neoclr_workers_wait(pool, ticket, 10000) == 1);
    CHECK(neoclr_workers_fault_copy(pool, ticket, &info, message, sizeof(message)) == 1 && !info.code && !info.message_bytes);
    neoclr_worker_value value; int outcome;
    CHECK(neoclr_workers_take(pool, ticket, &outcome, &value, &root) == 1);
    CHECK(outcome == NEOCLR_WORKER_CANCELLED && !root.pool);
    CHECK(neoclr_workers_root_release(pool, callback));
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
}
int main(void) {
    cancellation_wins();
    neoclr_workers *empty = neoclr_workers_create(1, 1, 8192);
    neoclr_worker_handle scalar, unchanged = {99,99,99,99};
    CHECK(empty && neoclr_workers_root_new(empty, 5, &scalar));
    check_rejected(empty, scalar); /* no compiled image */
    CHECK(neoclr_workers_bootstrap(empty, neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1, 80, &unchanged) == 3 && unchanged.pool == 99);
    CHECK(neoclr_workers_root_release(empty, scalar));
    CHECK(neoclr_workers_close(empty) && neoclr_workers_destroy(empty));
    empty = neoclr_workers_create(1, 1, 8192);
    CHECK(empty);
    CHECK(neoclr_workers_bootstrap(empty, neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1, -1, &unchanged) != 0 && unchanged.pool == 99);
    CHECK(neoclr_workers_close(empty) && neoclr_workers_destroy(empty));

    neoclr_workers *pool = neoclr_workers_create(4, 4, 8192);
    neoclr_worker_handle callback, sentinel;
    CHECK(pool);
    CHECK(!neoclr_workers_bootstrap(pool, neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1, 80, &callback));
    neoclr_worker_handle untouched = {99,99,99,99};
    CHECK(neoclr_workers_bootstrap(pool, neoclr_bootstrap_callback_v1,
        neoclr_invoke_void_callback_v1, 80, &untouched) == 3 && untouched.pool == 99);
    CHECK(neoclr_workers_root_new(pool, 67890, &sentinel));
    check_rejected(pool, sentinel); /* non-function allocation */
    unsigned faults = 0, successes = 0;
    for (unsigned round = 0; round < 22; round++) {
        batch b = {.expected = round >= 20};
        neoclr_worker_handle tickets[4];
        for (unsigned i = 0; i < 4; i++) CHECK(neoclr_workers_submit(pool, callback, work, &b, &tickets[i]));
        for (unsigned i = 0; i < 4; i++) CHECK(neoclr_workers_wait(pool, tickets[i], 10000) == 1);
        for (unsigned i = 0; i < 4; i++) {
            neoclr_worker_value value; neoclr_worker_handle root; int outcome;
            CHECK(neoclr_workers_wait(pool, tickets[i], 10000) == 1);
            uint64_t extent, scalar;
            CHECK(neoclr_workers_collect(pool, &extent));
            neoclr_worker_fault_info info = {99,99,99};
            unsigned char message[128];
            CHECK(neoclr_workers_fault_copy(pool, tickets[i], &info, message, sizeof(message)) == 1);
            if (info.code) {
                CHECK(info.code == NEOCLR_AOT_FAULT_USER_FAULT && info.frame_count);
                CHECK(info.message_bytes == strlen("Worker failure 81"));
                CHECK(!memcmp(message, "Worker failure 81", info.message_bytes));
                neoclr_worker_fault_info untouched_info = {99,99,99};
                unsigned char tiny = 99;
                CHECK(neoclr_workers_fault_copy(pool, tickets[i], &untouched_info, &tiny, 1) == -2);
                CHECK(untouched_info.code == 99 && tiny == 99);
                faults++;
            } else { CHECK(!info.frame_count && !info.message_bytes); successes++; }
            CHECK(neoclr_workers_take(pool, tickets[i], &outcome, &value, &root) == 1);
            CHECK(outcome == (info.code ? NEOCLR_WORKER_FAILED : NEOCLR_WORKER_OK));
            CHECK(neoclr_workers_fault_copy(pool, tickets[i], &info, message, sizeof(message)) == -1);
            if (root.pool) {
                CHECK(neoclr_workers_collect(pool, &extent));
                CHECK(neoclr_workers_root_read(pool, root, &scalar) && scalar == 12345);
                CHECK(neoclr_workers_root_release(pool, root));
            } else CHECK(outcome == NEOCLR_WORKER_FAILED);
        }
        uint64_t scalar;
        CHECK(neoclr_workers_root_read(pool, sentinel, &scalar) && scalar == 67890);
    }
    CHECK(faults == 1 && successes == 87); /* One shared counter; independent jobs recover. */
    CHECK(neoclr_workers_root_release(pool, callback));
    CHECK(neoclr_workers_root_release(pool, sentinel));
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
    puts("PASS shared compiled callback: 87 successes, one isolated fault; diagnostic text survives collection and later jobs recover");
}
