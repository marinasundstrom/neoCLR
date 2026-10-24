#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
int main(void) {
    uint64_t heap[256] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *root = NULL;
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    unsigned cases = 0;
    for (int owned = 0; owned < 2; owned++) {
        if (owned) CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        for (unsigned bytes = 1; bytes <= 144; bytes++) {
            for (unsigned lane = 0; lane < ((bytes + 7) / 8); lane++) {
                for (int offset = -34; offset <= 17; offset++) {
                    unsigned char *child;
                    CHECK(!neoclr_gc_allocate_v1(&c.text, bytes, NEOCLR_GC_OBJECT, &root));
                    CHECK(!neoclr_gc_allocate_v1(&c.text, 9, NEOCLR_GC_BYTES, (void **)&child));
                    uint64_t word = offset == -34 ? UINT64_MAX : offset == -33 ? 0 : (uintptr_t)child + offset;
                    /* Fill the final padded word too: partial payload words must not trace. */
                    memcpy((unsigned char *)root + lane * 8, &word, 8);
                    CHECK(!neoclr_gc_collect_v1(&c, &frame));
                    uint64_t size; uint32_t kind;
                    int alive = lane > 0 && (lane + 1) * 8 <= bytes && offset >= 0 && offset < 9;
                    CHECK((neoclr_gc_allocation_info_v1(&c, child, &size, &kind) == 0) == alive);
                    CHECK(!neoclr_gc_allocation_info_v1(&c, root, &size, &kind) && size == bytes);
                    root = NULL;
                    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
                    cases++;
                }
            }
        }
        if (owned) CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    neoclr_gc_lookup_profile p = neoclr_gc_lookup_profile_v1();
    printf("%u %llu %llu %llu %llu %llu %llu\n", cases,
        (unsigned long long)p.mark_candidates, (unsigned long long)p.index_queries,
        (unsigned long long)p.index_outside, (unsigned long long)p.index_probes,
        (unsigned long long)p.newly_marked, (unsigned long long)p.object_words);
    return 0;
}
