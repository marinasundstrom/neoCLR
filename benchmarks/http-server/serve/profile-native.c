/* Diagnostic-only collection attribution; never linked into capacity executables. */
#include "native-gc.h"
#include "root-probe.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
static void report(void) {
    const char *path = getenv("NEOCLR_HTTP_CAPACITY_PROFILE");
    if (!path) return;
    FILE *file = fopen(path, "w");
    if (!file) abort();
    neoclr_gc_profile p = neoclr_gc_profile_v1();
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    fprintf(file, "{\"attempts\":%" PRIu64 ",\"gcNanoseconds\":%" PRIu64
        ",\"maximumPauseNanoseconds\":%" PRIu64 ",\"failures\":%" PRIu64
        ",\"allocations\":%" PRIu64 ",\"reclaimed\":%" PRIu64
        ",\"peakLiveBytes\":%" PRIu64 ",\"peakArenaBytes\":%" PRIu64
        ",\"rootPublications\":%" PRIu64 ",\"pauseBuckets\":[",
        p.attempts, p.nanoseconds, p.maximum_nanoseconds, p.failures,
        s.allocations, s.reclaimed_allocations, p.peak_live_bytes,
        p.peak_arena_used, neoclr_root_probe_count_v1());
    for (unsigned i = 0; i < 64; i++) fprintf(file, "%s%" PRIu64, i ? "," : "", p.pause_buckets[i]);
    fprintf(file, "]}\n");
    if (fclose(file)) abort();
}
__attribute__((constructor)) static void initialize(void) { if (atexit(report)) abort(); }
