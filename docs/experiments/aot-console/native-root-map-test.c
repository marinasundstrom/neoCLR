#include "native-gc.h"
#include <string.h>
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    uint64_t heap[128] = {0}, lanes[1] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_probe_storage unused = {lanes, 8, 0};
    neoclr_probe_frame frame;
    neoclr_gc_enter_owned_v1(&frame, &c, 0, &unused, 0, "{}", 2);
    neoclr_gc_root_map map = {NEOCLR_GC_ROOT_MAP_MAGIC, 1};
    void *first, *second;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_BYTES, &first));
    lanes[0] = (uintptr_t)first;
    neoclr_gc_stack_map_v1(&frame, 0, lanes, 1, &map, sizeof(map));
    CHECK(frame.reserved & NEOCLR_GC_TYPED_PLAN);
    if (!strcmp(argv[1], "magic")) map.magic ^= 1;
    else if (!strcmp(argv[1], "count")) map.lane_count = 2;
    else if (!strcmp(argv[1], "length")) {
        neoclr_gc_stack_map_v1(&frame, 0, lanes, 1, &map, 7);
        return 0;
    } else if (!strcmp(argv[1], "null")) {
        neoclr_gc_stack_map_v1(&frame, 0, lanes, 1, NULL, 8);
        return 0;
    } else if (!strcmp(argv[1], "head")) {
        neoclr_probe_frame stale = frame;
        neoclr_gc_stack_map_v1(&stale, 0, lanes, 1, &map, 8);
        return 0;
    } else if (strcmp(argv[1], "valid")) return 2;
    /* Mutated headers are checked again, rather than cached by address. */
    neoclr_gc_stack_map_v1(&frame, 0, lanes, 1, &map, sizeof(map));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_BYTES, &second));
    lanes[0] = (uintptr_t)second;
    unsigned char unaligned[9];
    memcpy(unaligned + 1, &map, sizeof(map));
    neoclr_gc_stack_map_v1(&frame, 1, lanes, 1, unaligned + 1, sizeof(map));
    uint64_t bytes; uint32_t kind;
    CHECK(neoclr_gc_allocation_info_v1(&c, first, &bytes, &kind) == 3);
    CHECK(!neoclr_gc_allocation_info_v1(&c, second, &bytes, &kind));
    CHECK(frame.plan == (const char *)unaligned + 1 && frame.length == 8);
    lanes[0] = 0;
    const char *json = "{\"requiredSpillLanes\":[]}";
    neoclr_probe_stack_roots_v2(&frame, 2, lanes, 0, json, (uint32_t)strlen(json));
    CHECK(!(frame.reserved & NEOCLR_GC_TYPED_PLAN) && !c.text.used);
    neoclr_probe_leave_v1(&frame);
    return 0;
}
