#include "native-gc.h"
#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
extern void experiment_gc_shadow(uint64_t *);
extern void experiment_gc_shadow_gate(int);
#define CHECK(x) do { if (!(x)) abort(); } while (0)
static void *run(void *unused) {
    (void)unused;
    for (unsigned trial = 0; trial < 100; trial++) {
        uint64_t heap[256] = {0};
        neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *a, *b, *dead;
        CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, (void **)&a));
        CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, (void **)&b));
        CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, (void **)&dead));
        a[1] = (uintptr_t)b;
        b[1] = (uintptr_t)a;
        void *root = a;
        neoclr_probe_storage slot = {&root, 8, 0};
        neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
        /* The shadow worker cannot read its snapshot until all original payloads
         * and roots have changed, and synchronous GC has reclaimed the dead node. */
        experiment_gc_shadow_gate(1);
        CHECK(!neoclr_gc_collect_v1(&c, &frame));
        CHECK(*dead == UINT64_C(0xdddddddddddddddd));
        a[1] = b[1] = 0;
        root = NULL;
        CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, (void **)&dead));
        dead[1] = (uintptr_t)a;
        experiment_gc_shadow_gate(0);
        uint64_t stats[5];
        experiment_gc_shadow(stats);
        CHECK(stats[0] == 2 * trial + 1 && !stats[4]);
        CHECK(!neoclr_gc_collect_v1(&c, &frame));
        CHECK(!c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    uint64_t stats[5]; experiment_gc_shadow(stats);
    CHECK(stats[0] == 200 && !stats[4]);
    return NULL;
}
int main(void) {
    /* Empty external arenas may have a null data pointer. */
    neoclr_aot_context empty = {0};
    CHECK(!neoclr_gc_collect_v1(&empty, NULL));
    uint64_t empty_stats[5]; experiment_gc_shadow(empty_stats);
    CHECK(empty_stats[0] == 1 && !empty_stats[4]);
    /* Use fresh owner threads so their lifetime counters start at zero. */
    /* Fresh owner TLS after thread exit, with an independent background worker. */
    for (unsigned i = 0; i < 3; i++) {
        pthread_t owner;
        CHECK(!pthread_create(&owner, NULL, run, NULL));
        CHECK(!pthread_join(owner, NULL));
    }
    puts("600 delayed-snapshot and reclamation comparisons passed");
    return 0;
}
