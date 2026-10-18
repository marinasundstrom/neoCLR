/* Diagnostic only: bounded static storage; never retain pointers into guest frames.
 * Payload bytes include array/object representation, not logical element count.
 * Thread-affine like the sampled native host. No allocation in the capture hook. */
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#include <string.h>
#ifndef CALLER_CAPACITY
#define CALLER_CAPACITY 32768
#endif
#define CALLER_DEPTH 4
typedef struct {
    uint32_t functions[CALLER_DEPTH], instructions[CALLER_DEPTH], depth, truncated, kind;
    uint64_t size, count;
} caller_row;
static caller_row rows[CALLER_CAPACITY];
static uint64_t dropped, dropped_bytes, allocations, allocated_bytes;
static uint64_t boundary_reasons[3], idle_calls;
void record_boundary(unsigned reason) {
    if (reason >= 3) abort();
    boundary_reasons[reason]++;
}
void record_idle(void) { idle_calls++; }
void record_caller(uint32_t kind, uint64_t bytes) {
    caller_row key = {0};
    key.kind = kind; key.size = bytes;
    const neoclr_probe_frame *frame = neoclr_root_probe_head_v1();
    uint64_t hash = bytes * 31 + kind;
    while (frame && key.depth < CALLER_DEPTH) {
        if (frame->function >= NEOCLR_NATIVE_FUNCTION_LIMIT) abort();
        key.functions[key.depth] = frame->function;
        key.instructions[key.depth++] = frame->instruction;
        hash = hash * 31 + frame->function;
        hash = hash * 31 + frame->instruction;
        frame = frame->previous;
    }
    key.truncated = frame != NULL;
    hash = hash * 31 + key.truncated;
    allocations++; allocated_bytes += bytes;
    for (unsigned probe = 0; probe < CALLER_CAPACITY; probe++) {
        caller_row *row = &rows[(hash + probe) % CALLER_CAPACITY];
        if (!row->count) { *row = key; row->count = 1; return; }
        if (row->kind == kind && row->size == bytes && row->depth == key.depth &&
            row->truncated == key.truncated &&
            !memcmp(row->functions, key.functions, sizeof(key.functions)) &&
            !memcmp(row->instructions, key.instructions, sizeof(key.instructions))) {
            row->count++; return;
        }
    }
    dropped++; dropped_bytes += bytes;
}
void report_callers(FILE *file) {
    fprintf(file, "{\"allocations\":%" PRIu64 ",\"allocatedBytes\":%" PRIu64
        ",\"dropped\":%" PRIu64 ",\"droppedBytes\":%" PRIu64
        ",\"idleCalls\":%" PRIu64 ",\"boundaryReasons\":[%" PRIu64 ",%" PRIu64 ",%" PRIu64 "],\"rows\":[",
        allocations, allocated_bytes, dropped, dropped_bytes, idle_calls,
        boundary_reasons[0], boundary_reasons[1], boundary_reasons[2]);
    int comma = 0;
    for (unsigned i = 0; i < CALLER_CAPACITY; i++) {
        const caller_row *row = &rows[i];
        if (!row->count) continue;
        fprintf(file, "%s{\"kind\":%u,\"size\":%" PRIu64 ",\"count\":%" PRIu64
            ",\"truncated\":%u,\"frames\":[", comma++ ? "," : "", row->kind, row->size, row->count, row->truncated);
        for (unsigned j = 0; j < row->depth; j++)
            fprintf(file, "%s{\"function\":%u,\"instruction\":%u}", j ? "," : "", row->functions[j], row->instructions[j]);
        fputs("]}", file);
    }
    fputs("]}\n", file);
}
#ifndef CALLER_TEST
static void write_callers(void) {
    const char *path = getenv("NEOCLR_ALLOCATION_CALLERS");
    if (!path) abort();
    FILE *file = fopen(path, "w");
    if (!file) abort();
    report_callers(file);
    if (fclose(file)) abort();
}
__attribute__((constructor)) static void initialize_callers(void) {
    if (atexit(write_callers)) abort();
}
#endif
