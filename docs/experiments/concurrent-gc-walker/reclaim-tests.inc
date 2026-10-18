static void cycle_run(collection_cycle *cycle, neoclr_aot_context *c) {
    assert(cycle_start(cycle, c));
    atomic_store(&cycle->job->proceed, true);
    assert(cycle_poll(cycle, true, false) == CYCLE_APPLIED);
    assert(cycle_poll(cycle, false, false) == CYCLE_EMPTY);
}
static void live_reclamation(void) {
    uint64_t heap[4096] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *parent = allocate(&c, 16, NEOCLR_GC_OBJECT);
    void *child = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *owner = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *value = allocate(&c, 8, NEOCLR_GC_BYTES);
    *(uint64_t *)child = 42;
    ((uint64_t *)parent)[1] = (uintptr_t)child;
    uint64_t handle;
    assert(!neoclr_gc_host_root_create_owned_v1(&c, owner, value, &handle));
    neoclr_gc_resource resource = {0};
    unsigned cleaned = 0;
    assert(!neoclr_gc_resource_attach_v1(&c, owner, &resource, &cleaned, cleanup));
    void *root = parent;
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    wait_entered(cycle.job);
    assert(!cycle_start(&cycle, &c));
    assert(cycle_poll(&cycle, false, false) == CYCLE_PENDING);
    /* Ordinary managed progress transfers an already-reachable reference and
     * allocates while the worker is outstanding; it does not invalidate. */
    ((uint64_t *)parent)[1] = 0;
    root = child;
    void *fresh = allocate(&c, 16, NEOCLR_GC_OBJECT);
    ((uint64_t *)fresh)[1] = (uintptr_t)child;
    uint64_t reclaimed = statistics.reclaimed_allocations;
    atomic_store(&cycle.job->proceed, true);
    assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
    assert(statistics.reclaimed_allocations == reclaimed + 2 && cleaned == 1);
    assert(!validated_bases(&c.text, owner, NULL) && !validated_bases(&c.text, value, NULL));
    assert(*(uint64_t *)value == UINT64_C(0xdddddddddddddddd));
    assert(validated_bases(&c.text, parent, child) && validated_bases(&c.text, fresh, NULL));
    assert(*(uint64_t *)child == 42 && ((uint64_t *)fresh)[1] == (uintptr_t)child);
    void *ignored;
    assert(neoclr_gc_host_root_read_v1(&c, handle, &ignored) == 3);
    assert(!neoclr_gc_host_root_release_v1(&c, handle));
    void *reused = allocate(&c, 8, NEOCLR_GC_BYTES);
    assert(reused == owner); /* Space freed by the concurrent cycle is reusable. */
    /* Fresh/unlinked parent were conservatively retained, not leaked forever. */
    cycle_run(&cycle, &c);
    assert(validated_bases(&c.text, child, NULL));
    assert(!validated_bases(&c.text, parent, NULL) && !validated_bases(&c.text, fresh, NULL));
    root = NULL;
    cycle_run(&cycle, &c);
    assert(!c.text.used && cleaned == 1);
    cycle_destroy(&cycle);
    neoclr_probe_leave_v1(&frame);
}
static void free_span_allocation(void) {
    uint64_t heap[1024] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *a = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *hole = allocate(&c, 256, NEOCLR_GC_BYTES);
    void *b = allocate(&c, 8, NEOCLR_GC_BYTES);
    uint64_t roots[2] = {(uintptr_t)a, (uintptr_t)b};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &empty_storage, 0, "{}", 2);
    neoclr_probe_stack_roots_v2(&frame, 0, roots, 2, "requiredSpillLanes", 18);
    assert(!neoclr_gc_collect_v1(&c, &frame));
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    void *fresh = allocate(&c, 16, NEOCLR_GC_BYTES);
    assert(fresh == hole);
    *(uint64_t *)fresh = 99;
    void *fresh_second = allocate(&c, 16, NEOCLR_GC_BYTES);
    assert((uintptr_t)fresh_second > (uintptr_t)hole &&
        (uintptr_t)fresh_second < (uintptr_t)hole + 256);
    *(uint64_t *)fresh_second = 101;
    void *tail = allocate(&c, 512, NEOCLR_GC_BYTES);
    assert((uintptr_t)tail > (uintptr_t)b);
    *(uint64_t *)tail = 103;
    atomic_store(&cycle.job->proceed, true);
    assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
    assert(validated_bases(&c.text, fresh, NULL) && *(uint64_t *)fresh == 99);
    assert(validated_bases(&c.text, fresh_second, tail));
    assert(*(uint64_t *)fresh_second == 101 && *(uint64_t *)tail == 103);
    roots[0] = roots[1] = 0;
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    cycle_destroy(&cycle);
    neoclr_probe_leave_v1(&frame);
}
static void parked_reclamation(void) {
    uint64_t heap[1024] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *owner = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *value = allocate(&c, 8, NEOCLR_GC_BYTES);
    neoclr_probe_storage slot = {&value, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    neoclr_parked_roots parked = {0};
    assert(!neoclr_gc_roots_park_owned_v1(&c, owner, &parked));
    collection_cycle cycle;
    assert(cycle_init(&cycle));
    cycle_run(&cycle, &c);
    assert(!c.text.used && parked.expired && !parked.head && !parked.owner);
    neoclr_roots_discard_v1(&parked);
    cycle_destroy(&cycle);
}
/* Each event must discard without touching current heap contents or counters. */
static void rejected_result(unsigned event) {
    uint64_t heap[1024] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *owner = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *value = allocate(&c, 8, NEOCLR_GC_BYTES);
    uint64_t handle;
    assert(!neoclr_gc_host_root_create_owned_v1(&c, owner, value, &handle));
    neoclr_gc_resource resource = {0};
    unsigned cleaned = 0;
    neoclr_parked_roots parked = {0};
    neoclr_probe_storage slot = {&value, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    if (event == 1) assert(!neoclr_gc_roots_park_owned_v1(&c, owner, &parked));
    if (event == 0) value = NULL; /* Only the conditional handle can rescue it. */
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    wait_entered(cycle.job);
    uint64_t extra = 0;
    switch (event) {
    case 0: assert(!neoclr_gc_host_root_read_v1(&c, handle, &value)); break;
    case 1: neoclr_roots_resume_v1(&parked); break;
    case 2: assert(!neoclr_gc_host_root_replace_v1(&c, handle, value)); break;
    case 3: assert(!neoclr_gc_host_root_create_v1(&c, value, &extra)); break;
    case 4: assert(!neoclr_gc_host_root_release_v1(&c, handle)); handle = 0; break;
    case 5: assert(!neoclr_gc_resource_attach_v1(&c, owner, &resource, &cleaned, cleanup)); break;
    case 6: {
        value = NULL;
        assert(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
        void *replacement = allocate(&c, 8, NEOCLR_GC_BYTES);
        assert(replacement == owner);
        allocate(&c, 8, NEOCLR_GC_BYTES);
        assert(c.text.used == cycle.job->copy.text.used);
        value = replacement;
        break;
    }
    case 7: break; /* Explicit cancellation, including a completed result. */
    case 8: ((block *)(void *)cycle.job->copy.text.data)->span = 1; break;
    case 9: assert(neoclr_gc_complete_invocation_v1(&c) == 3); break;
    default: abort();
    }
    unsigned char before[sizeof(heap)];
    memcpy(before, heap, sizeof(heap));
    uint64_t collections = statistics.collections, reclaimed = statistics.reclaimed_allocations;
    unsigned before_cleanup = cleaned;
    atomic_store(&cycle.job->proceed, true);
    if (event == 7) {
        for (;;) {
            assert(!pthread_mutex_lock(&cycle.worker.mutex));
            bool ready = cycle.worker.state == 2;
            assert(!pthread_mutex_unlock(&cycle.worker.mutex));
            if (ready) break;
            sched_yield();
        }
    }
    assert(cycle_poll(&cycle, true, event == 7) == CYCLE_DISCARDED);
    assert(!memcmp(before, heap, sizeof(heap)) && statistics.collections == collections &&
           statistics.reclaimed_allocations == reclaimed && cleaned == before_cleanup);
    assert(validated_bases(&c.text, value, NULL));
    if (handle) assert(!neoclr_gc_host_root_release_v1(&c, handle));
    if (extra) assert(!neoclr_gc_host_root_release_v1(&c, extra));
    neoclr_probe_leave_v1(&frame);
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    assert(cleaned == (event == 5 ? 1u : 0u));
    cycle_destroy(&cycle);
}
static uint64_t graph_outcome(unsigned seed, bool concurrent) {
    uint64_t heap[4096] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *objects[64];
    for (unsigned i = 0; i < 64; i++) objects[i] = allocate(&c, 24, i % 3 ? NEOCLR_GC_OBJECT : NEOCLR_GC_BYTES);
    for (unsigned i = 0; i < 64; i++) if (i % 3) {
        ((uint64_t *)objects[i])[1] = (uintptr_t)objects[random_word(&seed) % 64];
        ((uint64_t *)objects[i])[2] = (uintptr_t)objects[random_word(&seed) % 64];
    }
    uint64_t roots[2] = {(uintptr_t)objects[0] + 1, (uintptr_t)objects[32]};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &empty_storage, 0, "{}", 2);
    neoclr_probe_stack_roots_v2(&frame, 0, roots, 2, "requiredSpillLanes", 18);
    uint64_t handles[12];
    for (unsigned i = 0; i < 12; i++)
        assert(!neoclr_gc_host_root_create_owned_v1(&c, objects[random_word(&seed) % 64], objects[random_word(&seed) % 64], &handles[i]));
    collection_cycle cycle;
    assert(cycle_init(&cycle));
    uint64_t reclaimed = statistics.reclaimed_allocations, bytes = statistics.reclaimed_bytes;
    if (concurrent) cycle_run(&cycle, &c);
    else assert(!neoclr_gc_collect_v1(&c, &frame));
    uint64_t mask = 0, survivors = 0;
    for (unsigned i = 0; i < 64; i++) if (validated_bases(&c.text, objects[i], NULL)) {
        mask |= UINT64_C(1) << i;
        survivors++;
    }
    assert(statistics.reclaimed_allocations - reclaimed == 64 - survivors);
    assert(statistics.reclaimed_bytes - bytes == (64 - survivors) * 24);
    for (unsigned i = 0; i < 12; i++) assert(!neoclr_gc_host_root_release_v1(&c, handles[i]));
    neoclr_probe_leave_v1(&frame);
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    cycle_destroy(&cycle);
    return mask;
}
static void destroy_outstanding(void) {
    uint64_t heap[128] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *object = allocate(&c, 8, NEOCLR_GC_BYTES);
    *(uint64_t *)object = 123;
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    wait_entered(cycle.job);
    uint64_t reclaimed = statistics.reclaimed_allocations;
    cycle_destroy(&cycle);
    assert(statistics.reclaimed_allocations == reclaimed && *(uint64_t *)object == 123);
    assert(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
}
static void midwalk_progress(void) {
    uint64_t heap[4096] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *chain[128];
    for (unsigned i = 0; i < 128; i++) chain[i] = allocate(&c, 16, NEOCLR_GC_OBJECT);
    for (unsigned i = 0; i < 127; i++) ((uint64_t *)chain[i])[1] = (uintptr_t)chain[i + 1];
    void *garbage = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *root = chain[0];
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    /* Owner writes the test option before releasing the worker's entry gate. */
    cycle.job->pause_midwalk = true;
    atomic_store(&cycle.job->proceed, true);
    while (!atomic_load(&cycle.job->midwalk)) sched_yield();
    ((uint64_t *)chain[63])[1] = 0;
    root = chain[127];
    void *fresh = allocate(&c, 16, NEOCLR_GC_OBJECT);
    ((uint64_t *)fresh)[1] = (uintptr_t)root;
    atomic_store(&cycle.job->resume, true);
    assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
    assert(!validated_bases(&c.text, garbage, NULL));
    for (unsigned i = 0; i < 128; i++) assert(validated_bases(&c.text, chain[i], NULL));
    assert(validated_bases(&c.text, fresh, NULL));
    cycle_run(&cycle, &c);
    assert(!validated_bases(&c.text, fresh, NULL));
    for (unsigned i = 0; i < 127; i++) assert(!validated_bases(&c.text, chain[i], NULL));
    assert(validated_bases(&c.text, chain[127], NULL));
    root = NULL;
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    cycle_destroy(&cycle);
    neoclr_probe_leave_v1(&frame);
    puts("MIDWALK: reference transfer/allocation after 63 scanned objects; reclamation and eventual empty heap passed");
}
/* Corrupt the live heap without advancing the epoch to exercise validation,
 * including a valid captured prefix followed by an invalid appended block. */
static void rejected_live_descriptor(unsigned event) {
    uint64_t heap[512] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *a = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *hole = allocate(&c, 256, NEOCLR_GC_BYTES);
    void *b = allocate(&c, 8, NEOCLR_GC_BYTES);
    uint64_t roots[2] = {(uintptr_t)a, (uintptr_t)b};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &empty_storage, 0, "{}", 2);
    neoclr_probe_stack_roots_v2(&frame, 0, roots, 2, "requiredSpillLanes", 18);
    assert(!neoclr_gc_collect_v1(&c, &frame));
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    void *fresh = allocate(&c, 16, NEOCLR_GC_BYTES);
    assert(fresh == hole);
    void *tail = allocate(&c, 512, NEOCLR_GC_BYTES);
    unsigned char intact[sizeof(heap)], before[sizeof(heap)];
    memcpy(intact, heap, sizeof(heap));
    uint64_t used = c.text.used;
    switch (event) {
    case 0: ((block *)tail - 1)->span = 1; break;
    case 1: ((block *)b - 1)->bytes = 2; break;
    case 2: ((block *)a - 1)->state = ALLOCATED | MARKED; break;
    case 3: ((block *)fresh - 1)->span = 328; break; /* Cross old free span. */
    case 4: c.text.used = c.text.capacity + 8; break;
    default: abort();
    }
    memcpy(before, heap, sizeof(heap));
    neoclr_gc_statistics stats = statistics;
    atomic_store(&cycle.job->proceed, true);
    assert(cycle_poll(&cycle, true, false) == CYCLE_DISCARDED);
    assert(!memcmp(before, heap, sizeof(heap)));
    assert(!memcmp(&stats, &statistics, sizeof(stats)));
    memcpy(heap, intact, sizeof(heap));
    c.text.used = used;
    neoclr_probe_leave_v1(&frame);
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    cycle_destroy(&cycle);
}
static void strong_handle_read(unsigned event) {
    uint64_t heap[512] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *value = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *garbage = allocate(&c, 8, NEOCLR_GC_BYTES);
    *(uint64_t *)value = 123;
    uint64_t handle, extra = 0;
    assert(!neoclr_gc_host_root_create_v1(&c, value, &handle));
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    wait_entered(cycle.job);
    if (event == 1) assert(!neoclr_gc_host_root_replace_v1(&c, handle, garbage));
    if (event == 2) assert(!neoclr_gc_host_root_create_v1(&c, garbage, &extra));
    void *read = NULL;
    if (event == 3) assert(neoclr_gc_host_root_read_v1(&c, handle, NULL) == 3);
    else assert(!neoclr_gc_host_root_read_v1(&c, handle, &read));
    assert((cycle.epoch == experiment_epoch) == (event == 0));
    unsigned char before[sizeof(heap)];
    memcpy(before, heap, sizeof(heap));
    uint64_t collections = statistics.collections;
    atomic_store(&cycle.job->proceed, true);
    if (!event) {
        assert(read == value && cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
        assert(*(uint64_t *)read == 123);
        assert(!validated_bases(&c.text, garbage, NULL));
    } else {
        assert(cycle_poll(&cycle, true, false) == CYCLE_DISCARDED);
        assert(!memcmp(before, heap, sizeof(heap)) && statistics.collections == collections);
    }
    assert(!neoclr_gc_host_root_release_v1(&c, handle));
    if (extra) assert(!neoclr_gc_host_root_release_v1(&c, extra));
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    cycle_destroy(&cycle);
}
static void callback_handle_read(unsigned event) {
    uint64_t heap[512] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *value = allocate(&c, event == 6 ? 16 : 24, NEOCLR_GC_OBJECT);
    *(uint64_t *)value = event == 5 ? 0 : UINT32_MAX;
    void *garbage = allocate(&c, 24, NEOCLR_GC_OBJECT);
    *(uint64_t *)garbage = UINT32_MAX;
    uint64_t handle;
    if (event == 7) assert(!neoclr_gc_host_root_create_owned_v1(&c, value, value, &handle));
    else assert(!neoclr_gc_host_root_create_v1(&c, value, &handle));
    bool entered = event >= 1 && event <= 4;
    bool suspended = event == 1 || event == 3 || event == 4;
    neoclr_probe_frame frame, wrong = {0};
    if (entered) neoclr_probe_enter_v3(&frame, &c, 0, &empty_storage, 0, "{}", 2);
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    wait_entered(cycle.job);
    if (event == 9) c.fault.code = 3;
    if (event == 10) assert(!neoclr_gc_host_root_replace_v1(&c, handle, garbage));
    void *output = NULL;
    int status;
    if (suspended)
        status = neoclr_gc_callback_read_suspended_v1(&c, handle, &output,
            event == 3 ? NULL : event == 4 ? &wrong : &frame);
    else status = neoclr_gc_callback_read_v1(&c, handle, event == 8 ? NULL : &output);
    assert(status == (event < 2 || event == 10 ? 0 : 3));
    if (!status) assert(output == (event == 10 ? garbage : value));
    else assert(!output);
    assert((cycle.epoch == experiment_epoch) == (event < 2));
    unsigned char before[sizeof(heap)];
    memcpy(before, heap, sizeof(heap));
    uint64_t collections = statistics.collections;
    atomic_store(&cycle.job->proceed, true);
    if (event < 2) {
        assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
        assert(*(uint64_t *)output == UINT32_MAX);
        assert(!validated_bases(&c.text, garbage, NULL));
    } else {
        assert(cycle_poll(&cycle, true, false) == CYCLE_DISCARDED);
        assert(!memcmp(before, heap, sizeof(heap)) && statistics.collections == collections);
    }
    c.fault.code = 0;
    if (entered) neoclr_probe_leave_v1(&frame);
    assert(!neoclr_gc_host_root_release_v1(&c, handle));
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    cycle_destroy(&cycle);
}
#include "callback-churn.inc"
static void executing_callback_churn(void) {
    uint64_t heap[4096] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    uint64_t allocated = statistics.allocations, reclaimed = statistics.reclaimed_allocations;
    uint64_t *callback = churn_create(&c, 128);
    uint64_t handle, retired = 0, next = 128, checksum = 1;
    assert(!neoclr_gc_host_root_create_v1(&c, callback, &handle));
    collection_cycle cycle;
    assert(cycle_init(&cycle));
    for (unsigned round = 0; round < 80; round++) {
        assert(cycle_start(&cycle, &c));
        if (!round) cycle.job->pause_midwalk = true;
        atomic_store(&cycle.job->proceed, true);
        if (!round) while (!atomic_load(&cycle.job->midwalk)) sched_yield();
        void *read;
        assert(!neoclr_gc_callback_read_v1(&c, handle, &read) && read == callback);
        churn_invoke(&c, read, 17, &retired, &next, &checksum);
        churn_check(callback, 128, retired);
        assert(c.text.used <= (128 + 2 * 17 + 1) * 56);
        if (!round) atomic_store(&cycle.job->resume, true);
        assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
        churn_check(callback, 128, retired);
    }
    assert(retired == 1360 && next == 1488);
    assert(!neoclr_gc_host_root_release_v1(&c, handle));
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    assert(statistics.allocations - allocated == statistics.reclaimed_allocations - reclaimed);
    cycle_destroy(&cycle);
    puts("CALLBACK CHURN: 80 invoked native callbacks, midwalk mutation, 1360 replacements, bounded extent and full reclamation passed");
}
/* A synchronous pressure collection may run while the worker still traces its
 * private copy. Helper-only roots must survive, and old marks must never apply
 * to reused addresses. The impossible-size control must not collect at all. */
static void rooted_pressure_during_trace(unsigned event) {
    uint64_t heap[132 * 7] = {0}; /* Exactly 132 descriptor + 24-byte blocks. */
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    uint64_t allocated = statistics.allocations;
    uint64_t reclaimed = statistics.reclaimed_allocations;
    uint64_t *callback = churn_create(&c, 128);
    void *borrowed = allocate(&c, 24, NEOCLR_GC_BYTES);
    *(uint64_t *)borrowed = 9876;
    void *garbage = allocate(&c, 24, NEOCLR_GC_BYTES);
    allocate(&c, 24, NEOCLR_GC_BYTES);
    assert(c.text.used == c.text.capacity);
    neoclr_probe_storage slot = {&callback, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    collection_cycle cycle;
    assert(cycle_init(&cycle) && cycle_start(&cycle, &c));
    cycle.job->pause_midwalk = true;
    atomic_store(&cycle.job->proceed, true);
    while (!atomic_load(&cycle.job->midwalk)) sched_yield();
    uint64_t collections = statistics.collections;
    const void *values[] = {borrowed};
    void *fresh = NULL;
    uint64_t bytes = event == 0 ? 24 : event == 1 ? 200 : sizeof(heap);
    int32_t status = neoclr_gc_allocate_rooted_values_v1(
        &c, &frame, values, 1, bytes, NEOCLR_GC_BYTES, &fresh);
    assert(status == (event == 0 ? 0 : NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT));
    assert(statistics.collections == collections + (event < 2));
    assert(*(uint64_t *)borrowed == 9876);
    churn_check(callback, 128, 0);
    if (!event) {
        assert(fresh == garbage); /* Pressure collection and immediate reuse. */
        *(uint64_t *)fresh = 4321;
    } else assert(!fresh);
    uint64_t saved[132 * 7];
    memcpy(saved, heap, sizeof(heap));
    uint64_t used = c.text.used;
    uint64_t freed = statistics.reclaimed_allocations;
    atomic_store(&cycle.job->resume, true);
    assert(cycle_poll(&cycle, true, false) ==
        (event < 2 ? CYCLE_DISCARDED : CYCLE_APPLIED));
    if (event < 2) {
        assert(!memcmp(saved, heap, sizeof(heap)) && c.text.used == used);
        assert(statistics.reclaimed_allocations == freed);
        assert(*(uint64_t *)borrowed == 9876);
        if (!event) assert(*(uint64_t *)fresh == 4321);
    }
    churn_check(callback, 128, 0);
    callback = NULL;
    cycle_run(&cycle, &c);
    assert(!c.text.used);
    assert(statistics.allocations - allocated == statistics.reclaimed_allocations - reclaimed);
    cycle_destroy(&cycle);
    neoclr_probe_leave_v1(&frame);
}
static void lifecycle_pending(collection_cycle *cycle, neoclr_aot_context *c) {
    assert(cycle_start(cycle, c));
    cycle->job->pause_midwalk = true;
    atomic_store(&cycle->job->proceed, true);
    while (!atomic_load(&cycle->job->midwalk)) sched_yield();
}
static void lifecycle_discard(collection_cycle *cycle, neoclr_aot_context *c) {
    void *saved = malloc(c->text.capacity);
    assert(saved);
    memcpy(saved, c->text.data, c->text.capacity);
    uint64_t used = c->text.used, collections = statistics.collections;
    uint64_t reclaimed = statistics.reclaimed_allocations;
    atomic_store(&cycle->job->resume, true);
    assert(cycle_poll(cycle, true, false) == CYCLE_DISCARDED);
    assert(used == c->text.used && collections == statistics.collections);
    assert(reclaimed == statistics.reclaimed_allocations);
    assert(!memcmp(saved, c->text.data, c->text.capacity));
    free(saved);
}
/* Exercise native lifetime transitions together, including a fault that outlives
 * the invocation. This does not execute a Raven async state machine. */
static void suspended_invocation_lifecycle(bool abandon) {
    uint64_t heap[2048] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    uint64_t allocated = statistics.allocations, reclaimed = statistics.reclaimed_allocations;
    uint64_t *callback = churn_create(&c, 128);
    void *message = allocate(&c, 8, NEOCLR_GC_TEXT);
    memcpy(message, "failure", 8);
    void *intern = allocate(&c, 8, NEOCLR_GC_INTERN);
    c.static_storage = allocate(&c, 16, NEOCLR_GC_OBJECT);
    uint64_t handle;
    assert(!neoclr_gc_host_root_create_v1(&c, callback, &handle));
    neoclr_gc_resource resource = {0};
    unsigned cleaned = 0;
    assert(!neoclr_gc_resource_attach_v1(&c, callback, &resource, &cleaned, cleanup));
    neoclr_probe_storage slots[] = {{&callback, 8, 0}, {&message, 8, 0}};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, slots, 2, "{}", 2);
    collection_cycle cycle;
    assert(cycle_init(&cycle));
    neoclr_parked_roots parked = {0};
    lifecycle_pending(&cycle, &c);
    assert(!neoclr_gc_roots_park_owned_v1(&c, callback, &parked));
    lifecycle_discard(&cycle, &c);
    cycle_run(&cycle, &c); /* Strong owner keeps the suspended message alive. */
    assert(validated_bases(&c.text, message, intern) && !cleaned && !parked.expired);
    lifecycle_pending(&cycle, &c);
    if (abandon) neoclr_roots_discard_v1(&parked);
    else neoclr_roots_resume_v1(&parked);
    lifecycle_discard(&cycle, &c);
    assert(!parked.head && !parked.owner && !parked.owner_context);
    if (!abandon) {
        /* Publish an already-rooted value as the fault, then unwind its frame.
         * Snapshot reachability covers this transfer without another barrier. */
        lifecycle_pending(&cycle, &c);
        c.fault.code = 4;
        c.fault.message = message;
        neoclr_probe_leave_v1(&frame);
        atomic_store(&cycle.job->resume, true);
        assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
        assert(!memcmp(message, "failure", 8));
    }
    lifecycle_pending(&cycle, &c);
    assert(!neoclr_gc_complete_invocation_v1(&c));
    assert(!c.static_storage && ((block *)intern - 1)->kind == NEOCLR_GC_BYTES);
    lifecycle_discard(&cycle, &c);
    assert(!neoclr_gc_host_root_release_v1(&c, handle));
    cycle_run(&cycle, &c);
    assert(cleaned == 1 && !resource.owner);
    uint64_t survivors = 0;
    for (uint64_t at = 0; at < c.text.used;) {
        const block *b = (const void *)(c.text.data + at);
        survivors += (b->state & ALLOCATED) != 0;
        at += b->span;
    }
    assert(survivors == (abandon ? 0u : 1u));
    if (!abandon) {
        assert(validated_bases(&c.text, message, NULL) && !memcmp(message, "failure", 8));
        c.fault.code = 0;
        c.fault.message = NULL;
        cycle_run(&cycle, &c);
    }
    assert(!c.text.used && cleaned == 1);
    assert(statistics.allocations - allocated == statistics.reclaimed_allocations - reclaimed);
    assert(!neoclr_gc_entry_check_v1(&c));
    cycle_destroy(&cycle);
}
static void reclamation_tests(void) {
    suspended_invocation_lifecycle(false);
    suspended_invocation_lifecycle(true);
    puts("LIFECYCLE: park, resume/discard, fault transfer, successful completion, exactly-once cleanup and full reclamation passed");
    for (unsigned event = 0; event < 3; event++) rooted_pressure_during_trace(event);
    puts("PRESSURE: midwalk rooted allocation, helper-root survival, same-address reuse, failed retry and impossible-size control passed");
    executing_callback_churn();
    for (unsigned event = 0; event < 11; event++) callback_handle_read(event);
    for (unsigned event = 0; event < 4; event++) strong_handle_read(event);
    for (unsigned event = 0; event < 5; event++) rejected_live_descriptor(event);
    midwalk_progress();
    destroy_outstanding();
    live_reclamation();
    free_span_allocation();
    parked_reclamation();
    for (unsigned event = 0; event < 10; event++) rejected_result(event);
    for (unsigned seed = 1; seed <= 100; seed++)
        assert(graph_outcome(seed, true) == graph_outcome(seed, false));
    printf("POC applied=%llu discarded=%llu reclaimedPayloadBytes=%llu\n",
        (unsigned long long)cycles_applied, (unsigned long long)cycles_discarded,
        (unsigned long long)cycle_reclaimed_bytes);
    puts("RECLAMATION: live bytes freed; 100 graph/oracle survivor and byte-count comparisons; stale/error/cancel rejection; floating garbage, free-span allocation and parked expiry passed");
}
