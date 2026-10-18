#include <assert.h>
#include <sched.h>
#include <stdio.h>
#include "native-gc.c"
#include "snapshot.inc"
#include "reclaim.inc"
#include "native-session.h"

static collection_cycle *active_cycle;
static unsigned deferred_boundaries;
/* Called only after the generated root hook has validated/published its slots.
 * Defer eager collection for this one owner's pending cycle, never pressure GC. */
int32_t experiment_gc_boundary(neoclr_aot_context *c, const neoclr_probe_frame *boundary) {
    if (!c || boundary != neoclr_root_probe_head_v1() || !boundary || boundary->context != c)
        return 3;
    if (EXPERIMENT_DEFER_BOUNDARIES && active_cycle && active_cycle->job && active_cycle->context == c) {
        cycle_owner(active_cycle);
        deferred_boundaries++;
        return 0;
    }
    return neoclr_gc_collect_v1(c, boundary);
}
static void *allocate(neoclr_aot_context *c, uint64_t bytes, uint32_t kind) {
    void *value = NULL;
    assert(!neoclr_gc_allocate_v1(&c->text, bytes, kind, &value));
    return value;
}
static void collect(collection_cycle *cycle, neoclr_aot_context *c) {
    assert(cycle_start(cycle, c));
    atomic_store(&cycle->job->proceed, true);
    assert(cycle_poll(cycle, true, false) == CYCLE_APPLIED);
}
static void compiled_case(int mode, bool pressure) {
    uint64_t heap[4096] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    uint64_t allocated = statistics.allocations, reclaimed = statistics.reclaimed_allocations;
    uint64_t handle;
    assert(!neoclr_bootstrap_callback_v1(mode, &handle, &c));
    void *descriptor;
    assert(!neoclr_gc_host_root_read_v1(&c, handle, &descriptor));
    /* A rooted chain ensures the deterministic gate occurs during actual tracing.
     * This host fixture is not itself generated guest work. */
    uint64_t *chain = NULL;
    for (unsigned i = 0; i < 128; i++) {
        uint64_t *node = allocate(&c, 16, NEOCLR_GC_OBJECT);
        node[1] = (uintptr_t)chain;
        chain = node;
    }
    uint64_t chain_handle;
    assert(!neoclr_gc_host_root_create_v1(&c, chain, &chain_handle));
    if (pressure) {
        allocate(&c, 24, NEOCLR_GC_BYTES);
        allocate(&c, 24, NEOCLR_GC_BYTES);
        c.text.capacity = c.text.used; /* The generated allocation must collect/retry. */
    }
    collection_cycle cycle;
    assert(cycle_init(&cycle));
    active_cycle = &cycle;
    unsigned rounds = mode == 0 && !pressure ? 40 : 1;
    for (unsigned round = 0; round < rounds; round++) {
        assert(cycle_start(&cycle, &c));
        cycle.job->pause_midwalk = true;
        atomic_store(&cycle.job->proceed, true);
        while (!atomic_load(&cycle.job->midwalk)) sched_yield();
        uint64_t before = statistics.allocations, collections = statistics.collections;
        int status = neoclr_invoke_void_callback_v1(handle, &c);
        assert(status == (mode == 2 ? 1 : 0));
        assert(!neoclr_root_probe_depth_v1());
        if (!mode) {
            assert(statistics.allocations == before + 1);
            uint64_t *counter = (void *)(uintptr_t)((uint64_t *)descriptor)[2];
            assert(*(int32_t *)(counter + 1) == (int32_t)round + 1);
            uint64_t *next = (void *)(uintptr_t)counter[2];
            assert(next && *(int32_t *)(next + 1) == (int32_t)round + 1);
        } else if (mode == 2) {
            assert(c.fault.code == 1 && c.fault.frame_count == 1);
        }
        atomic_store(&cycle.job->resume, true);
        bool stale = cycle.epoch != experiment_epoch;
        assert(!EXPERIMENT_DEFER_BOUNDARIES || stale == pressure);
        if (EXPERIMENT_DEFER_BOUNDARIES && pressure)
            assert(statistics.collections == collections + 1);
        assert(cycle_poll(&cycle, true, false) == (stale ? CYCLE_DISCARDED : CYCLE_APPLIED));
        if (mode == 2) {
            assert(c.fault.code == 1 && c.fault.frame_count == 1);
            assert(neoclr_invoke_void_callback_v1(handle, &c) == 3);
        }
        collect(&cycle, &c); /* Reclaim the previous replacement's floating garbage. */
        if (!mode) {
            uint64_t *counter = (void *)(uintptr_t)((uint64_t *)descriptor)[2];
            uint64_t *next = (void *)(uintptr_t)counter[2];
            assert(validated_bases(&c.text, counter, next));
            assert(*(int32_t *)(next + 1) == (int32_t)round + 1);
        }
    }
    assert(!neoclr_gc_host_root_release_v1(&c, handle));
    assert(!neoclr_gc_host_root_release_v1(&c, chain_handle));
    memset(&c.fault, 0, sizeof(c.fault));
    assert(!neoclr_gc_complete_invocation_v1(&c));
    collect(&cycle, &c);
    assert(!c.text.used);
    assert(statistics.allocations - allocated == statistics.reclaimed_allocations - reclaimed);
    assert(!neoclr_gc_entry_check_v1(&c));
    cycle_destroy(&cycle);
    active_cycle = NULL;
}
int main(void) {
    for (int mode = 0; mode < 3; mode++) compiled_case(mode, false);
    compiled_case(0, true);
    assert(EXPERIMENT_DEFER_BOUNDARIES ? deferred_boundaries == 41 && cycles_discarded == 1 : cycles_discarded == 41);
    printf("COMPILED: 41 allocating instance callbacks (one under pressure), static callback, generated fault and full reclamation passed; applied=%llu discarded=%llu deferred=%u\n",
        (unsigned long long)cycles_applied, (unsigned long long)cycles_discarded, deferred_boundaries);
    return 0;
}
