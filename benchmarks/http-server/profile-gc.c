/* Diagnostic-only exit counters; never linked into published runtime hosts. */
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
static uint64_t requests, warm_allocations, warm_bytes, warm_text, warm_text_bytes;
static uint64_t measured_allocations, measured_bytes, measured_text, measured_text_bytes;
static uint64_t text_allocations, text_bytes;
void neoclr_http_profile_allocated(uint32_t kind, uint64_t bytes) {
    if (kind == NEOCLR_GC_TEXT) { text_allocations++; text_bytes += bytes; }
}
/* Called by a diagnostic copy of the socket adapter immediately before closing
 * each accepted connection. No managed allocation, roots or collection here. */
void neoclr_http_profile_request(void) {
    requests++;
    if (requests != 8 && requests != 32) return;
    neoclr_gc_profile p = neoclr_gc_profile_v1();
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    if (requests == 8) {
        warm_allocations = s.allocations; warm_bytes = p.allocated_bytes;
        warm_text = text_allocations; warm_text_bytes = text_bytes;
    } else {
        measured_allocations = s.allocations - warm_allocations;
        measured_bytes = p.allocated_bytes - warm_bytes;
        measured_text = text_allocations - warm_text;
        measured_text_bytes = text_bytes - warm_text_bytes;
    }
}
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
    fprintf(file, "{\"requests\":%" PRIu64 ",\"measuredRequests\":24,\"allocations\":%" PRIu64
        ",\"allocatedBytes\":%" PRIu64 ",\"textBlocks\":%" PRIu64 ",\"textBytes\":%" PRIu64 "}\n",
        requests, measured_allocations, measured_bytes, measured_text, measured_text_bytes);
    if (fclose(file)) abort();
}
__attribute__((constructor)) static void initialize(void) { if (atexit(report)) abort(); }
