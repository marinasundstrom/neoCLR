#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static int check(unsigned population, int stale_marks, unsigned intern) {
    uint64_t *heap = calloc(population + 80, 64);
    CHECK(heap);
    neoclr_aot_context c = {.text = {(unsigned char *)heap, (population + 80) * UINT64_C(64), 0}};
    void *root = NULL, *child = NULL, *dead = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &root));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &child));
    ((uint64_t *)root)[1] = (uintptr_t)child;
    *(uint64_t *)child = 42;
    for (unsigned i = 2; i < population; i++)
        CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &dead));
    const neoclr_aot_text *retained[33] = {0};
    for (unsigned i = 0; i < intern; i++) {
        void *text = NULL;
        CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_TEXT, &text));
        ((neoclr_aot_text *)text)->length = 1;
        ((neoclr_aot_text *)text)->bytes[0] = (unsigned char)i;
        CHECK(!neoclr_string_intern_v1(text, &c.text, &retained[i]) && retained[i] == text);
    }
    if (stale_marks) {
        /* Private descriptor mutation: valid marked/scanned states must be reset
         * before tracing; stale marks cannot retain garbage or hide child edges. */
        ((uint64_t *)root)[-1] = 3;
        ((uint64_t *)child)[-1] = 7;
        ((uint64_t *)dead)[-1] = 3;
    }
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    uint64_t before = neoclr_gc_statistics_v1().reclaimed_allocations;
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations - before == population - 2);
    CHECK(*(uint64_t *)child == 42);
    uint64_t bytes = 0; uint32_t kind = 0;
    CHECK(!neoclr_gc_allocation_info_v1(&c, child, &bytes, &kind) && bytes == 8);
    for (unsigned i = 0; i < intern; i++)
        CHECK(retained[i]->length == 1 && retained[i]->bytes[0] == i);
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    if (intern) {
        for (unsigned i = 0; i < intern; i++)
            CHECK(retained[i]->length == 1 && retained[i]->bytes[0] == i);
        CHECK(!neoclr_gc_entry_check_v1(&c));
        c.text.used = 0; /* Explicit entry reset releases the intern pool. */
    }
    CHECK(!c.text.used);
    free(heap);
    return 0;
}
int main(void) {
    const unsigned interns[] = {0, 1, 32, 33};
    for (int dense = 0; dense < 2; dense++)
        for (int stale = 0; stale < 2; stale++)
            for (unsigned i = 0; i < sizeof(interns) / sizeof(interns[0]); i++)
                CHECK(!check(dense ? 2050 : 64, stale, interns[i]));
    puts("PASS sparse/dense clear/stale marks, intern seeding, child tracing and reclamation");
    return 0;
}
