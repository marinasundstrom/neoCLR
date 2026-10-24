#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static int exercise(int owned, uint64_t *offsets) {
    uint64_t heap[256] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    CHECK(!owned || !neoclr_gc_heap_acquire_v1(&c.text));
    void *items[6];
    for (int i = 0; i < 6; i++)
        CHECK(!neoclr_gc_allocate_v1(&c.text, 8 + 16 * i, NEOCLR_GC_BYTES, &items[i]));
    void *roots[] = {items[1], items[3], items[5]};
    neoclr_probe_storage slots[3];
    for (int i = 0; i < 3; i++) slots[i] = (neoclr_probe_storage){&roots[i], 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = slots, .storage_count = 3};
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    /* Skip a small first hole, split a later hole, then reuse the first hole. */
    unsigned sizes[] = {40, 8, 8, 8, 128};
    for (unsigned i = 0; i < 5; i++) {
        void *out = NULL;
        CHECK(!neoclr_gc_allocate_v1(&c.text, sizes[i], NEOCLR_GC_BYTES, &out));
        offsets[i] = (unsigned char *)out - c.text.data;
    }
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    void *out = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &out));
    offsets[5] = (unsigned char *)out - c.text.data;
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    CHECK(!owned || !neoclr_gc_heap_release_v1(&c.text));
    return 0;
}
static int base_cache(void) {
    uint64_t heap[4096] = {0}, foreign[8] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *items[129];
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    for (unsigned i = 0; i < 129; i++)
        CHECK(!neoclr_gc_allocate_v1(&c.text, 96, NEOCLR_GC_BYTES, &items[i]));
    /* More entries than cache slots force eviction. Every lookup must still
     * prove the exact base; an object-shaped header inside payload is not one. */
    uint64_t fake[] = {40, 8, NEOCLR_GC_BYTES, 1, 42};
    memcpy(items[0], fake, sizeof(fake));
    uint64_t bytes = 0; uint32_t kind = 0;
    for (unsigned i = 0; i < 129; i++) {
        CHECK(!neoclr_gc_allocation_info_v1(&c, items[i], &bytes, &kind));
        CHECK(bytes == 96 && kind == NEOCLR_GC_BYTES);
    }
    void *invalid[] = {(unsigned char *)items[0] + 32,
        (unsigned char *)items[0] + 1, (unsigned char *)items[0] - 8,
        (unsigned char *)items[0] + 96, foreign, c.text.data + c.text.used};
    for (unsigned i = 0; i < sizeof(invalid) / sizeof(invalid[0]); i++) {
        bytes = 123; kind = 456;
        CHECK(neoclr_gc_allocation_info_v1(&c, invalid[i], &bytes, &kind) == 3);
        CHECK(bytes == 123 && kind == 456);
        uint64_t handle = 789;
        CHECK(neoclr_gc_host_root_create_v1(&c, invalid[i], &handle) == 3 && handle == 789);
    }
    uint64_t handle = 0;
    CHECK(!neoclr_gc_host_root_create_owned_v1(&c, items[0], items[128], &handle));
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    neoclr_probe_storage slots[] = {{&items[0], 8, 0}, {&items[128], 8, 0}};
    neoclr_probe_frame frame = {.context = &c, .storage = slots, .storage_count = 2};
    CHECK(!neoclr_gc_allocation_info_v1(&c, items[1], &bytes, &kind));
    CHECK(!neoclr_gc_allocation_info_v1(&c, items[128], &bytes, &kind));
    neoclr_gc_lookup_profile_reset_v1();
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    /* Reclaiming many colliding bases must preserve the survivor's proof. */
    CHECK(!neoclr_gc_allocation_info_v1(&c, items[128], &bytes, &kind));
#ifdef NEOCLR_GC_PROFILE
    neoclr_gc_lookup_profile lookup = neoclr_gc_lookup_profile_v1();
    CHECK(lookup.base_calls == 1 && lookup.base_cache_hits == 1 && !lookup.base_walks);
#endif
    CHECK(neoclr_gc_allocation_info_v1(&c, items[1], &bytes, &kind) == 3);
    CHECK(!neoclr_gc_allocation_info_v1(&c, items[0], &bytes, &kind));
    void *reused = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 9, NEOCLR_GC_BYTES, &reused));
    CHECK(reused == items[1]);
    CHECK(!neoclr_gc_allocation_info_v1(&c, reused, &bytes, &kind) && bytes == 9);
    CHECK(neoclr_gc_allocation_info_v1(&c, (unsigned char *)reused + 9, &bytes, &kind) == 3);
    /* Ownership reset must preserve cumulative allocation/collection counters. */
    neoclr_gc_statistics before_release = neoclr_gc_statistics_v1();
    /* Scope release/reacquisition at identical addresses must not retain proofs. */
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    neoclr_gc_statistics after_release = neoclr_gc_statistics_v1();
    CHECK(before_release.allocations == after_release.allocations);
    CHECK(before_release.collections == after_release.collections);
    CHECK(before_release.reclaimed_allocations == after_release.reclaimed_allocations);
    CHECK(before_release.reclaimed_bytes == after_release.reclaimed_bytes);
    c.text.used = 0;
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(neoclr_gc_allocation_info_v1(&c, reused, &bytes, &kind) == 3);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    return 0;
}
int main(void) {
    CHECK(!base_cache());
    uint64_t plain[6], owned[6];
    CHECK(!exercise(0, plain) && !exercise(1, owned));
    CHECK(!memcmp(plain, owned, sizeof(plain)));
    uint64_t heap[32] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_aot_text_arena other = {0};
    CHECK(neoclr_gc_heap_acquire_v1(NULL) == 3);
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(neoclr_gc_heap_acquire_v1(&c.text) == 3);
    CHECK(neoclr_gc_heap_acquire_v1(&other) == 3);
    CHECK(neoclr_gc_heap_release_v1(&other) == 3);
    void *out = (void *)(uintptr_t)1;
    /* Shape changes are detected without mutating heap or output. */
    c.text.capacity -= 8;
    CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &out) == 3);
    CHECK(out == (void *)(uintptr_t)1 && !c.text.used);
    c.text.capacity += 8;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &out));
    uint64_t used = c.text.used;
    c.text.used = 0;
    CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &out) == 3);
    c.text.used = used;
    /* Full collection/release still diagnose descriptor corruption. */
    uint64_t span = heap[0];
    heap[0] = 0;
    CHECK(neoclr_gc_collect_v1(&c, NULL) == 3);
    CHECK(neoclr_gc_heap_release_v1(&c.text) == 3);
    heap[0] = span;
    CHECK(neoclr_gc_heap_release_v1(&c.text) == 3);
    CHECK(neoclr_gc_heap_acquire_v1(&c.text) == 3); /* not empty */
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    puts("PASS owned heap cursor, reclamation, shape checks and release");
    return 0;
}
