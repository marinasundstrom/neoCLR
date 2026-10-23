#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#ifdef NEOCLR_TEST_THREADS
#include <pthread.h>
#endif
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)

/* Compare the indexed owner against the unchanged external first-fit allocator.
 * Address offsets, pressure outcomes, zeroing and live contents must agree. */
static int differential(void) {
    uint64_t heaps[2][8192] = {{0}};
    neoclr_aot_context c[2] = {
        {.text = {(unsigned char *)heaps[0], sizeof(heaps[0]), 0}},
        {.text = {(unsigned char *)heaps[1], sizeof(heaps[1]), 0}}
    };
    void *roots[2][32] = {{0}};
    uint32_t sizes[32] = {0};
    neoclr_probe_storage slots[2][32];
    neoclr_probe_frame frames[2] = {{.context = &c[0], .storage = slots[0], .storage_count = 32},
                                   {.context = &c[1], .storage = slots[1], .storage_count = 32}};
    for (unsigned side = 0; side < 2; side++)
        for (unsigned i = 0; i < 32; i++) slots[side][i] = (neoclr_probe_storage){&roots[side][i], 8, 0};
    CHECK(!neoclr_gc_heap_acquire_v1(&c[1].text));
    uint32_t random = 0x98a731u;
    for (unsigned round = 0; round < 4000; round++) {
        random ^= random << 13; random ^= random >> 17; random ^= random << 5;
        unsigned slot = random % 32, bytes = 1 + ((random >> 8) % 2048);
        roots[0][slot] = roots[1][slot] = NULL;
        if (round % 7 == 0) {
            CHECK(!neoclr_gc_collect_v1(&c[0], &frames[0]));
            CHECK(!neoclr_gc_collect_v1(&c[1], &frames[1]));
        }
        int32_t a = neoclr_gc_allocate_v1(&c[0].text, bytes, NEOCLR_GC_BYTES, &roots[0][slot]);
        int32_t b = neoclr_gc_allocate_v1(&c[1].text, bytes, NEOCLR_GC_BYTES, &roots[1][slot]);
        CHECK(a == b && c[0].text.used == c[1].text.used);
        if (a) {
            CHECK(!neoclr_gc_collect_v1(&c[0], &frames[0]));
            CHECK(!neoclr_gc_collect_v1(&c[1], &frames[1]));
            CHECK(!neoclr_gc_allocate_v1(&c[0].text, bytes, NEOCLR_GC_BYTES, &roots[0][slot]));
            CHECK(!neoclr_gc_allocate_v1(&c[1].text, bytes, NEOCLR_GC_BYTES, &roots[1][slot]));
        }
        CHECK((unsigned char *)roots[0][slot] - c[0].text.data ==
              (unsigned char *)roots[1][slot] - c[1].text.data);
        sizes[slot] = bytes;
        for (unsigned side = 0; side < 2; side++) {
            unsigned char *data = roots[side][slot];
            for (unsigned i = 0; i < bytes; i++) CHECK(data[i] == 0);
            memset(data, slot + 1, bytes);
            for (unsigned other = 0; other < 32; other++) if (roots[side][other]) {
                unsigned char *live = roots[side][other];
                CHECK(live[0] == other + 1 && live[sizes[other] - 1] == other + 1);
            }
        }
    }
    CHECK(!neoclr_gc_collect_v1(&c[0], NULL) && !c[0].text.used);
    CHECK(!neoclr_gc_collect_v1(&c[1], NULL) && !c[1].text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c[1].text));
    CHECK(!neoclr_gc_heap_acquire_v1(&c[1].text));
    CHECK(!neoclr_gc_heap_release_v1(&c[1].text));
    return 0;
}

static int invalid_links(void) {
    uint64_t heap[256] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *objects[6];
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    for (unsigned i = 0; i < 6; i++)
        CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &objects[i]));
    neoclr_probe_storage slots[] = {{&objects[1], 8, 0}, {&objects[3], 8, 0}, {&objects[5], 8, 0}};
    neoclr_probe_frame frame = {.context = &c, .storage = slots, .storage_count = 3};
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    /* Private descriptor: span, bytes/next-free, kind, state. */
    uint64_t correct = heap[1], used = c.text.used;
    uint64_t invalid[] = {1, 2, c.text.used + 1, UINT64_MAX};
    for (unsigned i = 0; i < sizeof(invalid) / sizeof(invalid[0]); i++) {
        heap[1] = invalid[i];
        void *out = (void *)(uintptr_t)7;
        CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &out) == 3);
        CHECK(out == (void *)(uintptr_t)7 && c.text.used == used);
        CHECK(neoclr_gc_collect_v1(&c, &frame) == 3 && c.text.used == used);
        heap[1] = correct;
    }
    /* Missing and allocated-target links fail the full reconciliation. */
    heap[1] = 0;
    CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
    heap[1] = 41;
    CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
    heap[1] = correct;
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    heap[1] = 0;
    CHECK(neoclr_gc_heap_release_v1(&c.text) == 3);
    /* Release relinquishes ownership even on failure. External collection ignores
     * index metadata and can reclaim the abandoned arena before reacquisition. */
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    return 0;
}
#ifdef NEOCLR_TEST_THREADS
static void *threaded(void *unused) {
    (void)unused;
    return (void *)(uintptr_t)differential();
}
#endif
int main(void) {
    CHECK(!differential());
    CHECK(!invalid_links());
#ifdef NEOCLR_TEST_THREADS
    pthread_t threads[2];
    for (unsigned i = 0; i < 2; i++) CHECK(!pthread_create(&threads[i], NULL, threaded, NULL));
    for (unsigned i = 0; i < 2; i++) {
        void *result = NULL;
        CHECK(!pthread_join(threads[i], &result) && !result);
    }
#endif
    puts("PASS indexed first-fit differential, corruption, reclamation and reset");
    return 0;
}
