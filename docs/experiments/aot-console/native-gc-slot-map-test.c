#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static uint64_t empty;
static const neoclr_gc_root_map roots = {NEOCLR_GC_ROOT_MAP_MAGIC, 0};
#ifdef NEOCLR_WORKER_GC
static void boundary(neoclr_aot_context *c, const neoclr_probe_frame *f, uint32_t exclusive) {
    if (exclusive != 1 || neoclr_gc_collect_v1(c, f)) abort();
}
#endif
static void publish(neoclr_probe_frame *f) {
#ifdef NEOCLR_WORKER_GC
    neoclr_gc_worker_stack_map_v1(f, 0, &empty, 0, &roots, sizeof(roots), 1);
#else
    neoclr_gc_stack_map_v1(f, 0, &empty, 0, &roots, sizeof(roots));
#endif
}
int main(int argc, char **argv) {
#ifdef _WIN32
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    uint64_t heap[1024] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_probe_frame frame;
    void *value = NULL, *first = NULL, *second = NULL;
    neoclr_probe_storage slot = {&value, 8, 0};
    neoclr_gc_root_map storage = {NEOCLR_GC_STORAGE_MAP_MAGIC, 1};
    neoclr_gc_root_map transient = {NEOCLR_GC_TRANSIENT_MAP_MAGIC, 1};
    unsigned char unaligned[9], transient_unaligned[9];
#ifdef NEOCLR_WORKER_GC
    neoclr_gc_worker_boundary_hook = boundary;
#endif
    if (argc > 1) {
        const char *mode = argv[1];
        int is_transient = !strncmp(mode, "transient-", 10);
        const char *fault = mode + (is_transient ? 10 : 8);
        neoclr_gc_root_map *map = is_transient ? &transient : &storage;
        uint32_t length = sizeof(*map);
        if (is_transient) neoclr_gc_enter_map_v1(&frame, &c, 0, &slot, 1, &storage, sizeof(storage));
        if (!strcmp(fault, "changed")) {
            if (is_transient) neoclr_gc_transient_map_v1(&frame, 3, &slot, 1, map, length);
            else {
                neoclr_gc_enter_map_v1(&frame, &c, 0, &slot, 1, map, length);
                neoclr_probe_leave_v1(&frame);
            }
            map->magic++;
        } else if (!strcmp(fault, "magic")) map->magic++;
        else if (!strcmp(fault, "kind")) map->magic = NEOCLR_GC_ROOT_MAP_MAGIC;
        else if (!strcmp(fault, "count")) map->lane_count++;
        else if (!strcmp(fault, "short")) length--;
        else if (!strcmp(fault, "long")) length++;
        else if (!strcmp(fault, "null")) map = NULL;
        else if (!strcmp(fault, "width")) slot.read_bytes = 12;
        else return 2;
        if (is_transient) neoclr_gc_transient_map_v1(&frame, 3, &slot, 1, map, length);
        else neoclr_gc_enter_map_v1(&frame, &c, 0, &slot, 1, map, length);
        return 0; /* Invalid headers/descriptors must abort. */
    }
    memcpy(unaligned + 1, &storage, sizeof(storage));
    neoclr_gc_enter_map_v1(&frame, &c, 0, &slot, 1, unaligned + 1, sizeof(storage));
    CHECK(frame.reserved & NEOCLR_GC_STORAGE_MAP);
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &first));
    value = first;
    publish(&frame);
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &second));
    value = second;
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    uint64_t bytes; uint32_t kind;
    CHECK(neoclr_gc_allocation_info_v1(&c, first, &bytes, &kind) == 3);
    CHECK(!neoclr_gc_allocation_info_v1(&c, second, &bytes, &kind));
    memcpy(transient_unaligned + 1, &transient, sizeof(transient));
    neoclr_gc_transient_map_v1(&frame, 3, &slot, 1, transient_unaligned + 1, sizeof(transient));
    CHECK(frame.reserved & NEOCLR_GC_TRANSIENT_MAP);
    neoclr_parked_roots parked = {0};
    neoclr_roots_park_v1(&parked);
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    CHECK(!neoclr_gc_allocation_info_v1(&c, second, &bytes, &kind));
    neoclr_roots_resume_v1(&parked);
    CHECK(frame.reserved & NEOCLR_GC_TRANSIENT_MAP);
    neoclr_probe_transient_v2(&frame, 3, &slot, 1, "{}", 2);
    CHECK(!(frame.reserved & (NEOCLR_GC_TRANSIENT_MAP | NEOCLR_GC_OWNED_TRANSIENT)));
    neoclr_gc_transient_map_v1(&frame, 3, &slot, 1, &transient, sizeof(transient));
    publish(&frame);
    CHECK(!(frame.reserved & NEOCLR_GC_TRANSIENT_MAP));
    CHECK(frame.reserved & NEOCLR_GC_STORAGE_MAP);
    value = NULL;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    neoclr_probe_leave_v1(&frame);
    CHECK(!frame.reserved);
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 0, "{}", 2);
    CHECK(!(frame.reserved & NEOCLR_GC_STORAGE_MAP));
    neoclr_probe_leave_v1(&frame);
    return 0;
}
