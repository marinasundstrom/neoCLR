#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
__attribute__((destructor)) static void report(void) {
    const char *path = getenv("NEOCLR_JSON_GC_REPORT");
    if (!path) return;
    FILE *file = fopen(path, "w");
    if (!file) abort();
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    neoclr_gc_profile p = neoclr_gc_profile_v1();
    fprintf(file, "{\"allocations\":%" PRIu64 ",\"collections\":%" PRIu64
        ",\"boundaries\":%" PRIu64 ",\"gcNanoseconds\":%" PRIu64
        ",\"peakLiveBytes\":%" PRIu64 ",\"peakArenaUsed\":%" PRIu64
        ",\"failures\":%" PRIu64 "}\n",
        s.allocations, s.collections, neoclr_root_probe_count_v1(),
        p.nanoseconds, p.peak_live_bytes, p.peak_arena_used, p.failures);
    if (fclose(file)) abort();
}
