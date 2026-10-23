#define _DARWIN_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#include "native-workers.h"
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d: %s\n", __LINE__, #x); abort(); } } while (0)
extern int32_t neoclr_bootstrap_callback_v1(int32_t, uint64_t *, neoclr_aot_context *);
typedef struct { int parallel; uint64_t allocations, bytes, collections, gc_ns; } invocation;
static int work(neoclr_worker_job *job, void *state) {
    invocation *s = state;
    neoclr_gc_statistics before = neoclr_gc_statistics_v1();
    neoclr_gc_profile profile_before = neoclr_gc_profile_v1();
    int result = s->parallel ? neoclr_worker_invoke_capture_parallel(job) : neoclr_worker_invoke_capture(job);
    neoclr_gc_statistics after = neoclr_gc_statistics_v1();
    s->allocations = after.allocations - before.allocations;
    neoclr_gc_profile profile_after = neoclr_gc_profile_v1();
    s->bytes = profile_after.allocated_bytes - profile_before.allocated_bytes;
    s->gc_ns = profile_after.nanoseconds - profile_before.nanoseconds;
    s->collections = after.collections - before.collections;
    return result;
}
static double monotonic(void) {
    struct timespec t; CHECK(!clock_gettime(CLOCK_MONOTONIC, &t));
    return t.tv_sec + t.tv_nsec / 1e9;
}
static double cpu(struct rusage r) {
    return r.ru_utime.tv_sec + r.ru_stime.tv_sec + (r.ru_utime.tv_usec + r.ru_stime.tv_usec) / 1e6;
}
int main(int argc, char **argv) {
    CHECK(argc == 3);
    unsigned workers = (unsigned)atoi(argv[1]);
    int parallel = atoi(argv[2]);
    CHECK(workers >= 1 && workers <= 4 && (parallel == 0 || parallel == 1));
    neoclr_workers *p = neoclr_workers_create(workers, 16, 32768);
    neoclr_worker_handle root;
    CHECK(p && !neoclr_workers_bootstrap(p, neoclr_bootstrap_callback_v1, neoclr_invoke_void_callback_v1, 40, &root));
    uint64_t allocations = 0, bytes = 0, collections = 0, gc_ns = 0;
    struct rusage before, after;
    double start = 0;
    /* One warmup batch, then 4096 identical jobs. Fixed queue/heap/work across owner counts. */
    for (unsigned batch = 0; batch < 257; batch++) {
        if (batch == 1) { CHECK(!getrusage(RUSAGE_SELF, &before)); start = monotonic(); }
        neoclr_worker_handle tickets[16];
        invocation states[16] = {0};
        for (unsigned i = 0; i < 16; i++) {
            states[i].parallel = parallel;
            CHECK(neoclr_workers_submit(p, root, work, &states[i], &tickets[i]));
        }
        for (unsigned i = 0; i < 16; i++) {
            neoclr_worker_value value; neoclr_worker_handle ignored; int outcome;
            CHECK(neoclr_workers_wait(p, tickets[i], 30000) == 1);
            CHECK(neoclr_workers_take(p, tickets[i], &outcome, &value, &ignored) == 1);
            CHECK(outcome == NEOCLR_WORKER_OK && !ignored.pool);
            if (batch) {
                allocations += states[i].allocations;
                bytes += states[i].bytes;
                collections += states[i].collections;
                gc_ns += states[i].gc_ns;
            }
        }
    }
    double wall = monotonic() - start; CHECK(!getrusage(RUSAGE_SELF, &after));
    unsigned peak; uint64_t coordinated, extent;
    CHECK(neoclr_workers_compiled_statistics(p, &peak, &coordinated));
    CHECK(neoclr_workers_root_release(p, root));
    CHECK(neoclr_workers_collect(p, &extent) && !extent);
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    printf("{\"workers\":%u,\"parallel\":%d,\"jobs\":4096,\"wallSeconds\":%.6f,\"cpuSeconds\":%.6f,"
           "\"allocations\":%llu,\"payloadBytes\":%llu,\"collections\":%llu,\"gcNanoseconds\":%llu,\"peakParticipants\":%u,"
           "\"voluntarySwitches\":%ld,\"involuntarySwitches\":%ld}\n",
           workers, parallel, wall, cpu(after)-cpu(before),
           (unsigned long long)allocations, (unsigned long long)bytes, (unsigned long long)collections, (unsigned long long)gc_ns, peak,
           after.ru_nvcsw-before.ru_nvcsw, after.ru_nivcsw-before.ru_nivcsw);
}
