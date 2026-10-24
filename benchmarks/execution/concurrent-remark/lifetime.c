#include "native-gc.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern void experiment_gc_shadow_gate(int);
extern void experiment_gc_shadow(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
static uint64_t *object(neoclr_aot_context *c) {
    uint64_t *value = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c->text, 16, NEOCLR_GC_OBJECT, (void **)&value));
    value[0] = 17;
    return value;
}
int main(void) {
    for (unsigned trial = 0; trial < 100; trial++) {
        uint64_t heap[512] = {0};
        neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}}, other = {0};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *a = object(&c), *hole = object(&c), *b = object(&c), *dead = object(&c);
        uint64_t *white = object(&c), *child = object(&c), *changed = object(&c);
        a[1] = (uintptr_t)b; b[1] = (uintptr_t)a;
        white[1] = (uintptr_t)child;
        void *root = a;
        neoclr_probe_storage slot = {&root, 8, 0};
        neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
        uint64_t keep[] = {(uintptr_t)a, (uintptr_t)b, (uintptr_t)dead,
            (uintptr_t)white, (uintptr_t)child, (uintptr_t)changed};
        neoclr_probe_frame preparation = {.context = &c, .lanes = keep, .lane_count = 6};
        CHECK(!neoclr_gc_collect_v1(&c, &preparation));
        CHECK(*hole == UINT64_C(0xdddddddddddddddd));
        experiment_gc_shadow_gate(1);
        CHECK(!experiment_gc_begin(&c, &frame, NULL, 0));
        CHECK(experiment_gc_begin(&c, &frame, NULL, 0) == 3);
        CHECK(experiment_gc_finish(&other, NULL, NULL, 0) == 3);
        CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
        CHECK(neoclr_gc_heap_release_v1(&c.text) == 3);
        CHECK(*dead == 17); /* begin has not reclaimed anything */
        uint64_t *fresh = object(&c);
        CHECK(fresh == hole);
        fresh[1] = (uintptr_t)white;
        a[1] = (uintptr_t)fresh;
        changed[1] = (uintptr_t)child; /* unreachable mutation is retained conservatively */
        experiment_gc_shadow_gate(0);
        CHECK(!experiment_gc_finish(&c, &frame, NULL, 0));
        CHECK(*dead == UINT64_C(0xdddddddddddddddd));
        CHECK(*a == 17 && *b == 17 && *fresh == 17 && *white == 17 && *child == 17 && *changed == 17);
        CHECK(experiment_gc_finish(&c, &frame, NULL, 0) == 3);
        root = NULL;
        CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    /* A weak/conditional value exposed into a new strong root after capture must
     * survive even when its original owner was never strongly reachable. */
    uint64_t heap[128] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    uint64_t *owner = object(&c), *value = object(&c), *child = object(&c);
    value[1] = (uintptr_t)child;
    uint64_t handle;
    CHECK(!neoclr_gc_host_root_create_owned_v1(&c, owner, value, &handle));
    void *root = NULL;
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    CHECK(!experiment_gc_begin(&c, &frame, NULL, 0));
    CHECK(!neoclr_gc_host_root_read_v1(&c, handle, &root));
    CHECK(!experiment_gc_finish(&c, &frame, NULL, 0));
    CHECK(*owner == UINT64_C(0xdddddddddddddddd) && *value == 17 && *child == 17);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    root = NULL;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    /* A malformed payload discovered asynchronously must fail before sweeping,
     * clear private marks and leave the synchronous recovery path usable. */
    uint64_t *bad = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_STRINGS, (void **)&bad));
    root = bad;
    bad[0] = 0;
    CHECK(!experiment_gc_begin(&c, &frame, NULL, 0));
    CHECK(experiment_gc_finish(&c, &frame, NULL, 0) == 3);
    CHECK(bad[0] == 0);
    bad[0] = UINT64_C(0x80000003);
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    root = NULL;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    uint64_t stats[5]; experiment_gc_shadow(stats);
    puts("101 concurrent reclamation and exposure cycles passed");
    return 0;
}
