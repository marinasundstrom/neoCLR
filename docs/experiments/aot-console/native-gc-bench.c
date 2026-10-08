/* Collection-only reverse-chain microbenchmark; construction is untimed.
 * Deliberately adversarial allocation order, not an application throughput claim. */
#define _POSIX_C_SOURCE 200809L
#include "native-gc.h"
#include <stdlib.h>
#include <time.h>
#include <stdio.h>
static double seconds(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) abort();
    return (double)t.tv_sec + (double)t.tv_nsec / 1e9;
}
int main(void) {
    enum { NODES = 1024, ROUNDS = 100 };
    unsigned char *buffer = malloc(NODES * 48);
    if (!buffer) return 1;
    neoclr_aot_context c = {.text = {buffer, NODES * 48, 0}};
    uint64_t root = 0;
    for (int i = 0; i < NODES; i++) {
        void *p = NULL;
        if (neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &p)) return 2;
        ((uint64_t *)p)[1] = root; root = (uintptr_t)p;
    }
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    if (neoclr_gc_collect_v1(&c, &frame)) return 3;
    double start = seconds();
    for (int i = 0; i < ROUNDS; i++) if (neoclr_gc_collect_v1(&c, &frame)) return 3;
    double elapsed = seconds() - start;
    uint64_t cursor = root;
    for (int i = 0; i < NODES; i++) {
        if (!cursor) return 4;
        cursor = ((uint64_t *)(uintptr_t)cursor)[1];
    }
    if (cursor || c.text.used != NODES * 48 || neoclr_gc_statistics_v1().reclaimed_allocations) return 4;
    root = 0;
    if (neoclr_gc_collect_v1(&c, &frame) || c.text.used) return 5;
    printf("%.9f\n", elapsed);
    free(buffer);
    return 0;
}
