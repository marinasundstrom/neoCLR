/* Collection-only native GC workload. Construction is deliberately untimed. */
#define _POSIX_C_SOURCE 200809L
#include "native-gc.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
static double seconds(void) {
    LARGE_INTEGER now, frequency;
    if (!QueryPerformanceFrequency(&frequency) || !QueryPerformanceCounter(&now)) abort();
    return (double)now.QuadPart / (double)frequency.QuadPart;
}
#else
#include <time.h>
static double seconds(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now)) abort();
    return (double)now.tv_sec + (double)now.tv_nsec / 1e9;
}
#endif
#ifdef LOOKUP_DIAGNOSTICS
extern uint64_t lookup_steps;
#endif
int main(int argc, char **argv) {
    if (argc != 4) return 1;
    int nodes = atoi(argv[1]), rounds = atoi(argv[2]);
    int holes = !strcmp(argv[3], "holes");
    uint64_t bytes = !strcmp(argv[3], "strings") ? 128 : !strcmp(argv[3], "buffers") ? 8192 : 16;
    uint32_t kind = !strcmp(argv[3], "objects") ? NEOCLR_GC_OBJECT : NEOCLR_GC_BYTES;
    if (nodes < 1 || nodes > 16384 || rounds < 1 || rounds > 10000) return 1;
    uint64_t capacity = (uint64_t)nodes * (bytes + 32) * (holes ? 2 : 1);
    unsigned char *buffer = malloc((size_t)capacity);
    void **values = calloc((size_t)nodes, sizeof(void *));
    neoclr_probe_storage *slots = calloc((size_t)nodes, sizeof(*slots));
    if (!buffer || !values || !slots) return 2;
    neoclr_aot_context c = {.text = {buffer, capacity, 0}};
    neoclr_probe_frame frame = {.context = &c, .storage = slots, .storage_count = (uint32_t)nodes};
    double elapsed = 0;
    uint64_t steps = 0;
    for (int round = 0; round < rounds; round++) {
        if (holes) {
            for (int i = 0; i < nodes; i++) {
                void *dead = NULL;
                if (neoclr_gc_allocate_v1(&c.text, bytes, kind, &dead) ||
                    neoclr_gc_allocate_v1(&c.text, bytes, kind, &values[i])) return 3;
                ((uint64_t *)values[i])[1] = (uint64_t)i;
                slots[i] = (neoclr_probe_storage){&values[i], 8, 0};
            }
            if (neoclr_gc_collect_v1(&c, &frame)) return 4;
        }
#ifdef LOOKUP_DIAGNOSTICS
        lookup_steps = 0;
#endif
        double start = seconds();
        for (int i = 0; i < nodes; i++) {
            void *p = NULL;
            if (neoclr_gc_allocate_v1(&c.text, bytes, kind, &p)) return 3;
            ((uint64_t *)p)[1] = (uint64_t)i;
            if (!holes) values[i] = p;
        }
        elapsed += seconds() - start;
#ifdef LOOKUP_DIAGNOSTICS
        steps += lookup_steps;
#endif
        for (int i = 0; i < nodes; i++)
            if (!values[i] || ((uint64_t *)values[i])[1] != (uint64_t)i) return 5;
        if (neoclr_gc_collect_v1(&c, NULL) || c.text.used) return 6;
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.allocations != stats.reclaimed_allocations) return 6;
    printf("{\"seconds\":%.9f,\"lookupSteps\":%llu,\"reclaimed\":%llu}\n", elapsed,
        (unsigned long long)steps, (unsigned long long)stats.reclaimed_allocations);
    free(slots); free(values); free(buffer);
    return 0;
}
