#include "native-workers.h"
#include <stdatomic.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed %d: %s\n", __LINE__, #x); abort(); } } while (0)
extern int32_t neoclr_bootstrap_callback_v1(int32_t, uint64_t *, neoclr_aot_context *);
/* Native image adapter audit: the pool owner is inactive while each fresh
 * invocation exclusively owns the same arena, including newly created statics. */
static neoclr_aot_context *heap_owner;
static unsigned char *heap_data;
static int32_t audit_bootstrap(int32_t argument, uint64_t *handle, neoclr_aot_context *context) {
    heap_owner = context; heap_data = context->text.data;
    return neoclr_bootstrap_callback_v1(argument, handle, context);
}
static int32_t audit_invoke(uint64_t handle, neoclr_aot_context *context) {
    CHECK(context != heap_owner && context->text.data == heap_data);
    CHECK(!context->fault.code && !context->fault.frame_count && !context->fault.message);
    CHECK(!heap_owner->text.data && !heap_owner->text.capacity && !heap_owner->text.used);
    CHECK(!heap_owner->static_storage && !heap_owner->fault.code);
    int32_t status = neoclr_invoke_void_callback_v1(handle, context);
    CHECK(context->text.data == heap_data);
    if (status != 3) CHECK(context->static_storage);
    return status;
}
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
    CHECK(pool && !neoclr_workers_bootstrap(pool, audit_bootstrap,
        audit_invoke, 0, &callback));
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
static int exhaust_callback_handles(neoclr_worker_job *job, void *state) {
    (void)state;
    uint64_t buffer[64] = {0};
    neoclr_aot_context foreign = {.text = {(unsigned char *)buffer, sizeof(buffer), 0}};
    void *value = NULL;
    CHECK(!neoclr_gc_allocate_v1(&foreign.text, 8, NEOCLR_GC_BYTES, &value));
    uint64_t handles[NEOCLR_GC_HOST_ROOT_LIMIT];
    for (unsigned i = 0; i < NEOCLR_GC_HOST_ROOT_LIMIT; i++)
        CHECK(!neoclr_gc_host_root_create_v1(&foreign, value, &handles[i]));
#ifdef NEOCLR_WORKER_OWNED_CALLBACKS
    /* Exclusive adoption rejects foreign TLS roots before requesting a handle. */
    CHECK(neoclr_worker_invoke_capture(job) == NEOCLR_AOT_FAULT_RUNTIME_ERROR);
#else
    CHECK(neoclr_worker_invoke_capture(job) == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
#endif
    neoclr_worker_enter(job);
    CHECK(heap_owner->text.data == heap_data && !heap_owner->fault.code);
    neoclr_worker_leave(job);
    for (unsigned i = 0; i < NEOCLR_GC_HOST_ROOT_LIMIT; i++)
        CHECK(!neoclr_gc_host_root_release_v1(&foreign, handles[i]));
    CHECK(!neoclr_gc_owned_roots_check_v1(&foreign, NULL, 0));
    /* Admission failure must return the lease without faulting this job. */
    CHECK(!neoclr_worker_invoke_capture(job));
    return 0;
}
static void admission_recovery(void) {
    neoclr_workers *pool = neoclr_workers_create(1, 1, 8192);
    neoclr_worker_handle callback, ticket, root;
    CHECK(pool && !neoclr_workers_bootstrap(pool, audit_bootstrap, audit_invoke, 1, &callback));
    CHECK(neoclr_workers_submit(pool, callback, exhaust_callback_handles, NULL, &ticket));
    CHECK(neoclr_workers_wait(pool, ticket, 10000) == 1);
    neoclr_worker_value value; int outcome;
    CHECK(neoclr_workers_take(pool, ticket, &outcome, &value, &root) == 1 && !outcome && !root.pool);
    CHECK(neoclr_workers_root_release(pool, callback));
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
}
typedef struct { unsigned faults, successes; } observations;
static void observe(neoclr_workers *pool, neoclr_worker_handle ticket, void *state) {
    observations *stats = state;
    neoclr_worker_value value; neoclr_worker_handle root; int outcome;
    CHECK(neoclr_workers_wait(pool, ticket, 10000) == 1);
    uint64_t extent, scalar;
    CHECK(neoclr_workers_collect(pool, &extent));
    neoclr_worker_fault_info info = {99,99,99};
    unsigned char message[128];
    CHECK(neoclr_workers_fault_copy(pool, ticket, &info, message, sizeof(message)) == 1);
    if (info.code) {
        CHECK(info.code == NEOCLR_AOT_FAULT_USER_FAULT && info.frame_count);
        CHECK(info.message_bytes == strlen("Worker failure 81"));
        CHECK(!memcmp(message, "Worker failure 81", info.message_bytes));
        neoclr_worker_fault_info untouched_info = {99,99,99};
        unsigned char tiny = 99;
        CHECK(neoclr_workers_fault_copy(pool, ticket, &untouched_info, &tiny, 1) == -2);
        CHECK(untouched_info.code == 99 && tiny == 99);
        stats->faults++;
    } else { CHECK(!info.frame_count && !info.message_bytes); stats->successes++; }
    CHECK(neoclr_workers_take(pool, ticket, &outcome, &value, &root) == 1);
    CHECK(outcome == (info.code ? NEOCLR_WORKER_FAILED : NEOCLR_WORKER_OK));
    CHECK(neoclr_workers_fault_copy(pool, ticket, &info, message, sizeof(message)) == -1);
    if (root.pool) {
        CHECK(neoclr_workers_collect(pool, &extent));
        CHECK(neoclr_workers_root_read(pool, root, &scalar) && scalar == 12345);
        CHECK(neoclr_workers_root_release(pool, root));
    } else CHECK(outcome == NEOCLR_WORKER_FAILED);
}
int main(void) {
    cancellation_wins();
    admission_recovery();
    neoclr_workers *empty = neoclr_workers_create(1, 1, 8192);
    neoclr_worker_handle scalar, unchanged = {99,99,99,99};
    CHECK(empty && neoclr_workers_root_new(empty, 5, &scalar));
    check_rejected(empty, scalar); /* no compiled image */
    CHECK(neoclr_workers_bootstrap(empty, audit_bootstrap,
        audit_invoke, 80, &unchanged) == 3 && unchanged.pool == 99);
    CHECK(neoclr_workers_root_release(empty, scalar));
    CHECK(neoclr_workers_close(empty) && neoclr_workers_destroy(empty));
    empty = neoclr_workers_create(1, 1, 8192);
    CHECK(empty);
    CHECK(neoclr_workers_bootstrap(empty, audit_bootstrap,
        audit_invoke, -1, &unchanged) != 0 && unchanged.pool == 99);
    CHECK(neoclr_workers_close(empty) && neoclr_workers_destroy(empty));

    neoclr_workers *pool = neoclr_workers_create(4, 4, 8192);
    neoclr_worker_handle callback, sentinel;
    CHECK(pool);
    CHECK(!neoclr_workers_bootstrap(pool, audit_bootstrap,
        audit_invoke, 80, &callback));
    neoclr_worker_handle untouched = {99,99,99,99};
    CHECK(neoclr_workers_bootstrap(pool, audit_bootstrap,
        audit_invoke, 80, &untouched) == 3 && untouched.pool == 99);
    CHECK(neoclr_workers_root_new(pool, 67890, &sentinel));
    check_rejected(pool, sentinel); /* non-function allocation */
    observations stats = {0};
    for (unsigned round = 0; round < 22; round++) {
        batch b = {.expected = round >= 20};
        neoclr_worker_handle tickets[4];
        for (unsigned i = 0; i < 4; i++) CHECK(neoclr_workers_submit(pool, callback, work, &b, &tickets[i]));
        for (unsigned i = 0; i < 4; i++) CHECK(neoclr_workers_wait(pool, tickets[i], 10000) == 1);
        CHECK(neoclr_workers_dispatch(pool, 2, observe, &stats) == 2);
        CHECK(neoclr_workers_dispatch(pool, 2, observe, &stats) == 2);
        CHECK(neoclr_workers_dispatch(pool, 4, observe, &stats) == 0);
        uint64_t scalar;
        CHECK(neoclr_workers_root_read(pool, sentinel, &scalar) && scalar == 67890);
    }
    CHECK(stats.faults == 1 && stats.successes == 87); /* One shared counter; independent jobs recover. */
    CHECK(neoclr_workers_root_release(pool, callback));
    CHECK(neoclr_workers_root_release(pool, sentinel));
    uint64_t reclaimed_extent = 99;
    /* Image statics intentionally remain rooted until invocation teardown. */
    CHECK(heap_owner->text.data == heap_data && heap_owner->static_storage && !heap_owner->fault.code);
    uint64_t reclaimed_before = neoclr_gc_statistics_v1().reclaimed_allocations;
    CHECK(neoclr_workers_collect(pool, &reclaimed_extent) && reclaimed_extent);
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations > reclaimed_before);
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
    puts("PASS invocation context ownership, shared statics, graph identity, cycles and teardown across four workers: 87 successes, one isolated fault; diagnostic text survives collection and later jobs recover");
}
