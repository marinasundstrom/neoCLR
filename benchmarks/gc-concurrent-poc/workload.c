/* Same supported work/cadence for sync and concurrent POC; not an HTTP benchmark. */
#include <assert.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <sys/resource.h>
#include "native-gc.c"
#include "snapshot.inc"
#include "reclaim.inc"
static uint64_t ns(clockid_t clock) {
    struct timespec t;
    if (clock_gettime(clock, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
static void *allocate(neoclr_aot_context *c, uint64_t bytes, uint32_t kind) {
    void *p;
    assert(!neoclr_gc_allocate_v1(&c->text, bytes, kind, &p));
    return p;
}
static int compare(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}
static uint64_t percentile(uint64_t *samples, unsigned count, unsigned percent) {
    qsort(samples, count, sizeof(*samples), compare);
    return samples[((uint64_t)count * percent + 99) / 100 - 1];
}
int main(int argc, char **argv) {
    assert(argc == 4);
    bool concurrent = !strcmp(argv[1], "concurrent");
    assert(concurrent || !strcmp(argv[1], "sync"));
    unsigned rounds = (unsigned)strtoul(argv[2], NULL, 10);
    unsigned work = (unsigned)strtoul(argv[3], NULL, 10);
    assert(rounds >= 100 && rounds <= 10000 && work <= 1000000);
    const unsigned warmup = 100, live_count = 512, batch = 64;
    unsigned char *heap = calloc(1, 512 * 1024);
    assert(heap);
    neoclr_aot_context c = {.text = {heap, 512 * 1024, 0}};
    void *root = NULL, *previous = NULL;
    for (unsigned i = 0; i < live_count; i++) {
        void *object = allocate(&c, 24, NEOCLR_GC_OBJECT);
        if (previous) ((uint64_t *)previous)[1] = (uintptr_t)object;
        else root = object;
        previous = object;
    }
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
    collection_cycle cycle;
    if (concurrent) assert(cycle_init(&cycle));
    uint64_t *latency = calloc(rounds, sizeof(*latency));
    uint64_t *pauses = calloc(rounds * 2, sizeof(*pauses));
    assert(latency && pauses);
    uint64_t start = 0, cpu_start = 0, owner_start = 0, gc_before = 0, allocations_before = 0;
    uint64_t checksum = 1, pause_sum = 0, capture_sum = 0, finish_sum = 0, peak_extent = 0;
    unsigned pause_count = 0;
    for (unsigned round = 0; round < rounds + warmup; round++) {
        if (round == warmup) {
            start = ns(CLOCK_MONOTONIC);
            cpu_start = ns(CLOCK_PROCESS_CPUTIME_ID);
            owner_start = ns(CLOCK_THREAD_CPUTIME_ID);
            gc_before = statistics.collections;
            allocations_before = statistics.allocations;
        }
        uint64_t begin = ns(CLOCK_MONOTONIC), after_gc;
        uint64_t capture_time = 0, finish_time = 0;
        if (concurrent) {
            assert(cycle_start(&cycle, &c));
            atomic_store(&cycle.job->proceed, true);
        } else assert(!neoclr_gc_collect_v1(&c, &frame));
        after_gc = ns(CLOCK_MONOTONIC);
        capture_time = after_gc - begin;
        for (unsigned i = 0; i < batch; i++) {
            uint64_t *garbage = allocate(&c, 128, NEOCLR_GC_BYTES);
            *garbage = checksum + i;
        }
        /* Reproducible CPU work independent of GC decisions, with observable
         * output. References remain in the stable, rooted live chain. */
        for (unsigned i = 0; i < work; i++) checksum = checksum * UINT64_C(6364136223846793005) + 1;
        ((uint64_t *)root)[2] = checksum & UINT64_C(0xffffffff);
        if (c.text.used > peak_extent) peak_extent = c.text.used;
        if (concurrent) {
            uint64_t before_finish = ns(CLOCK_MONOTONIC);
            assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
            finish_time = ns(CLOCK_MONOTONIC) - before_finish;
        }
        if (round >= warmup) {
            latency[round - warmup] = ns(CLOCK_MONOTONIC) - begin;
            pauses[pause_count++] = capture_time;
            if (concurrent) pauses[pause_count++] = finish_time;
            capture_sum += capture_time;
            finish_sum += finish_time;
            pause_sum += capture_time + finish_time;
        }
    }
    uint64_t elapsed = ns(CLOCK_MONOTONIC) - start;
    uint64_t cpu = ns(CLOCK_PROCESS_CPUTIME_ID) - cpu_start;
    uint64_t owner_cpu = ns(CLOCK_THREAD_CPUTIME_ID) - owner_start;
    uint64_t collections = statistics.collections - gc_before;
    uint64_t allocations = statistics.allocations - allocations_before;
    assert(collections == rounds && allocations == (uint64_t)rounds * batch);
    /* Identical post-window live set; final cleanup is outside timing. */
    assert(!neoclr_gc_collect_v1(&c, &frame));
    assert(validated_bases(&c.text, root, previous));
    root = NULL;
    if (concurrent) {
        assert(cycle_start(&cycle, &c));
        atomic_store(&cycle.job->proceed, true);
        assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
        cycle_destroy(&cycle);
    } else assert(!neoclr_gc_collect_v1(&c, &frame));
    assert(!c.text.used && statistics.allocations == statistics.reclaimed_allocations);
    struct rusage usage;
    assert(!getrusage(RUSAGE_SELF, &usage));
    printf("{\"mode\":\"%s\",\"rounds\":%u,\"work\":%u,\"elapsedNs\":%llu,\"cpuNs\":%llu,\"ownerCpuNs\":%llu,\"pauseTotalNs\":%llu,\"captureOrSyncNs\":%llu,\"waitApplyNs\":%llu,\"batchP99Ns\":%llu,\"pauseP99Ns\":%llu,\"peakArenaBytes\":%llu,\"maxRssBytes\":%ld,\"allocations\":%llu,\"collections\":%llu,\"checksum\":%llu,\"finalArenaBytes\":0}\n",
        argv[1], rounds, work, (unsigned long long)elapsed, (unsigned long long)cpu,
        (unsigned long long)owner_cpu, (unsigned long long)pause_sum,
        (unsigned long long)capture_sum, (unsigned long long)finish_sum,
        (unsigned long long)percentile(latency, rounds, 99),
        (unsigned long long)percentile(pauses, pause_count, 99),
        (unsigned long long)peak_extent, usage.ru_maxrss,
        (unsigned long long)allocations, (unsigned long long)collections,
        (unsigned long long)checksum);
    neoclr_probe_leave_v1(&frame);
    free(latency); free(pauses); free(heap);
}
