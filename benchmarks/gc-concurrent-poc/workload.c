/* Same supported work/cadence for sync and concurrent POC; not an HTTP benchmark. */
#include <assert.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <sys/resource.h>
#include <stdint.h>
static uint64_t ns(clockid_t clock) {
    struct timespec t;
    if (clock_gettime(clock, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
static uint64_t phase_totals[5];
#ifdef POC_PHASE_TIMING
static int phase_enabled;
static uint64_t phase_previous;
static void phase_event(unsigned phase) {
    if (!phase_enabled) return;
    uint64_t now = ns(CLOCK_MONOTONIC);
    if (phase) phase_totals[phase - 1] += now - phase_previous;
    phase_previous = now;
}
#define RECLAIM_PHASE(phase) phase_event(phase)
#endif
#include "native-gc.c"
#include "snapshot.inc"
#include "reclaim.inc"
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
    assert(argc >= 4 && argc <= 7);
    bool concurrent = !strcmp(argv[1], "concurrent");
    assert(concurrent || !strcmp(argv[1], "sync"));
    unsigned rounds = (unsigned)strtoul(argv[2], NULL, 10);
    unsigned work = (unsigned)strtoul(argv[3], NULL, 10);
    assert(rounds >= 100 && rounds <= 10000 && work <= 1000000);
    const unsigned warmup = 100, batch = 64;
    unsigned live_count = argc >= 5 ? (unsigned)strtoul(argv[4], NULL, 10) : 512;
    assert(live_count >= 1 && live_count <= 8192);
    unsigned host_count = argc >= 6 ? (unsigned)strtoul(argv[5], NULL, 10) : 0;
    assert(host_count <= 256 && host_count <= live_count);
    unsigned churn_every = argc == 7 ? (unsigned)strtoul(argv[6], NULL, 10) : 0;
    assert(!churn_every || host_count);
    void *nodes[8192];
    uint64_t handles[256];
    unsigned char *heap = calloc(1, 512 * 1024);
    assert(heap);
    neoclr_aot_context c = {.text = {heap, 512 * 1024, 0}};
    void *root = NULL, *previous = NULL;
    for (unsigned i = 0; i < live_count; i++) {
        void *object = allocate(&c, 24, NEOCLR_GC_OBJECT);
        nodes[i] = object;
        if (previous) ((uint64_t *)previous)[1] = (uintptr_t)object;
        else root = object;
        previous = object;
    }
    for (unsigned i = 0; i < host_count; i++)
        assert(!neoclr_gc_host_root_create_v1(&c, nodes[i * live_count / host_count], &handles[i]));
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
    uint64_t discarded = 0, fallbacks = 0, replacements = 0;
    for (unsigned round = 0; round < rounds + warmup; round++) {
        if (round == warmup) {
#ifdef POC_PHASE_TIMING
            phase_enabled = 1;
#endif
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
        if (churn_every) {
            /* Rotate the rooted chain without changing its live objects. */
            void *next = (void *)(uintptr_t)((uint64_t *)root)[1];
            if (next) {
                ((uint64_t *)previous)[1] = (uintptr_t)root;
                ((uint64_t *)root)[1] = 0;
                previous = root;
                root = next;
            }
            if (round % churn_every == 0) {
                assert(!neoclr_gc_host_root_replace_v1(&c, handles[round % host_count], root));
                if (round >= warmup) replacements++;
            }
        }
        if (c.text.used > peak_extent) peak_extent = c.text.used;
        if (concurrent) {
            uint64_t before_finish = ns(CLOCK_MONOTONIC);
            cycle_result result = cycle_poll(&cycle, true, false);
            assert(result == CYCLE_APPLIED || result == CYCLE_DISCARDED);
            if (result == CYCLE_DISCARDED) {
                /* No skipped collection: charge a synchronous fallback in-window. */
                assert(!neoclr_gc_collect_v1(&c, &frame));
                if (round >= warmup) { discarded++; fallbacks++; }
            }
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
#ifdef POC_PHASE_TIMING
    phase_enabled = 0;
#endif
    assert(discarded == (concurrent ? replacements : 0));
    /* Identical post-window live set; final cleanup is outside timing. */
    assert(!neoclr_gc_collect_v1(&c, &frame));
    assert(validated_bases(&c.text, root, previous));
    void *cursor = root;
    unsigned offset = churn_every ? (rounds + warmup) % live_count : 0;
    for (unsigned i = 0; i < live_count; i++) {
        assert(cursor == nodes[(offset + i) % live_count]);
        cursor = (void *)(uintptr_t)((uint64_t *)cursor)[1];
    }
    assert(!cursor && fallbacks == discarded);
    for (unsigned i = 0; i < host_count; i++)
        assert(!neoclr_gc_host_root_release_v1(&c, handles[i]));
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
    printf("{\"churnEvery\":%u,\"replacements\":%llu,\"discardedCycles\":%llu,\"fallbackCollections\":%llu,",
        churn_every, (unsigned long long)replacements, (unsigned long long)discarded,
        (unsigned long long)fallbacks);
    printf("\"phaseTiming\":%s,\"phaseNs\":[%llu,%llu,%llu,%llu,%llu],",
#ifdef POC_PHASE_TIMING
        "true",
#else
        "false",
#endif
        (unsigned long long)phase_totals[0], (unsigned long long)phase_totals[1],
        (unsigned long long)phase_totals[2], (unsigned long long)phase_totals[3],
        (unsigned long long)phase_totals[4]);
    printf("\"mode\":\"%s\",\"rounds\":%u,\"work\":%u,\"liveCount\":%u,\"hostRoots\":%u,\"elapsedNs\":%llu,\"cpuNs\":%llu,\"ownerCpuNs\":%llu,\"pauseTotalNs\":%llu,\"captureOrSyncNs\":%llu,\"waitApplyNs\":%llu,\"batchP99Ns\":%llu,\"pauseP99Ns\":%llu,\"peakArenaBytes\":%llu,\"maxRssBytes\":%ld,\"allocations\":%llu,\"collections\":%llu,\"checksum\":%llu,\"finalArenaBytes\":0}\n",
        argv[1], rounds, work, live_count, host_count, (unsigned long long)elapsed, (unsigned long long)cpu,
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
