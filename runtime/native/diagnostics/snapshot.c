#include "snapshot.h"
#include "../gc/native-gc.h"
#ifdef NEOCLR_RUNTIME_ASYNC
#include "../async/native-runtime.h"
#endif
static neoclr_runtime_snapshot empty(void) {
    return (neoclr_runtime_snapshot){.version = 1, .size = sizeof(neoclr_runtime_snapshot)};
}
#define SET(metric, value) do { s.values[metric] = (value); s.available |= UINT64_C(1) << (metric); } while (0)
neoclr_runtime_snapshot neoclr_runtime_snapshot_v1(void) {
    neoclr_runtime_snapshot s = empty();
#ifdef NEOCLR_NATIVE_GC
    neoclr_gc_statistics g = neoclr_gc_statistics_v1();
    SET(NEOCLR_METRIC_COLLECTIONS, g.collections);
    SET(NEOCLR_METRIC_ALLOCATIONS, g.allocations);
    SET(NEOCLR_METRIC_RECLAIMED_ALLOCATIONS, g.reclaimed_allocations);
    SET(NEOCLR_METRIC_RECLAIMED_BYTES, g.reclaimed_bytes);
    SET(NEOCLR_METRIC_ROOT_PUBLICATIONS, neoclr_root_probe_count_v1());
#ifdef NEOCLR_GC_PROFILE
    neoclr_gc_profile p = neoclr_gc_profile_v1();
    SET(NEOCLR_METRIC_GC_ATTEMPTS, p.attempts);
    SET(NEOCLR_METRIC_GC_NANOSECONDS, p.nanoseconds);
    SET(NEOCLR_METRIC_GC_FAILURES, p.failures);
    SET(NEOCLR_METRIC_ALLOCATED_BYTES, p.allocated_bytes);
#endif
#endif
#ifdef NEOCLR_RUNTIME_ASYNC
    neoclr_async_statistics a = neoclr_runtime_async_statistics_v1();
    SET(NEOCLR_METRIC_ASYNC_FIBERS, a.fibers);
    SET(NEOCLR_METRIC_ASYNC_RESUMES, a.resumes);
    SET(NEOCLR_METRIC_STACK_MAPPINGS, a.mappings);
    SET(NEOCLR_METRIC_STACK_UNMAPPINGS, a.unmappings);
#endif
    return s;
}
static int valid(const neoclr_runtime_snapshot *s) {
    return s && s->version == 1 && s->size == sizeof(*s) &&
        !(s->available >> NEOCLR_METRIC_COUNT);
}
static int combine(const neoclr_runtime_snapshot *a, const neoclr_runtime_snapshot *b,
    neoclr_runtime_snapshot *out, int delta) {
    if (!out || !valid(a) || !valid(b) || a->available != b->available) return 0;
    neoclr_runtime_snapshot s = empty();
    s.available = a->available;
    for (unsigned i = 0; i < NEOCLR_METRIC_COUNT; i++) {
        if (!(s.available & (UINT64_C(1) << i))) continue;
        if (delta) {
            if (b->values[i] < a->values[i]) return 0;
            s.values[i] = b->values[i] - a->values[i];
        } else {
            if (UINT64_MAX - a->values[i] < b->values[i]) return 0;
            s.values[i] = a->values[i] + b->values[i];
        }
    }
    *out = s;
    return 1;
}
int neoclr_runtime_snapshot_delta_v1(const neoclr_runtime_snapshot *a,
    const neoclr_runtime_snapshot *b, neoclr_runtime_snapshot *out) {
    return combine(a, b, out, 1);
}
int neoclr_runtime_snapshot_merge_v1(const neoclr_runtime_snapshot *a,
    const neoclr_runtime_snapshot *b, neoclr_runtime_snapshot *out) {
    return combine(a, b, out, 0);
}
