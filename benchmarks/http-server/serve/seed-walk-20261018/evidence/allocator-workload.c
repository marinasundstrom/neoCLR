#define _POSIX_C_SOURCE 200809L
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>
typedef struct { uint64_t span, bytes, kind, state; } header;
int main(int argc, char **argv) {
    if (argc != 3) return 1;
    unsigned count = (unsigned)atoi(argv[1]);
    unsigned hole = (unsigned)atoi(argv[2]);
    if (!count || hole > count) return 2;
    uint64_t iterations = 100000000 / count;
    unsigned char *heap = calloc(count + 1, 40);
    if (!heap) return 3;
    neoclr_aot_text_arena arena = {heap, (count + 1) * UINT64_C(40), count * UINT64_C(40)};
    for (unsigned i = 0; i < count; i++)
        *(header *)(heap + 40 * i) = (header){40, 8, NEOCLR_GC_BYTES, 1};
    struct timespec a, b;
    clock_gettime(CLOCK_MONOTONIC, &a);
    for (uint64_t i = 0; i < iterations; i++) {
        if (hole < count) ((header *)(heap + 40 * hole))->state = 0;
        arena.used = count * UINT64_C(40);
        void *output = NULL;
        if (neoclr_gc_allocate_v1(&arena, 8, NEOCLR_GC_BYTES, &output) || output != heap + 40 * hole + 32) return 4;
        if (*(uint64_t *)output) return 5;
        *(uint64_t *)output = UINT64_MAX;
    }
    clock_gettime(CLOCK_MONOTONIC, &b);
    printf("%.9f\n", (b.tv_sec-a.tv_sec)+(b.tv_nsec-a.tv_nsec)*1e-9);
    free(heap);
    return 0;
}
