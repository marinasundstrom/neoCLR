/* Diagnostic-only exit counters; never linked into published runtime hosts. */
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
static void report(void) {
    const char *path = getenv("NEOCLR_HTTP_PROFILE_OUTPUT");
    if (!path) return;
    FILE *file = fopen(path, "w");
    if (!file) abort();
    neoclr_gc_profile p = neoclr_gc_profile_v1();
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    fprintf(file, "{\"attempts\":%" PRIu64 ",\"gcNanoseconds\":%" PRIu64
        ",\"failures\":%" PRIu64 ",\"allocations\":%" PRIu64
        ",\"reclaimed\":%" PRIu64 ",\"allocatedBytes\":%" PRIu64
        ",\"peakLiveBytes\":%" PRIu64 ",\"peakArenaBytes\":%" PRIu64 "}\n",
        p.attempts, p.nanoseconds, p.failures, s.allocations, s.reclaimed_allocations,
        p.allocated_bytes, p.peak_live_bytes, p.peak_arena_used);
    if (fclose(file)) abort();
}
__attribute__((constructor)) static void initialize(void) { if (atexit(report)) abort(); }
