#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static int32_t allocate(int mode, int rooted, neoclr_aot_text_arena *arena, int32_t length, void **out) {
    if (mode < 2) return rooted ? neoclr_allocate_strings_rooted_v1(arena, length, mode, out) : neoclr_allocate_strings_v1(arena, length, mode, out);
    if (mode == 2) return rooted ? neoclr_allocate_references_rooted_v1(arena, length, out) : neoclr_allocate_references_v1(arena, length, out);
    return rooted ? neoclr_reserve_records_rooted_v1(arena, length, 1, out) : neoclr_reserve_records_v1(arena, length, 1, out);
}
static int wide_records(void) {
    uint64_t heap[2048] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *array = NULL, *leaf = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_BYTES, &leaf));
    CHECK(!neoclr_reserve_records_v1(&c.text, 1, NEOCLR_NATIVE_LAYOUT_LANES, &array));
    unsigned char *data = array;
    memcpy(data + 24 + (NEOCLR_NATIVE_LAYOUT_LANES - 1) * 8, &leaf, sizeof(leaf));
    data[24 + NEOCLR_NATIVE_LAYOUT_LANES * 8] = 1;
    neoclr_probe_storage slot = {&array, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    for (int i = 0; i < 3; i++) {
        CHECK(!neoclr_gc_collect_v1(&c, &frame));
        uint64_t bytes = 0; uint32_t kind = 0;
        CHECK(!neoclr_gc_allocation_info_v1(&c, leaf, &bytes, &kind));
        CHECK(bytes == 16 && kind == NEOCLR_GC_BYTES);
    }
    void *rejected = (void *)(uintptr_t)123;
    CHECK(neoclr_reserve_records_v1(&c.text, 1, NEOCLR_NATIVE_LAYOUT_LANES + 1, &rejected) == 3);
    CHECK(rejected == (void *)(uintptr_t)123);
    uint64_t invalid = NEOCLR_NATIVE_LAYOUT_LANES + 1, valid = NEOCLR_NATIVE_LAYOUT_LANES;
    memcpy(data + 16, &invalid, 8);
    CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
    memcpy(data + 16, &valid, 8);
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    /* Uninitialized elements must not retain their stale reference lanes. */
    data[24 + NEOCLR_NATIVE_LAYOUT_LANES * 8] = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    uint64_t bytes = 0; uint32_t kind = 0;
    CHECK(neoclr_gc_allocation_info_v1(&c, leaf, &bytes, &kind) != 0);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    return 0;
}
int main(void) {
    CHECK(!wide_records());
    for (int mode = 0; mode < 4; mode++) {
        uint64_t heap[16] = {0}, foreign_heap[16] = {0};
        neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
        neoclr_aot_context foreign = {.text = {(unsigned char *)foreign_heap, sizeof(foreign_heap), 0}};
        void *live = NULL, *dead = NULL, *out = (void *)(uintptr_t)123;
        int32_t length = 2;
        CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_BYTES, &live));
        CHECK(!neoclr_gc_allocate_v1(&c.text, 48, NEOCLR_GC_BYTES, &dead));
        memset(live, 0xa5, 16);
        uint64_t before = neoclr_gc_statistics_v1().collections;
        CHECK(allocate(mode, 1, &c.text, length, &out) == 3 && out == (void *)(uintptr_t)123);
        neoclr_probe_storage slots[] = {{&live, 8, 0}, {&dead, 8, 0}};
        neoclr_probe_frame frame;
        neoclr_probe_enter_v3(&frame, &c, 0, slots, 2, "{}", 2);
        CHECK(allocate(mode, 1, &foreign.text, length, &out) == 3 && !foreign.text.used);
        CHECK(allocate(mode, 1, &c.text, -1, &out) == 3);
        CHECK(allocate(mode, 1, &c.text, 65537, &out) == 7);
        CHECK(neoclr_allocate_scalars_rooted_v1(&c.text, length, 2, &out) == 3);
        CHECK(allocate(mode, 0, &c.text, length, &out) == 5);
        CHECK(out == (void *)(uintptr_t)123 && neoclr_gc_statistics_v1().collections == before);
        CHECK(allocate(mode, 1, &c.text, length, &out) == 5 && out == (void *)(uintptr_t)123);
        CHECK(neoclr_gc_statistics_v1().collections == before + 1);
        dead = NULL;
        CHECK(!allocate(mode, 1, &c.text, length, &out));
        CHECK(neoclr_gc_statistics_v1().collections == before + 2);
        uint64_t *header = out;
        CHECK(header[0] == (mode == 0 ? UINT64_C(0x80000003) : mode == 1 ? UINT64_C(0x80000004) : UINT64_C(0x80000005)));
        CHECK(header[1] == (uint64_t)length);
        if (mode >= 2) CHECK(header[2] == 1);
        unsigned char *data = (unsigned char *)out + (mode < 2 ? 16 : 24);
        int payload = length * 8;
        for (int i = 0; i < payload; i++) CHECK(!data[i]);
        if (mode == 1 || mode >= 2)
            for (int i = 0; i < length; i++) CHECK(data[payload + i] == (mode == 2 ? 1 : 0));
        for (int i = 0; i < 16; i++) CHECK(((unsigned char *)live)[i] == 0xa5);
        /* Initialized reference slots retain the referent after its caller root disappears. */
        void *old_live = live;
        memcpy(data, &old_live, sizeof(old_live));
        if (mode == 1 || mode == 3) data[payload] = 1;
        slots[0] = (neoclr_probe_storage){&out, 8, 0};
        CHECK(!neoclr_gc_collect_v1(&c, &frame));
        uint64_t bytes = 123; uint32_t kind = 456;
        CHECK(!neoclr_gc_allocation_info_v1(&c, old_live, &bytes, &kind));
        CHECK(bytes == 16 && kind == NEOCLR_GC_BYTES);
        CHECK(!neoclr_gc_allocation_info_v1(&c, out, &bytes, &kind) && kind == (uint32_t)(mode < 2 ? NEOCLR_GC_STRINGS : NEOCLR_GC_RECORDS));
        CHECK(neoclr_root_probe_head_v1() == &frame);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        /* Empty arrays still have a valid initialized header and publish normally. */
        neoclr_probe_enter_v3(&frame, &c, 0, slots, 0, "{}", 2);
        CHECK(!allocate(mode, 1, &c.text, 0, &out) && ((uint64_t *)out)[1] == 0);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        CHECK(!neoclr_gc_entry_check_v1(&c));
    }
    puts("PASS rooted reference arrays, initialization, referent retention, rejection and reclamation");
    return 0;
}
