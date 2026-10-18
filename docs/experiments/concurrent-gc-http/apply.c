/* Experimental single-owner HTTP coordinator; worker touches only private copies. */
#include <assert.h>
#include <stdio.h>
#define neoclr_gc_boundary_v1 apply_sync_boundary
#define neoclr_gc_idle_v1 apply_sync_idle
#include "native-gc.c"
#undef neoclr_gc_boundary_v1
#undef neoclr_gc_idle_v1
#include "snapshot.inc"
static uint64_t trace_errors;
static bool take_checked(gc_worker *w, bool wait, bool cancel, int *status) {
    bool done = gc_worker_take(w, wait, cancel, status);
    if (done && *status && !(*status == 7 && cancel)) trace_errors++;
    return done;
}
#define gc_worker_take take_checked
#include "reclaim.inc"
#undef gc_worker_take
static collection_cycle active;
static bool initialized, stopped;
static uint64_t started, rejected, deferred, fallbacks;
static void reset_pacing(neoclr_aot_context *c) {
    pacing.context = c;
    pacing.allocations = statistics.allocations;
    pacing.collections = statistics.collections;
    pacing.boundaries = 0;
}
static int32_t collect_async(neoclr_aot_context *c, const neoclr_probe_frame *boundary, bool idle) {
    if (!c || resource_cleanup_active || boundary != neoclr_root_probe_head_v1() ||
        (boundary && boundary->context != c) || stopped) return 3;
    if (!initialized) {
        if (!cycle_init(&active)) return 3;
        initialized = true;
    }
    cycle_owner(&active);
    cycle_result result = cycle_poll(&active, false, false);
    if (result == CYCLE_APPLIED) {
        reset_pacing(c);
        return 0;
    }
    if (active.job) {
        if (active.context == c && active.epoch == experiment_epoch) {
            /* Rooted allocation pressure still collects synchronously and invalidates. */
            deferred++;
            return 0;
        }
        fallbacks++;
        return idle ? apply_sync_idle(c, boundary) : apply_sync_boundary(c, boundary);
    }
    if (!idle && pacing.context == c && statistics.collections == pacing.collections &&
        statistics.allocations - pacing.allocations < 16 && ++pacing.boundaries < 64) return 0;
    if (!cycle_start(&active, c)) {
        rejected++;
        return apply_sync_idle(c, boundary);
    }
    started++;
    atomic_store(&active.job->proceed, true);
    return 0;
}
int32_t neoclr_gc_boundary_v1(neoclr_aot_context *c, const neoclr_probe_frame *boundary) {
    return collect_async(c, boundary, false);
}
int32_t neoclr_gc_idle_v1(neoclr_aot_context *c, const neoclr_probe_frame *boundary) {
    return collect_async(c, boundary, true);
}
void neoclr_shadow_finish_v1(void) {
    if (initialized) {
        cycle_poll(&active, true, true);
        cycle_destroy(&active);
    }
    stopped = true;
}
static void report_apply(void) {
    assert(stopped && !active.job);
    assert(started == cycles_applied + cycles_discarded);
    const char *path = getenv("NEOCLR_GC_SHADOW_REPORT");
    if (!path) return;
    FILE *file = fopen(path, "w"); assert(file);
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    fprintf(file, "{\"started\":%llu,\"applied\":%llu,\"discarded\":%llu,\"concurrentReclaimedBytes\":%llu,\"traceErrors\":%llu,\"captureRejected\":%llu,\"deferred\":%llu,\"fallbacks\":%llu,\"allocations\":%llu,\"reclaimedAllocations\":%llu}\n",
        (unsigned long long)started, (unsigned long long)cycles_applied,
        (unsigned long long)cycles_discarded, (unsigned long long)cycle_reclaimed_bytes,
        (unsigned long long)trace_errors, (unsigned long long)rejected, (unsigned long long)deferred,
        (unsigned long long)fallbacks, (unsigned long long)s.allocations,
        (unsigned long long)s.reclaimed_allocations);
    assert(!fclose(file));
}
__attribute__((constructor)) static void setup_apply(void) { assert(!atexit(report_apply)); }
