#include "native-gc.h"
#include <string.h>
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
int main(void) {
    uint64_t heap[64] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *hole = NULL, *live = NULL, *output = (void *)(uintptr_t)1;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 64, NEOCLR_GC_BYTES, &hole));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_BYTES, &live));
    neoclr_probe_storage slot = {&live, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    uint64_t *header = (uint64_t *)live - 4;
    uint64_t span = header[0], used = c.text.used;
    header[0] = 7;
    uint64_t snapshot[64]; memcpy(snapshot, heap, sizeof(heap));
    /* A fitting first hole cannot conceal a corrupt later header. */
    CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &output) == 3);
    CHECK(neoclr_gc_allocate_v1(&c.text, UINT64_MAX, NEOCLR_GC_BYTES, &output) == 3);
    CHECK(output == (void *)(uintptr_t)1 && c.text.used == used && !memcmp(snapshot, heap, sizeof(heap)));
    header[0] = span;
    CHECK(neoclr_gc_allocate_v1(&c.text, UINT64_MAX, NEOCLR_GC_BYTES, &output) == 5);
    CHECK(output == (void *)(uintptr_t)1);
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &output) && output == hole);
    CHECK(c.text.used == used);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    return 0;
}
