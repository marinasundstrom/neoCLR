#include "snapshot.h"
#include "native-gc.h"
#include "native-runtime.h"
#include <assert.h>
#include <pthread.h>
#include <string.h>
static _Thread_local uint64_t n;
neoclr_gc_statistics neoclr_gc_statistics_v1(void) {
    return (neoclr_gc_statistics){n, n*2, n*3, n*4};
}
uint64_t neoclr_root_probe_count_v1(void) { return n*5; }
neoclr_gc_profile neoclr_gc_profile_v1(void) {
    return (neoclr_gc_profile){.attempts=n*6, .nanoseconds=n*7, .failures=n*8, .allocated_bytes=n*9};
}
neoclr_async_statistics neoclr_runtime_async_statistics_v1(void) {
    return (neoclr_async_statistics){.fibers=n*10, .resumes=n*11, .mappings=n*12, .unmappings=n*13};
}
static void *worker(void *state) {
    neoclr_runtime_snapshot *s = state;
    assert(!n);
    n = 10;
    neoclr_runtime_snapshot before = neoclr_runtime_snapshot_v1();
    n = 13;
    neoclr_runtime_snapshot after = neoclr_runtime_snapshot_v1();
    assert(neoclr_runtime_snapshot_delta_v1(&before, &after, s));
    return NULL;
}
int main(void) {
    uint64_t mask = 0;
#ifdef NEOCLR_NATIVE_GC
    mask |= 31;
#ifdef NEOCLR_GC_PROFILE
    mask |= 480;
#endif
#endif
#ifdef NEOCLR_RUNTIME_ASYNC
    mask |= 7680;
#endif
    n = 17;
    neoclr_runtime_snapshot baseline = neoclr_runtime_snapshot_v1();
    assert(baseline.version == 1 && baseline.size == sizeof(baseline) && baseline.available == mask);
    for (unsigned i=0; i<NEOCLR_METRIC_COUNT; i++)
        assert(baseline.values[i] == ((mask >> i & 1) ? 17*(i+1) : 0));
    pthread_t threads[4]; neoclr_runtime_snapshot snapshots[4];
    for (unsigned i=0; i<4; i++) assert(!pthread_create(&threads[i], NULL, worker, &snapshots[i]));
    for (unsigned i=0; i<4; i++) assert(!pthread_join(threads[i], NULL));
    neoclr_runtime_snapshot total = snapshots[0];
    for (unsigned i=1; i<4; i++) assert(neoclr_runtime_snapshot_merge_v1(&total, &snapshots[i], &total));
    assert(n == 17);
    for (unsigned i=0; i<NEOCLR_METRIC_COUNT; i++)
        assert(total.values[i] == ((mask >> i & 1) ? 12*(i+1) : 0));
    neoclr_runtime_snapshot output = baseline, bad = baseline;
#define REJECT(expr) do { assert(!(expr)); assert(!memcmp(&output, &baseline, sizeof(output))); } while (0)
    bad.version++;
    REJECT(neoclr_runtime_snapshot_merge_v1(&bad, &baseline, &output));
    bad=baseline; bad.size--;
    REJECT(neoclr_runtime_snapshot_delta_v1(&bad, &baseline, &output));
    bad=baseline; bad.available |= UINT64_C(1)<<63;
    REJECT(neoclr_runtime_snapshot_merge_v1(&bad, &baseline, &output));
    REJECT(neoclr_runtime_snapshot_delta_v1(NULL, &baseline, &output));
    assert(!neoclr_runtime_snapshot_merge_v1(&baseline, &baseline, NULL));
    bad=baseline; bad.available ^= 1;
    REJECT(neoclr_runtime_snapshot_merge_v1(&bad, &baseline, &output));
    bad=baseline; bad.available=1; bad.values[0]=UINT64_MAX;
    neoclr_runtime_snapshot one=bad; one.values[0]=1;
    REJECT(neoclr_runtime_snapshot_merge_v1(&bad, &one, &output));
    REJECT(neoclr_runtime_snapshot_delta_v1(&bad, &one, &output));
    assert(neoclr_runtime_snapshot_delta_v1(&bad, &bad, &bad));
    assert(bad.values[0]==0);
    return 0;
}
