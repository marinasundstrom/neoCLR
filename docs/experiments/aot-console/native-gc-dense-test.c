/* Dense index boundaries, interior roots, holes and complete reclamation. */
#include "native-gc.h"
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
static void cleaned(void *state) { ++*(int *)state; }
int main(void) {
    enum { COUNT = 4097, SPAN = 48 };
    unsigned char *buffer = malloc(COUNT * SPAN * 2);
    if (!buffer) return 1;
    neoclr_aot_context c = {.text = {buffer, COUNT * SPAN * 2, 0}};
    uint64_t root = 0;
    void *conditional_value = NULL;
    for (int i = 0; i < COUNT; i++) {
        void *live = NULL, *dead = NULL;
        if (neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &live) ||
            neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_TEXT, &dead)) return 2;
        ((uint64_t *)live)[1] = root;
        /* Every link is an interior address, including across bucket boundaries. */
        root = (uintptr_t)live + 8;
        conditional_value = dead;
    }
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    neoclr_gc_conditional_root conditional = {(void *)(uintptr_t)(root - 8), conditional_value, 0};
    neoclr_gc_resource resource = {0};
    int cleanup_count = 0;
    if (neoclr_gc_resource_attach_v1(&c, conditional.owner, &resource, &cleanup_count, cleaned)) return 7;
    /* Corruption after a valid dense prefix must fail before marks, resource
     * cleanup or conditional outputs change. Repair and collect the same heap. */
    unsigned char *snapshot = malloc((size_t)c.text.used);
    if (!snapshot) return 9;
    uint64_t *last_header = (uint64_t *)conditional_value - 4;
    uint64_t saved_span = last_header[0];
    last_header[0] = c.text.used + 8;
    memcpy(snapshot, buffer, (size_t)c.text.used);
    neoclr_gc_statistics before = neoclr_gc_statistics_v1();
    if (neoclr_gc_collect_conditional_v1(&c, &frame, &conditional, 1) != 3 ||
        memcmp(snapshot, buffer, (size_t)c.text.used) || cleanup_count ||
        conditional.owner != (void *)(uintptr_t)(root - 8) ||
        conditional.value != conditional_value || conditional.retained ||
        neoclr_gc_statistics_v1().reclaimed_allocations != before.reclaimed_allocations) return 10;
    last_header[0] = saved_span;
    free(snapshot);
    /* A dense index must still reject an interior owner without mutating entries. */
    void *owner = conditional.owner;
    conditional.owner = (unsigned char *)owner + 1;
    if (neoclr_gc_collect_conditional_v1(&c, &frame, &conditional, 1) != 3 ||
        conditional.owner != (unsigned char *)owner + 1 || conditional.value != conditional_value || cleanup_count) return 8;
    conditional.owner = owner;
    for (int round = 0; round < 3; round++) {
        if (neoclr_gc_collect_conditional_v1(&c, &frame, &conditional, 1) || !conditional.retained || cleanup_count) return 3;
        uint64_t cursor = root;
        for (int i = 0; i < COUNT; i++) {
            if (!cursor) return 4;
            cursor = *(uint64_t *)(uintptr_t)cursor;
        }
        if (cursor || c.text.used != COUNT * 2 * SPAN ||
            neoclr_gc_statistics_v1().reclaimed_allocations != COUNT - 1) return 5;
    }
    root = 0;
    if (neoclr_gc_collect_conditional_v1(&c, &frame, &conditional, 1) || c.text.used ||
        conditional.owner || conditional.value || conditional.retained || cleanup_count != 1 || resource.context ||
        neoclr_gc_statistics_v1().reclaimed_allocations != COUNT * 2) return 6;
    free(buffer);
    return 0;
}
