/* Observe private traces only. This file never imports marks or sweeps live state. */
#include <assert.h>
#include <stdio.h>
#define neoclr_gc_boundary_v1 shadow_sync_boundary
#define neoclr_gc_idle_v1 shadow_sync_idle
#include "native-gc.c"
#undef neoclr_gc_boundary_v1
#undef neoclr_gc_idle_v1
#include "snapshot.inc"
static struct {
    gc_worker worker;
    pthread_t owner;
    snapshot *job;
    uint64_t epoch, opportunities, started, completed, eligible, invalidated, rejected, errors, cancelled;
    uint64_t peak_snapshot_bytes;
    bool initialized, stopped;
} shadow;
static void poll_shadow(bool stopping) {
    if (!shadow.job) return;
    int result;
    if (!gc_worker_take(&shadow.worker, stopping, stopping, &result)) return;
    if (result == 7 && stopping) shadow.cancelled++;
    else if (result) shadow.errors++;
    else {
        shadow.completed++;
        if (shadow.epoch == experiment_epoch) shadow.eligible++;
        else shadow.invalidated++;
    }
    free(shadow.job->copy.text.data);
    free(shadow.job);
    shadow.job = NULL;
}
static void observe(neoclr_aot_context *c) {
    if (shadow.stopped) return;
    if (!shadow.initialized) {
        assert(!gc_worker_init(&shadow.worker));
        shadow.owner = pthread_self();
        shadow.initialized = true;
    }
    assert(pthread_equal(shadow.owner, pthread_self()));
    /* Sample one in 64 boundary opportunities; never queue a second snapshot. */
    if (++shadow.opportunities % 64 || shadow.job) return;
    snapshot *s = malloc(sizeof(*s));
    if (!s || !capture(s, c)) { free(s); shadow.rejected++; return; }
    shadow.job = s;
    shadow.epoch = experiment_epoch;
    shadow.started++;
    if (s->copy.text.used > shadow.peak_snapshot_bytes) shadow.peak_snapshot_bytes = s->copy.text.used;
    atomic_store(&s->proceed, true);
    assert(gc_worker_submit(&shadow.worker, trace_snapshot, s));
}
int32_t neoclr_gc_boundary_v1(neoclr_aot_context *c, const neoclr_probe_frame *boundary) {
    poll_shadow(false);
    int32_t status = shadow_sync_boundary(c, boundary);
    if (!status) observe(c);
    return status;
}
int32_t neoclr_gc_idle_v1(neoclr_aot_context *c, const neoclr_probe_frame *boundary) {
    poll_shadow(false);
    int32_t status = shadow_sync_idle(c, boundary);
    if (!status) observe(c);
    return status;
}
void neoclr_shadow_finish_v1(void) {
    if (shadow.initialized) {
        assert(pthread_equal(shadow.owner, pthread_self()));
        poll_shadow(true);
        gc_worker_destroy(&shadow.worker);
    }
    shadow.stopped = true;
}
static void report_shadow(void) {
    assert(shadow.stopped && !shadow.job);
    assert(shadow.started == shadow.completed + shadow.errors + shadow.cancelled);
    const char *path = getenv("NEOCLR_GC_SHADOW_REPORT");
    if (!path) return;
    FILE *file = fopen(path, "w"); assert(file);
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    fprintf(file, "{\"opportunities\":%llu,\"started\":%llu,\"completed\":%llu,\"epochUnchanged\":%llu,\"invalidated\":%llu,\"captureRejected\":%llu,\"traceErrors\":%llu,\"cancelledAtShutdown\":%llu,\"peakSnapshotBytes\":%llu,\"allocations\":%llu,\"reclaimedAllocations\":%llu,\"applied\":0}\n",
        (unsigned long long)shadow.opportunities, (unsigned long long)shadow.started,
        (unsigned long long)shadow.completed, (unsigned long long)shadow.eligible,
        (unsigned long long)shadow.invalidated, (unsigned long long)shadow.rejected,
        (unsigned long long)shadow.errors, (unsigned long long)shadow.cancelled,
        (unsigned long long)shadow.peak_snapshot_bytes, (unsigned long long)s.allocations,
        (unsigned long long)s.reclaimed_allocations);
    assert(!fclose(file));
}
__attribute__((constructor)) static void setup_shadow(void) { assert(!atexit(report_shadow)); }
