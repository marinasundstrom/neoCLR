/* Dense index boundaries, interior roots, holes and complete reclamation. */
#include "native-gc.h"
#include <stdlib.h>
#include <stdint.h>
int main(void) {
    enum { COUNT = 4097, SPAN = 48 };
    unsigned char *buffer = malloc(COUNT * SPAN * 2);
    if (!buffer) return 1;
    neoclr_aot_context c = {.text = {buffer, COUNT * SPAN * 2, 0}};
    uint64_t root = 0;
    for (int i = 0; i < COUNT; i++) {
        void *live = NULL, *dead = NULL;
        if (neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &live) ||
            neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_TEXT, &dead)) return 2;
        ((uint64_t *)live)[1] = root;
        /* Every link is an interior address, including across bucket boundaries. */
        root = (uintptr_t)live + 8;
    }
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    for (int round = 0; round < 3; round++) {
        if (neoclr_gc_collect_v1(&c, &frame)) return 3;
        uint64_t cursor = root;
        for (int i = 0; i < COUNT; i++) {
            if (!cursor) return 4;
            cursor = *(uint64_t *)(uintptr_t)cursor;
        }
        if (cursor || c.text.used != (COUNT * 2 - 1) * SPAN ||
            neoclr_gc_statistics_v1().reclaimed_allocations != COUNT) return 5;
    }
    root = 0;
    if (neoclr_gc_collect_v1(&c, &frame) || c.text.used ||
        neoclr_gc_statistics_v1().reclaimed_allocations != COUNT * 2) return 6;
    free(buffer);
    return 0;
}
