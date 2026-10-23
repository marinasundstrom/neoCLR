/* Diagnostic-only collection attribution; never linked into capacity executables. */
#include "native-gc.h"
#include "root-probe.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#ifdef NEOCLR_GC_SITE_PROFILE
/* Diagnostic-only, thread-local and bounded. Fail instead of dropping sites. */
enum { SITE_CAPACITY = 16384 };
typedef struct { uint32_t function, instruction; uint64_t count; } site_count;
static _Thread_local site_count sites[SITE_CAPACITY];
void neoclr_gc_site_profile_v1(uint32_t function, uint32_t instruction) {
    if (function >= NEOCLR_NATIVE_FUNCTION_LIMIT || instruction >= 8192) abort();
    uint32_t key = (function * 8192 + instruction) * UINT32_C(2654435761);
    uint32_t bucket = (key ^ (key >> 16)) & (SITE_CAPACITY - 1);
    for (uint32_t i = 0; i < SITE_CAPACITY; i++) {
        site_count *s = &sites[(bucket + i) & (SITE_CAPACITY - 1)];
        if (!s->count) { *s = (site_count){function, instruction, 1}; return; }
        if (s->function == function && s->instruction == instruction) {
            if (s->count == UINT64_MAX) abort();
            s->count++; return;
        }
    }
    abort();
}
static int by_count(const void *left, const void *right) {
    const site_count *a = left, *b = right;
    if (a->count != b->count) return a->count < b->count ? 1 : -1;
    if (a->function != b->function) return a->function < b->function ? -1 : 1;
    return (a->instruction > b->instruction) - (a->instruction < b->instruction);
}
static void report_sites(FILE *file) {
    uint64_t total = 0;
    qsort(sites, SITE_CAPACITY, sizeof(*sites), by_count);
    fprintf(file, ",\"sites\":[");
    for (uint32_t i = 0; i < SITE_CAPACITY && sites[i].count; i++) {
        const site_count *s = &sites[i];
        total += s->count;
        fprintf(file, "%s{\"function\":%u,\"instruction\":%u,\"count\":%" PRIu64 "}",
            i ? "," : "", s->function, s->instruction, s->count);
    }
    if (total != neoclr_root_probe_count_v1()) abort();
    fprintf(file, "]");
}
#endif
static void report(void) {
    const char *path = getenv("NEOCLR_HTTP_CAPACITY_PROFILE");
    if (!path) return;
    FILE *file = fopen(path, "w");
    if (!file) abort();
    neoclr_gc_profile p = neoclr_gc_profile_v1();
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    fprintf(file, "{\"attempts\":%" PRIu64 ",\"gcNanoseconds\":%" PRIu64
        ",\"maximumPauseNanoseconds\":%" PRIu64 ",\"failures\":%" PRIu64
        ",\"allocatedBytes\":%" PRIu64
        ",\"allocations\":%" PRIu64 ",\"reclaimed\":%" PRIu64
        ",\"peakLiveBytes\":%" PRIu64 ",\"peakArenaBytes\":%" PRIu64
        ",\"rootPublications\":%" PRIu64 ",\"pauseBuckets\":[",
        p.attempts, p.nanoseconds, p.maximum_nanoseconds, p.failures,
        p.allocated_bytes, s.allocations, s.reclaimed_allocations, p.peak_live_bytes,
        p.peak_arena_used, neoclr_root_probe_count_v1());
    for (unsigned i = 0; i < 64; i++) fprintf(file, "%s%" PRIu64, i ? "," : "", p.pause_buckets[i]);
    fprintf(file, "]");
#ifdef NEOCLR_GC_SITE_PROFILE
    report_sites(file);
#endif
    fprintf(file, "}\n");
    if (fclose(file)) abort();
}
__attribute__((constructor)) static void initialize(void) { if (atexit(report)) abort(); }
