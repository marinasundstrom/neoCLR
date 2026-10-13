#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static int32_t allocate(int mode, int rooted, neoclr_aot_text_arena *arena, int32_t length, void **out) {
    if (mode == 0) return rooted ? neoclr_allocate_bytes_rooted_v1(arena, length, out) : neoclr_allocate_bytes_v1(arena, length, out);
    if (mode == 1) return rooted ? neoclr_reserve_bytes_rooted_v1(arena, length, out) : neoclr_reserve_bytes_v1(arena, length, out);
    return rooted ? neoclr_allocate_scalars_rooted_v1(arena, length, mode - 2, out) : neoclr_allocate_scalars_v1(arena, length, mode - 2, out);
}
int main(void) {
    for (int mode = 0; mode < 4; mode++) {
        uint64_t heap[16] = {0}, foreign_heap[16] = {0};
        neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
        neoclr_aot_context foreign = {.text = {(unsigned char *)foreign_heap, sizeof(foreign_heap), 0}};
        void *live = NULL, *dead = NULL, *out = (void *)(uintptr_t)123;
        int32_t length = mode < 2 ? 8 : 2;
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
        CHECK(header[0] == (mode == 0 ? UINT64_C(0x80000001) : mode == 1 ? UINT64_C(0x80000002) : UINT64_C(0x80000006)));
        CHECK(header[1] == (uint64_t)length);
        if (mode >= 2) CHECK(header[2] == 1);
        unsigned char *data = (unsigned char *)out + (mode < 2 ? 16 : 24);
        int payload = mode < 2 ? length : length * 8;
        for (int i = 0; i < payload; i++) CHECK(!data[i]);
        if (mode == 1 || mode >= 2)
            for (int i = 0; i < length; i++) CHECK(data[payload + i] == (mode == 2 ? 1 : 0));
        for (int i = 0; i < 16; i++) CHECK(((unsigned char *)live)[i] == 0xa5);
        /* Pointer-shaped scalar bits must not retain an otherwise unreachable allocation. */
        void *old_live = live;
        memcpy(data, &old_live, sizeof(old_live));
        slots[0] = (neoclr_probe_storage){&out, 8, 0};
        CHECK(!neoclr_gc_collect_v1(&c, &frame));
        uint64_t bytes = 123; uint32_t kind = 456;
        CHECK(neoclr_gc_allocation_info_v1(&c, old_live, &bytes, &kind) == 3);
        CHECK(bytes == 123 && kind == 456);
        CHECK(!neoclr_gc_allocation_info_v1(&c, out, &bytes, &kind) && kind == NEOCLR_GC_BYTES);
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
    puts("PASS rooted primitive arrays, initialization, atomic payloads, rejection and reclamation");
    return 0;
}
