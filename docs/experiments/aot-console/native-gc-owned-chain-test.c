#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static uint64_t empty;
static const neoclr_gc_root_map map = {NEOCLR_GC_ROOT_MAP_MAGIC, 0};
#ifdef NEOCLR_WORKER_GC
static unsigned worker_calls;
static void worker_boundary(neoclr_aot_context *context, const neoclr_probe_frame *frame, uint32_t exclusive) {
    if (exclusive != 1 || frame != neoclr_root_probe_head_v1()) abort();
    worker_calls++;
    if (neoclr_gc_collect_v1(context, frame)) abort();
}
#endif
static void publish(neoclr_probe_frame *frame) {
#ifdef NEOCLR_WORKER_GC
    neoclr_gc_worker_stack_map_v1(frame, 0, &empty, 0, &map, sizeof(map), 1);
#else
    neoclr_gc_stack_map_v1(frame, 0, &empty, 0, &map, sizeof(map));
#endif
}
int main(int argc, char **argv) {
#ifdef _WIN32
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    uint64_t heap[1024] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}}, other = {0};
    neoclr_probe_frame frames[16], child;
    void *value = NULL, *first = NULL, *second = NULL;
    neoclr_probe_storage slot = {&value, 8, 0};
#ifdef NEOCLR_WORKER_GC
    neoclr_gc_worker_boundary_hook = worker_boundary;
#endif
    neoclr_gc_enter_owned_v1(&frames[0], &c, 0, &slot, 1, "{}", 2);
    if (argc > 1) {
        if (!strcmp(argv[1], "foreign-fault")) {
            neoclr_gc_enter_owned_v1(&child, &other, 1, &slot, 0, "{}", 2);
            c.fault.code = 1; c.fault.frame_count = 65;
        } else if (!strcmp(argv[1], "fault")) {
            neoclr_gc_enter_owned_v1(&child, &c, 1, &slot, 0, "{}", 2);
            c.fault.code = 1; c.fault.frame_count = 65;
        } else if (!strcmp(argv[1], "mutable-transient")) {
            neoclr_probe_transient_v2(&frames[0], 3, &slot, 1, "{}", 2);
            neoclr_gc_enter_owned_v1(&child, &c, 1, &slot, 0, "{}", 2);
            slot.read_bytes = 12;
        } else return 2;
        publish(&child); /* Every malformed case must abort, even under pacing. */
        return 0;
    }
    for (unsigned i = 1; i < 16; i++) {
        neoclr_gc_enter_owned_v1(&frames[i], &c, i, &slot, 0, "{}", 2);
        CHECK(frames[i].reserved & NEOCLR_GC_OWNED_CHAIN);
    }
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &first));
    value = first;
    publish(&frames[15]);
    CHECK(!neoclr_gc_collect_v1(&c, &frames[15]));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &second));
    value = second;
    publish(&frames[15]);
    CHECK(!neoclr_gc_collect_v1(&c, &frames[15]));
    uint64_t bytes; uint32_t kind;
    CHECK(neoclr_gc_allocation_info_v1(&c, first, &bytes, &kind) == 3);
    CHECK(!neoclr_gc_allocation_info_v1(&c, second, &bytes, &kind));
    neoclr_parked_roots parked = {0};
    neoclr_roots_park_v1(&parked);
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    CHECK(!neoclr_gc_allocation_info_v1(&c, second, &bytes, &kind));
    neoclr_roots_resume_v1(&parked);
    value = NULL;
    publish(&frames[15]);
    CHECK(!neoclr_gc_collect_v1(&c, &frames[15]) && !c.text.used);
    for (unsigned i = 16; i > 1; i--) neoclr_probe_leave_v1(&frames[i-1]);
    /* Ownership of a frame's own transients is checked when it becomes a parent. */
    neoclr_probe_transient_v2(&frames[0], 3, &slot, 1, "{}", 2);
    neoclr_gc_enter_owned_v1(&child, &c, 1, &slot, 0, "{}", 2);
    CHECK(!(child.reserved & NEOCLR_GC_OWNED_CHAIN));
    publish(&child);
    neoclr_probe_leave_v1(&child);
    neoclr_gc_transient_owned_v1(&frames[0], 3, &slot, 1, "{}", 2);
    neoclr_gc_enter_owned_v1(&child, &c, 1, &slot, 0, "{}", 2);
    CHECK(child.reserved & NEOCLR_GC_OWNED_CHAIN);
    publish(&child);
    neoclr_probe_leave_v1(&child);
    neoclr_gc_enter_owned_v1(&child, &other, 1, &slot, 0, "{}", 2);
    CHECK(!(child.reserved & NEOCLR_GC_OWNED_CHAIN));
    publish(&child);
    neoclr_probe_leave_v1(&child);
    neoclr_probe_leave_v1(&frames[0]);
    neoclr_probe_enter_v3(&frames[0], &c, 0, &slot, 0, "{}", 2);
    neoclr_gc_enter_owned_v1(&child, &c, 1, &slot, 0, "{}", 2);
    CHECK(!(child.reserved & NEOCLR_GC_OWNED_CHAIN));
    publish(&child);
    neoclr_probe_leave_v1(&child);
    neoclr_probe_leave_v1(&frames[0]);
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
#ifdef NEOCLR_WORKER_GC
    CHECK(worker_calls == 7);
#endif
    return 0;
}
