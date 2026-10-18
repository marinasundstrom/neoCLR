/* Generated callback benchmark; identical host batch pacing in both modes. */
#include <assert.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <sys/resource.h>
#include "native-gc.c"
#include "snapshot.inc"
#include "reclaim.inc"
#include "native-session.h"
static bool batch_active;
static uint64_t deferred;
int32_t experiment_gc_boundary(neoclr_aot_context *c, const neoclr_probe_frame *boundary) {
    if (!c || !boundary || boundary != neoclr_root_probe_head_v1() || boundary->context != c) return 3;
    if (batch_active) { deferred++; return 0; }
    return neoclr_gc_collect_v1(c, boundary);
}
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

#ifndef LOAD_ARRIVAL
#define LOAD_ARRIVAL(index, start) ((void)0)
#define LOAD_COMPLETE(index, start, end) ((void)0)
#define LOAD_REPORT() ((void)0)
#endif
static int compare(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}
int main(int argc, char **argv) {
    assert(argc == 6);
    bool concurrent = !strcmp(argv[1], "concurrent");
    assert(concurrent || !strcmp(argv[1], "sync"));
    unsigned rounds = (unsigned)strtoul(argv[2], NULL, 10);
    unsigned work = (unsigned)strtoul(argv[3], NULL, 10);
    unsigned length = (unsigned)strtoul(argv[4], NULL, 10);
    unsigned replace_count = (unsigned)strtoul(argv[5], NULL, 10);
    assert(rounds >= 100 && rounds <= 100000 && work <= 1000000);
    assert(length >= 1 && length <= 8192 && replace_count >= 1 && replace_count <= 4096);
    assert((length + 2 * replace_count + 8) * 56 <= 512 * 1024);
    unsigned char *heap = calloc(1, 512 * 1024);
    assert(heap);
    neoclr_aot_context c = {.text = {heap, 512 * 1024, 0}};
    uint64_t handle;
    assert(!neoclr_bootstrap_callback_v1(0, &handle, &c));
    void *descriptor;
    assert(!neoclr_gc_host_root_read_v1(&c, handle, &descriptor));
    uint64_t *counter = (void *)(uintptr_t)((uint64_t *)descriptor)[2];
    uint64_t *chain = NULL;
    for (unsigned i = 0; i < length; i++) {
        uint64_t *node = allocate(&c, 16, NEOCLR_GC_OBJECT);
        node[1] = (uintptr_t)chain;
        chain = node;
    }
    uint64_t chain_handle;
    assert(!neoclr_gc_host_root_create_v1(&c, chain, &chain_handle));
    collection_cycle cycle;
    if (concurrent) assert(cycle_init(&cycle));
    const unsigned warmup = 100;
    uint64_t *latencies = calloc(rounds, sizeof(*latencies));
    assert(latencies);
    uint64_t start = 0, cpu_start = 0, owner_start = 0, collections_before = 0, allocations_before = 0;
    uint64_t invoked = 0, checksum = 1, capture = 0, finish = 0, peak = 0;
    for (unsigned round = 0; round < warmup + rounds; round++) {
        if (round == warmup) {
            start = ns(CLOCK_MONOTONIC);
            cpu_start = ns(CLOCK_PROCESS_CPUTIME_ID);
            owner_start = ns(CLOCK_THREAD_CPUTIME_ID);
            collections_before = statistics.collections;
            allocations_before = statistics.allocations;
        }
        if (round >= warmup) LOAD_ARRIVAL(round - warmup, start);
        uint64_t begin = ns(CLOCK_MONOTONIC);
        if (concurrent) {
            assert(cycle_start(&cycle, &c));
            atomic_store(&cycle.job->proceed, true);
        } else assert(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t after_capture = ns(CLOCK_MONOTONIC);
        batch_active = true;
        for (unsigned i = 0; i < replace_count; i++) {
            assert(!neoclr_invoke_void_callback_v1(handle, &c));
            invoked++;
            assert(*(int32_t *)(counter + 1) == (int32_t)invoked);
            checksum = checksum * UINT64_C(6364136223846793005) + invoked;
        }
        batch_active = false;
        for (unsigned i = 0; i < work; i++) checksum = checksum * UINT64_C(6364136223846793005) + 1;
        if (c.text.used > peak) peak = c.text.used;
        assert(c.text.used <= (length + 2 * replace_count + 8) * 56);
        uint64_t before_finish = ns(CLOCK_MONOTONIC);
        if (concurrent) assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
        uint64_t end = ns(CLOCK_MONOTONIC);
        if (round >= warmup) {
            capture += after_capture - begin;
            if (concurrent) finish += end - before_finish;
            latencies[round - warmup] = end - begin;
            LOAD_COMPLETE(round - warmup, start, end);
        }
    }
    uint64_t elapsed = ns(CLOCK_MONOTONIC) - start;
    uint64_t cpu = ns(CLOCK_PROCESS_CPUTIME_ID) - cpu_start;
    uint64_t owner_cpu = ns(CLOCK_THREAD_CPUTIME_ID) - owner_start;
    uint64_t collections = statistics.collections - collections_before;
    uint64_t allocations = statistics.allocations - allocations_before;
    assert(collections == rounds && allocations == (uint64_t)rounds * replace_count);
    assert(invoked == (uint64_t)(rounds + warmup) * replace_count && deferred == invoked);
    assert(!neoclr_gc_collect_v1(&c, NULL));
    uint64_t *child = (void *)(uintptr_t)counter[2];
    assert(validated_bases(&c.text, counter, child));
    assert(*(int32_t *)(child + 1) == (int32_t)invoked);
    assert(statistics.allocations - statistics.reclaimed_allocations == length + 3);
    assert(!neoclr_gc_host_root_release_v1(&c, handle));
    assert(!neoclr_gc_host_root_release_v1(&c, chain_handle));
    assert(!neoclr_gc_complete_invocation_v1(&c));
    if (concurrent) {
        assert(cycle_start(&cycle, &c));
        atomic_store(&cycle.job->proceed, true);
        assert(cycle_poll(&cycle, true, false) == CYCLE_APPLIED);
        cycle_destroy(&cycle);
    } else assert(!neoclr_gc_collect_v1(&c, NULL));
    assert(!c.text.used && statistics.allocations == statistics.reclaimed_allocations);
    qsort(latencies, rounds, sizeof(*latencies), compare);
    struct rusage usage;
    assert(!getrusage(RUSAGE_SELF, &usage));
    printf("{\"mode\":\"%s\",\"rounds\":%u,\"work\":%u,\"liveCount\":%u,\"callbacksPerBatch\":%u,\"elapsedNs\":%llu,\"cpuNs\":%llu,\"ownerCpuNs\":%llu,\"captureOrSyncNs\":%llu,\"waitApplyNs\":%llu,\"pauseTotalNs\":%llu,\"batchP99Ns\":%llu,\"peakArenaBytes\":%llu,\"maxRssBytes\":%ld,\"allocations\":%llu,\"collections\":%llu,\"checksum\":%llu,\"finalArenaBytes\":0}\n",
        argv[1], rounds, work, length, replace_count, (unsigned long long)elapsed,
        (unsigned long long)cpu, (unsigned long long)owner_cpu, (unsigned long long)capture,
        (unsigned long long)finish, (unsigned long long)(capture + finish),
        (unsigned long long)latencies[((uint64_t)rounds * 99 + 99) / 100 - 1],
        (unsigned long long)peak, usage.ru_maxrss, (unsigned long long)allocations,
        (unsigned long long)collections, (unsigned long long)checksum);
    LOAD_REPORT();
    free(latencies);
    free(heap);
}
