/* A reserve-only boundary policy can reject a recoverable large allocation. */
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"check failed at %d\n",__LINE__); return 1; } } while (0)
int main(void) {
    unsigned char *memory = calloc(1, 4 * 1024 * 1024);
    CHECK(memory);
    neoclr_aot_context c = {.text = {memory, 4 * 1024 * 1024, 0}};
    void *root = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 1024 * 1024, NEOCLR_GC_BYTES, &root));
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame;
    uint64_t lanes[1] = {0};
    const char plan[] = "{\"requiredSpillLanes\":[]}";
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    neoclr_probe_stack_roots_v2(&frame, 0, lanes, 0, plan, sizeof(plan)-1);
    root = NULL;
    neoclr_probe_stack_roots_v2(&frame, 1, lanes, 0, plan, sizeof(plan)-1);
    void *large = NULL;
    int status = neoclr_gc_allocate_v1(&c.text, 7 * 512 * 1024, NEOCLR_GC_BYTES, &large);
#ifdef EXPECT_PACING_GAP
    CHECK(status == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT && !large);
    CHECK(!neoclr_gc_collect_v1(&c, neoclr_root_probe_head_v1()));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 7 * 512 * 1024, NEOCLR_GC_BYTES, &large));
#else
    CHECK(!status && large);
#endif
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    free(memory);
    puts("PASS pressure admission witness");
    return 0;
}
