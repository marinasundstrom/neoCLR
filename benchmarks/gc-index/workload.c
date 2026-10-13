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
extern uint64_t lookup_steps, validation_steps;
#endif
int main(int argc, char **argv) {
    if (argc != 4) return 1;
    int nodes = atoi(argv[1]), rounds = atoi(argv[2]);
    int atomic = !strcmp(argv[3], "atomic"), wide = !strcmp(argv[3], "wide");
    int holes = !strcmp(argv[3], "holes");
    if (nodes < 1 || nodes > 65536 || rounds < 1 || rounds > 100000) return 1;
    uint64_t bytes = atomic ? 16384 : 16;
    uint64_t capacity = (uint64_t)nodes * (bytes + 80) + 64;
    unsigned char *buffer = malloc((size_t)capacity);
    if (!buffer) return 2;
    neoclr_aot_context c = {.text = {buffer, capacity, 0}};
    uint64_t root = 0;
    void *hub = NULL;
    if ((wide || atomic) && neoclr_gc_allocate_v1(&c.text, 8 + (uint64_t)nodes * 8, NEOCLR_GC_OBJECT, &hub)) return 3;
    for (int i = 0; i < nodes; i++) {
        void *p = NULL, *dead = NULL;
        if (neoclr_gc_allocate_v1(&c.text, bytes, atomic ? NEOCLR_GC_TEXT : NEOCLR_GC_OBJECT, &p)) return 3;
        ((uint64_t *)p)[1] = wide || atomic ? (uint64_t)i : root;
        root = (uintptr_t)p;
        if (hub) ((uint64_t *)hub)[i + 1] = root;
        if (holes && neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_TEXT, &dead)) return 3;
    }
    if (hub) root = (uintptr_t)hub;
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    if (neoclr_gc_collect_v1(&c, &frame)) return 4;
#ifdef LOOKUP_DIAGNOSTICS
    lookup_steps = validation_steps = 0;
#endif
    double start = seconds();
    for (int i = 0; i < rounds; i++) if (neoclr_gc_collect_v1(&c, &frame)) return 4;
    double elapsed = seconds() - start;
#ifdef LOOKUP_DIAGNOSTICS
    uint64_t steps = lookup_steps, validations = validation_steps;
#else
    uint64_t steps = 0, validations = 0;
#endif
    uint64_t cursor = root;
    for (int i = 0; i < nodes; i++) {
        if (hub) {
            cursor = ((uint64_t *)hub)[i + 1];
            if (!cursor || ((uint64_t *)(uintptr_t)cursor)[1] != (uint64_t)i) return 5;
        } else {
            if (!cursor) return 5;
            cursor = ((uint64_t *)(uintptr_t)cursor)[1];
        }
    }
    if ((!hub && cursor) || neoclr_gc_statistics_v1().reclaimed_allocations != (holes ? (uint64_t)nodes : 0)) return 5;
    root = 0;
    if (neoclr_gc_collect_v1(&c, &frame) || c.text.used ||
        neoclr_gc_statistics_v1().allocations != neoclr_gc_statistics_v1().reclaimed_allocations) return 6;
    printf("{\"seconds\":%.9f,\"lookupSteps\":%llu,\"validationSteps\":%llu,\"reclaimed\":%llu}\n", elapsed,
        (unsigned long long)steps, (unsigned long long)validations, (unsigned long long)neoclr_gc_statistics_v1().reclaimed_allocations);
    free(buffer);
    return 0;
}
