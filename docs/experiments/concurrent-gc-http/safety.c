/* Deterministic coordinator tests supplement the actual HTTP scheduler tests. */
#include "apply.c"
static void *allocate(neoclr_aot_context *c, uint64_t bytes) {
    void *p = NULL;
    assert(!neoclr_gc_allocate_v1(&c->text, bytes, NEOCLR_GC_BYTES, &p));
    return p;
}
static void gated_start(neoclr_aot_context *c) {
    assert(cycle_start(&active, c));
    started++;
    while (!atomic_load(&active.job->entered)) sched_yield();
}
static void ready(void) {
    atomic_store(&active.job->proceed, true);
    for (;;) {
        assert(!pthread_mutex_lock(&active.worker.mutex));
        bool done = active.worker.state == 2;
        assert(!pthread_mutex_unlock(&active.worker.mutex));
        if (done) return;
        sched_yield();
    }
}
int main(void) {
    uint64_t heap[513] = {0};
    heap[512] = UINT64_C(0xcafebabedeadbeef);
    neoclr_aot_context c = {.text = {(void *)heap, 4096, 0}};
    void *root = allocate(&c, 8);
    *(uint64_t *)root = 42;
    allocate(&c, 128);
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    assert(cycle_init(&active));
    initialized = true;
    gated_start(&c);
    assert(neoclr_gc_boundary_v1(NULL, &frame) == 3);
    assert(neoclr_gc_boundary_v1(&c, NULL) == 3);
    assert(!neoclr_gc_boundary_v1(&c, &frame) && deferred == 1);
    void *fresh = allocate(&c, 8);
    *(uint64_t *)fresh = 99;
    ready();
    assert(!neoclr_gc_boundary_v1(&c, &frame));
    assert(cycles_applied == 1 && cycle_reclaimed_bytes == 128);
    assert(validated_bases(&c.text, root, fresh));
    assert(*(uint64_t *)root == 42 && *(uint64_t *)fresh == 99);
    /* Real rooted-allocation pressure must invalidate a pending snapshot. */
    gated_start(&c);
    void *garbage;
    while (!neoclr_gc_allocate_v1(&c.text, 128, NEOCLR_GC_BYTES, &garbage)) {}
    uint64_t before = statistics.collections;
    assert(!neoclr_gc_allocate_rooted_v1(&c, &frame, 256, NEOCLR_GC_BYTES, &garbage));
    assert(statistics.collections > before && active.epoch != experiment_epoch);
    assert(!neoclr_gc_boundary_v1(&c, &frame) && fallbacks == 1);
    ready();
    assert(!neoclr_gc_boundary_v1(&c, &frame));
    assert(!active.job && cycles_discarded == 1 && *(uint64_t *)root == 42);
    /* Cancel/join a worker held before tracing, before the arena is torn down. */
    gated_start(&c);
    neoclr_shadow_finish_v1();
    assert(cycles_discarded == 2 && !active.job);
    neoclr_probe_leave_v1(&frame);
    assert(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    assert(statistics.allocations == statistics.reclaimed_allocations);
    assert(heap[512] == UINT64_C(0xcafebabedeadbeef));
    puts("HTTP adapter: live application, fresh retention, invalid boundary, pressure rejection, pending cancellation, guard/full reclamation PASS");
}
