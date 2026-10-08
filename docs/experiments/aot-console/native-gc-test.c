#include "native-gc.h"
#include <string.h>
#define CHECK(test) do { if (!(test)) return __LINE__; } while (0)
static void *allocate(neoclr_aot_context *c, uint64_t bytes, uint32_t kind) {
    void *p = NULL;
    return neoclr_gc_allocate_v1(&c->text, bytes, kind, &p) ? NULL : p;
}
int main(void) {
    uint64_t buffer[129];
    buffer[128] = UINT64_C(0x1122334455667788);
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}};
    uint64_t *a = allocate(&c, 24, NEOCLR_GC_OBJECT);
    uint64_t *b = allocate(&c, 24, NEOCLR_GC_OBJECT);
    CHECK(a && b);
    a[1] = (uintptr_t)b; b[1] = (uintptr_t)a; a[2] = 42;
    uint64_t interior = (uintptr_t)&a[2];
    neoclr_probe_storage slot = {&interior, 8, 2};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    uint64_t used = c.text.used;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == used && a[2] == 42 && b[1] == (uintptr_t)a);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 0); /* unreachable cycle */
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == 2);
    uint64_t *text = allocate(&c, 16, NEOCLR_GC_TEXT);
    uint64_t *dead = allocate(&c, 16, NEOCLR_GC_TEXT);
    uint64_t *strings = allocate(&c, 34, NEOCLR_GC_STRINGS);
    CHECK(text && dead && strings);
    text[0] = 1; ((char *)(text + 1))[0] = 'x';
    strings[0] = UINT64_C(0x80000004); strings[1] = 2;
    strings[2] = (uintptr_t)text; strings[3] = (uintptr_t)dead;
    ((unsigned char *)strings)[32] = 1; /* Only initialized element zero is traced. */
    interior = (uintptr_t)strings;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && text[0] == 1 && neoclr_gc_statistics_v1().reclaimed_allocations == 3);
    void *reuse = allocate(&c, 16, NEOCLR_GC_BYTES);
    CHECK(reuse == dead); /* free-list reuse without moving retained data */
    interior = 0;
    c.fault.code = 4; c.fault.message = (const void *)text;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && text[0] == 1);
    c.fault.code = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 0);
    /* Atomic bytes that resemble an address do not trace their payload. */
    uint64_t *atomic = allocate(&c, 16, NEOCLR_GC_BYTES);
    dead = allocate(&c, 16, NEOCLR_GC_TEXT);
    CHECK(atomic && dead); atomic[1] = (uintptr_t)dead; interior = (uintptr_t)atomic;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 48);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    /* Live-set exhaustion must neither collect inside allocation nor publish output. */
    void *large = allocate(&c, 992, NEOCLR_GC_BYTES);
    CHECK(large && c.text.used == 1024);
    void *output = (void *)(uintptr_t)1;
    CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_TEXT, &output) == 5 && output == (void *)(uintptr_t)1);
    interior = (uintptr_t)large;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 1024);
    CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_TEXT, &output) == 5);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    CHECK(buffer[128] == UINT64_C(0x1122334455667788));
    return 0;
}
