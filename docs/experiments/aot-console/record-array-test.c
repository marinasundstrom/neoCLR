#include "text-arena.h"
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
#include <string.h>
#define CHECK(test) do { if (!(test)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[129];
    buffer[128] = UINT64_C(0x1122334455667788);
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}};
    void *result = (void *)(uintptr_t)1;
    CHECK(neoclr_reserve_records_v1(&c.text, -1, 1, &result) == 3);
    CHECK(neoclr_reserve_records_v1(&c.text, 65537, 1, &result) == 7);
    CHECK(neoclr_reserve_records_v1(&c.text, 1, 0, &result) == 3);
    CHECK(neoclr_reserve_records_v1(&c.text, 1, 65, &result) == 3);
    CHECK(neoclr_reserve_records_v1(NULL, 1, 1, &result) == 3);
    CHECK(neoclr_reserve_records_v1(&c.text, 1, 1, NULL) == 3);
    CHECK(result == (void *)(uintptr_t)1 && !c.text.used);
    c.text.capacity = 40;
    CHECK(neoclr_reserve_records_v1(&c.text, 2, 3, &result) == 5);
    CHECK(result == (void *)(uintptr_t)1 && !c.text.used);
    c.text.capacity = 1024;
    CHECK(!neoclr_reserve_records_v1(&c.text, 2, 3, &result));
    uint64_t *records = result;
    CHECK(records[0] == UINT64_C(0x80000005) && records[1] == 2 && records[2] == 3);
    for (unsigned i = 24; i < 74; i++) CHECK(!((unsigned char *)records)[i]);
#ifdef NEOCLR_NATIVE_GC
    void *live, *dead;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &live));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &dead));
    records[3] = 42; records[4] = (uintptr_t)live; records[7] = (uintptr_t)dead;
    ((unsigned char *)records)[72] = 1; /* Element one remains uninitialized. */
    uint64_t root = (uintptr_t)records;
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == 1);
    CHECK(records[3] == 42 && records[4] == (uintptr_t)live);
    for (unsigned test = 0; test < 5; test++) {
        uint64_t original[] = {records[0], records[1], records[2]};
        if (test == 0) records[0] = 0;
        if (test == 1) records[1] = 65537;
        if (test == 2) records[1] = 3; /* Extent exceeds this allocation. */
        if (test == 3) records[2] = 0;
        if (test == 4) records[2] = 65;
        uint64_t collections = neoclr_gc_statistics_v1().collections;
        CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
        CHECK(neoclr_gc_statistics_v1().collections == collections);
        memcpy(records, original, sizeof(original));
        CHECK(!neoclr_gc_collect_v1(&c, &frame));
    }
    records[4] = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == 2);
    root = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
#endif
    uint64_t wide_buffer[257] = {0};
    wide_buffer[256] = UINT64_C(0x8877665544332211);
    neoclr_aot_context wide_context = {.text = {(unsigned char *)wide_buffer, 2048, 0}};
    void *wide_result = NULL;
    CHECK(!neoclr_reserve_records_v1(&wide_context.text, 2, 64, &wide_result));
    uint64_t *wide = wide_result;
    CHECK(wide[1] == 2 && wide[2] == 64);
    for (unsigned i = 24; i < 1050; i++) CHECK(!((unsigned char *)wide)[i]);
#ifdef NEOCLR_NATIVE_GC
    uint64_t reclaimed_before = neoclr_gc_statistics_v1().reclaimed_allocations;
    void *wide_live, *wide_dead;
    CHECK(!neoclr_gc_allocate_v1(&wide_context.text, 16, NEOCLR_GC_OBJECT, &wide_live));
    CHECK(!neoclr_gc_allocate_v1(&wide_context.text, 16, NEOCLR_GC_OBJECT, &wide_dead));
    wide[66] = (uintptr_t)wide_dead; /* Uninitialized first element. */
    wide[130] = (uintptr_t)wide_live; /* Last lane of second element. */
    ((unsigned char *)wide)[1049] = 1;
    uint64_t wide_root = (uintptr_t)wide;
    neoclr_probe_storage wide_slot = {&wide_root, 8, 0};
    neoclr_probe_frame wide_frame = {.context = &wide_context, .storage = &wide_slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&wide_context, &wide_frame));
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == reclaimed_before + 1);
    wide[130] = 0;
    CHECK(!neoclr_gc_collect_v1(&wide_context, &wide_frame));
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == reclaimed_before + 2);
#endif
    CHECK(wide_buffer[256] == UINT64_C(0x8877665544332211));
    return buffer[128] == UINT64_C(0x1122334455667788) ? 0 : 1;
}
