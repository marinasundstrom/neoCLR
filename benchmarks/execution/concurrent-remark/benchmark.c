#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern void experiment_gc_shadow(uint64_t *);
#define CHECK(x) do { if (!(x)) abort(); } while (0)
static uint64_t now(clockid_t clock) {
    struct timespec t; CHECK(!clock_gettime(clock, &t));
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
static uint64_t *node(neoclr_aot_context *c) {
    uint64_t *p = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c->text, 24, NEOCLR_GC_OBJECT, (void **)&p));
    p[0] = 17;
    return p;
}
int main(int argc, char **argv) {
    CHECK(argc == 2);
    int concurrent = !strcmp(argv[1], "concurrent");
    CHECK(concurrent || !strcmp(argv[1], "sync"));
    neoclr_aot_context c = {.text = {calloc(1, 1024*1024), 1024*1024, 0}};
    CHECK(c.text.data && !neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root = node(&c), *last = root;
    for (unsigned i = 1; i < 4096; i++) {
        uint64_t *next = node(&c); last[1] = (uintptr_t)next; last = next;
    }
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    uint64_t wall = now(CLOCK_MONOTONIC), cpu = now(CLOCK_PROCESS_CPUTIME_ID);
    uint64_t peak = c.text.used;
    volatile uint64_t checksum = 1;
    for (unsigned round = 0; round < 10000; round++) {
        if (concurrent) CHECK(!experiment_gc_begin(&c, &frame, NULL, 0));
        for (unsigned i = 0; i < 64; i++) root[2] = (uintptr_t)node(&c);
        for (unsigned i = 0; i < 10000; i++) checksum = checksum * 33 + i;
        if (c.text.used > peak) peak = c.text.used;
        if (concurrent) CHECK(!experiment_gc_finish(&c, &frame, NULL, 0));
        else CHECK(!neoclr_gc_collect_v1(&c, &frame));
        CHECK(root[0] == 17 && ((uint64_t *)(uintptr_t)root[2])[0] == 17);
    }
    cpu = now(CLOCK_PROCESS_CPUTIME_ID) - cpu;
    wall = now(CLOCK_MONOTONIC) - wall;
    unsigned count = 0;
    for (uint64_t *p = root; p; p = (void *)(uintptr_t)p[1]) { CHECK(p[0] == 17); count++; }
    CHECK(count == 4096);
    printf("%s %llu %llu %llu %llu\n",argv[1],(unsigned long long)wall,
        (unsigned long long)cpu,(unsigned long long)peak,(unsigned long long)checksum);
    root = NULL;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    free(c.text.data);
    return 0;
}
