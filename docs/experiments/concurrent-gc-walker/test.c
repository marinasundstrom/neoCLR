#include <assert.h>
#include <sched.h>
#include <stdio.h>
#include <sys/mman.h>
#include <unistd.h>
#include "native-gc.c"
#include "snapshot.inc"
#include "reclaim.inc"

static pthread_t owner_thread;
static unsigned cleanups;
static const neoclr_probe_storage empty_storage = {0};
static void cleanup(void *state) {
    assert(pthread_equal(owner_thread, pthread_self()));
    ++*(unsigned *)state;
}
static void *allocate(neoclr_aot_context *c, uint64_t bytes, uint32_t kind) {
    void *result = NULL;
    assert(!neoclr_gc_allocate_v1(&c->text, bytes, kind, &result));
    return result;
}
static void wait_entered(snapshot *s) {
    while (!atomic_load(&s->entered)) sched_yield();
}
static int finish(gc_worker *worker, snapshot *s) {
    int result = -1;
    atomic_store(&s->proceed, true);
    assert(gc_worker_take(worker, true, false, &result));
    return result;
}
static unsigned random_word(unsigned *seed) {
    *seed = *seed * 1664525u + 1013904223u;
    return *seed;
}
static bool marked(snapshot *s, void *original) {
    block *b = (block *)relocate(s, original) - 1;
    return (b->state & MARKED) != 0;
}
static void compare_case(gc_worker *worker, unsigned seed, unsigned count, bool guard) {
    const size_t capacity = SNAPSHOT_BYTES;
    void *heap = mmap(NULL, capacity, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    assert(heap != MAP_FAILED);
    neoclr_aot_context c = {.text = {heap, capacity, 0}};
    void **objects = calloc(count, sizeof(*objects));
    bool *expected = calloc(count, sizeof(*expected));
    assert(objects && expected);
    for (unsigned i = 0; i < count; i++)
        objects[i] = allocate(&c, 24, i % 3 ? NEOCLR_GC_OBJECT : NEOCLR_GC_BYTES);
    for (unsigned i = 0; i < count; i++) {
        if (i % 3) {
            ((uint64_t *)objects[i])[1] = (uintptr_t)objects[random_word(&seed) % count];
            ((uint64_t *)objects[i])[2] = (uintptr_t)objects[random_word(&seed) % count];
        }
    }
    uint64_t roots[3] = {(uintptr_t)objects[0] + 1, (uintptr_t)objects[count / 2], 123};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &empty_storage, 0, "{}", 2);
    neoclr_probe_stack_roots_v2(&frame, 0, roots, 3, "requiredSpillLanes", 18);
    uint64_t handles[13];
    assert(!neoclr_gc_host_root_create_v1(&c, objects[1], &handles[0]));
    for (unsigned i = 1; i < 13; i++)
        assert(!neoclr_gc_host_root_create_owned_v1(&c, objects[random_word(&seed) % count],
                objects[random_word(&seed) % count], &handles[i]));
    neoclr_gc_resource resource = {0};
    unsigned before = cleanups;
    assert(!neoclr_gc_resource_attach_v1(&c, objects[count - 1], &resource, &cleanups, cleanup));
    snapshot *s = malloc(sizeof(*s));
    assert(s && capture(s, &c));
    assert(gc_worker_submit(worker, trace_snapshot, s));
    wait_entered(s);
    /* Guard all original allocation descriptors and payloads. Captured addresses
     * remain numeric identities only; the worker may read solely its private copy. */
    if (guard) assert(!mprotect(heap, capacity, PROT_NONE));
    assert(finish(worker, s) == 0);
    if (guard) assert(!mprotect(heap, capacity, PROT_READ | PROT_WRITE));
    assert(cleanups == before);
    for (unsigned i = 0; i < count; i++) expected[i] = marked(s, objects[i]);
    assert(!neoclr_gc_collect_v1(&c, &frame));
    for (unsigned i = 0; i < count; i++)
        assert(expected[i] == (bool)validated_bases(&c.text, objects[i], NULL));
    for (unsigned i = 0; i < s->pairs_count; i++) {
        void *original = (void *)(s->source + ((uintptr_t)s->pairs[i].owner - (uintptr_t)s->copy.text.data));
        assert((bool)s->pairs[i].retained == (bool)validated_bases(&c.text, original, NULL));
    }
    assert(cleanups == before + (expected[count - 1] ? 0u : 1u));
    for (unsigned i = 0; i < 13; i++) assert(!neoclr_gc_host_root_release_v1(&c, handles[i]));
    neoclr_probe_leave_v1(&frame);
    assert(!neoclr_gc_collect_v1(&c, NULL));
    assert(c.text.used == 0 && cleanups == before + 1);
    free(s->copy.text.data);
    free(s);
    free(expected);
    free(objects);
    assert(!munmap(heap, capacity));
}
static void conditional_regression(gc_worker *worker) {
    uint64_t heap[2048] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *a = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *b = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *d = allocate(&c, 8, NEOCLR_GC_BYTES);
    uint64_t handles[3];
    /* Register in an order requiring a fixed point through atomic leaves. */
    assert(!neoclr_gc_host_root_create_owned_v1(&c, b, d, &handles[0]));
    assert(!neoclr_gc_host_root_create_owned_v1(&c, a, b, &handles[1]));
    assert(!neoclr_gc_host_root_create_owned_v1(&c, d, a, &handles[2]));
    snapshot *s = malloc(sizeof(*s));
    assert(s);
    for (unsigned round = 0; round < 3; round++) {
        assert(capture(s, &c));
        assert(gc_worker_submit(worker, trace_snapshot, s));
        assert(finish(worker, s) == 0);
        assert(!marked(s, a) && !marked(s, b) && !marked(s, d));
        for (unsigned i = 0; i < 3; i++) assert(!s->pairs[i].retained);
        free(s->copy.text.data);
    }
    uint64_t root = (uintptr_t)a;
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &empty_storage, 0, "{}", 2);
    neoclr_probe_stack_roots_v2(&frame, 0, &root, 1, "requiredSpillLanes", 18);
    assert(capture(s, &c));
    assert(gc_worker_submit(worker, trace_snapshot, s));
    assert(finish(worker, s) == 0);
    assert(marked(s, a) && marked(s, b) && marked(s, d));
    assert(!neoclr_gc_collect_v1(&c, &frame));
    assert(validated_bases(&c.text, a, b) && validated_bases(&c.text, d, NULL));
    free(s->copy.text.data);
    neoclr_probe_leave_v1(&frame);
    assert(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    for (unsigned i = 0; i < 3; i++) assert(!neoclr_gc_host_root_release_v1(&c, handles[i]));
    free(s);
}
static void specialized_roots(gc_worker *worker) {
    uint64_t heap[2048] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *child = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *intern = allocate(&c, 8, NEOCLR_GC_INTERN);
    *(uint64_t *)intern = (uintptr_t)child;
    void *static_value = allocate(&c, 16, NEOCLR_GC_OBJECT);
    c.static_storage = static_value;
    void *message = allocate(&c, 8, NEOCLR_GC_TEXT);
    c.fault.code = 4;
    c.fault.message = message;
    snapshot *s = malloc(sizeof(*s));
    assert(s && capture(s, &c));
    assert(gc_worker_submit(worker, trace_snapshot, s));
    assert(finish(worker, s) == 0);
    assert(marked(s, child) && marked(s, intern) && marked(s, static_value) && marked(s, message));
    assert(!neoclr_gc_collect_v1(&c, NULL));
    assert(validated_bases(&c.text, child, intern) && validated_bases(&c.text, static_value, message));
    free(s->copy.text.data);
    free(s);
}
static void failure_and_mutation(gc_worker *worker) {
    uint64_t heap[2048] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *a = allocate(&c, 16, NEOCLR_GC_OBJECT);
    void *b = allocate(&c, 8, NEOCLR_GC_BYTES);
    ((uint64_t *)a)[1] = (uintptr_t)b;
    uint64_t root = (uintptr_t)a;
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &empty_storage, 0, "{}", 2);
    neoclr_probe_stack_roots_v2(&frame, 0, &root, 1, "requiredSpillLanes", 18);
    snapshot *s = malloc(sizeof(*s));
    assert(s && capture(s, &c));
    assert(gc_worker_submit(worker, trace_snapshot, s));
    wait_entered(s);
    ((uint64_t *)a)[1] = 0;
    root = 0;
    allocate(&c, 64, NEOCLR_GC_BYTES);
    assert(finish(worker, s) == 0);
    assert(marked(s, a) && marked(s, b)); /* Snapshot, not current graph. */
    assert(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    free(s->copy.text.data);
    assert(capture(s, &c));
    assert(gc_worker_submit(worker, trace_snapshot, s));
    wait_entered(s);
    int status;
    assert(gc_worker_take(worker, true, true, &status) && status == 7);
    free(s->copy.text.data);
    neoclr_probe_leave_v1(&frame);
    free(s);
}
static void arrays_and_invalid_descriptor(gc_worker *worker) {
    uint64_t heap[4096] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    void *live = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *dead = allocate(&c, 8, NEOCLR_GC_BYTES);
    uint64_t *records = allocate(&c, 42, NEOCLR_GC_RECORDS);
    records[0] = UINT64_C(0x80000005);
    records[1] = 2;
    records[2] = 1;
    records[3] = (uintptr_t)live;
    records[4] = (uintptr_t)dead;
    ((unsigned char *)records)[40] = 1;
    ((unsigned char *)records)[41] = 0;
    uint64_t *strings = allocate(&c, 34, NEOCLR_GC_STRINGS);
    strings[0] = UINT64_C(0x80000004);
    strings[1] = 2;
    strings[2] = (uintptr_t)live;
    strings[3] = (uintptr_t)dead;
    ((unsigned char *)strings)[32] = 1;
    ((unsigned char *)strings)[33] = 0;
    /* Overflow the bounded intern-seed cache, exercising the reset walk too. */
    for (unsigned i = 0; i < 34; i++) {
        uint64_t *intern = allocate(&c, 8, NEOCLR_GC_INTERN);
        *intern = (uintptr_t)live;
    }
    neoclr_probe_storage slot = {&records, 8, 0}, transient = {&strings, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    neoclr_probe_transient_v2(&frame, 1, &transient, 1, "{}", 2);
    snapshot *s = malloc(sizeof(*s));
    assert(s && capture(s, &c));
    assert(gc_worker_submit(worker, trace_snapshot, s));
    assert(finish(worker, s) == 0);
    assert(marked(s, live) && !marked(s, dead) && marked(s, records) && marked(s, strings));
    assert(!neoclr_gc_collect_v1(&c, &frame));
    assert(validated_bases(&c.text, live, records) && validated_bases(&c.text, strings, NULL));
    assert(!validated_bases(&c.text, dead, NULL));
    free(s->copy.text.data);
    assert(capture(s, &c));
    uint64_t *copied_records = relocate(s, records);
    copied_records[1] = UINT64_MAX;
    assert(gc_worker_submit(worker, trace_snapshot, s));
    assert(finish(worker, s) == 3);
    assert(records[1] == 2); /* Failure never touched live descriptors. */
    free(s->copy.text.data);
    assert(capture(s, &c));
    assert(gc_worker_submit(worker, trace_snapshot, s));
    assert(finish(worker, s) == 0); /* Worker TLS and job state reset after error. */
    free(s->copy.text.data);
    free(s);
    neoclr_probe_leave_v1(&frame);
}
#include "parked-tests.inc"
#include "publication-tests.inc"
#include "reclaim-tests.inc"
int main(void) {
    owner_thread = pthread_self();
    reclamation_tests();
    gc_worker worker;
    assert(!gc_worker_init(&worker));
    for (unsigned mode = 0; mode < 3; mode++) parked_case(&worker, mode);
    parked_limits();
    host_read_after_capture(&worker);
    resume_after_capture(&worker);
    reused_address_after_capture(&worker);
    conditional_regression(&worker);
    specialized_roots(&worker);
    failure_and_mutation(&worker);
    arrays_and_invalid_descriptor(&worker);
    for (unsigned seed = 1; seed <= 100; seed++) compare_case(&worker, seed, 64, true);
    compare_case(&worker, 123, 3000, true);
    gc_worker_destroy(&worker);
    puts("Parked roots: three guarded oracle cases, expiry and capacity rejection passed");
    puts("101 guarded graph/oracle comparisons passed; conditional cycle/fixed point, intern/static/fault roots, initialized arrays, malformed descriptors, mutation, cancellation passed");
}
