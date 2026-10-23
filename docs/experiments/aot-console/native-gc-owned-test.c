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
int main(void) {
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
