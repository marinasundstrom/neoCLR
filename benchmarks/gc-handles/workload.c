/* Owned-handle admission workload. Heap construction is deliberately untimed. */
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
int main(int argc, char **argv) {
    if (argc != 5) return 1;
    int nodes = atoi(argv[1]), rounds = atoi(argv[2]), position = atoi(argv[3]), owner_position = atoi(argv[4]);
    if (nodes < 1 || rounds < 1 || position < 0 || position >= nodes || owner_position < -1 || owner_position >= nodes) return 1;
    uint64_t capacity = (uint64_t)nodes * 64;
    unsigned char *buffer = malloc((size_t)capacity);
    if (!buffer) return 2;
    neoclr_aot_context c = {.text = {buffer, capacity, 0}};
    void *value = NULL, *owner = NULL;
    for (int i = 0; i < nodes; i++) {
        void *p = NULL;
        if (neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &p)) return 3;
        if (i == position) value = p;
        if (i == owner_position) owner = p;
    }
    double start = seconds();
    for (int i = 0; i < rounds; i++) {
        uint64_t handle = 0;
        int32_t status = owner_position < 0
            ? neoclr_gc_host_root_create_v1(&c, value, &handle)
            : neoclr_gc_host_root_create_owned_v1(&c, owner, value, &handle);
        if (status) return 4;
        void *read = NULL;
        if (neoclr_gc_host_root_read_v1(&c, handle, &read) || read != value) return 5;
        if (neoclr_gc_host_root_release_v1(&c, handle)) return 6;
    }
    double elapsed = seconds() - start;
    if (neoclr_gc_entry_check_v1(&c) || neoclr_gc_collect_v1(&c, NULL) || c.text.used) return 7;
    printf("{\"seconds\":%.9f,\"operations\":%d}\n", elapsed, rounds);
    free(buffer);
    return 0;
}
