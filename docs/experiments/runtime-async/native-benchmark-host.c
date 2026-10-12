/* Private matched JIT/AOT benchmark; each sample owns a fresh heap and scopes. */
#define neoclr_jit_run_v1 neoclr_benchmark_run_once
#include "../jit-managed/host.c"
#undef neoclr_jit_run_v1
#include <inttypes.h>
#include <time.h>

#ifndef NEOCLR_BENCH_REPETITIONS
#define NEOCLR_BENCH_REPETITIONS 4
#endif

static double seconds(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC, &value)) abort();
    return (double)value.tv_sec + (double)value.tv_nsec / 1e9;
}

int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args,
                         int32_t argc, const entry_char **argv) {
    for (int i = 0; i < NEOCLR_BENCH_REPETITIONS; ++i) {
        neoclr_gc_statistics before = neoclr_gc_statistics_v1();
        neoclr_async_statistics stacks_before = neoclr_runtime_async_statistics_v1();
        neoclr_gc_profile_reset_v1();
        neoclr_gc_profile gc_before = neoclr_gc_profile_v1();
        double start = seconds();
        int32_t status = neoclr_benchmark_run_once(entry, with_args, argc, argv);
        double elapsed = seconds() - start;
        neoclr_gc_profile gc_after = neoclr_gc_profile_v1();
        neoclr_gc_statistics after = neoclr_gc_statistics_v1();
        if (status) return status;
        neoclr_async_statistics stacks = neoclr_runtime_async_statistics_v1();
        if (stacks.reserved_bytes) abort();
        uint64_t allocated = after.allocations - before.allocations;
        uint64_t reclaimed = after.reclaimed_allocations - before.reclaimed_allocations;
        if (allocated != reclaimed) abort();
#ifdef NEOCLR_GC_PROFILE
        if (gc_after.failures || gc_after.live_bytes || gc_after.live_blocks ||
            gc_after.allocated_bytes != after.reclaimed_bytes - before.reclaimed_bytes) abort();
#endif
        fprintf(stderr, "NATIVE_SAMPLE {\"seconds\":%.9f,\"allocatedBlocks\":%" PRIu64
                ",\"reclaimedBlocks\":%" PRIu64 ",\"collections\":%" PRIu64
                ",\"fibers\":%" PRIu64 ",\"resumes\":%" PRIu64
                ",\"stackMappings\":%" PRIu64 ",\"stackUnmappings\":%" PRIu64
                ",\"peakReservedStackBytes\":%" PRIu64 ",\"reservedStackBytes\":%" PRIu64
                ",\"gcNanoseconds\":%" PRIu64 ",\"gcAttempts\":%" PRIu64,
                elapsed, allocated, reclaimed, after.collections - before.collections,
                stacks.fibers - stacks_before.fibers, stacks.resumes - stacks_before.resumes,
                stacks.mappings - stacks_before.mappings, stacks.unmappings - stacks_before.unmappings,
                stacks.peak_reserved_bytes, stacks.reserved_bytes,
                gc_after.nanoseconds - gc_before.nanoseconds, gc_after.attempts - gc_before.attempts);
#ifdef NEOCLR_GC_PROFILE
        fprintf(stderr, ",\"gcFailures\":%" PRIu64 ",\"gcMaximumNanoseconds\":%" PRIu64
                ",\"allocatedPayloadBytes\":%" PRIu64 ",\"reclaimedPayloadBytes\":%" PRIu64
                ",\"peakArenaUsedBytes\":%" PRIu64 ",\"livePayloadBytesAfterCollection\":%" PRIu64
                ",\"liveBlocksAfterCollection\":%" PRIu64 ",\"peakLivePayloadBytesAfterCollection\":%" PRIu64
                ",\"gcPauseBuckets\":[",
                gc_after.failures, gc_after.maximum_nanoseconds, gc_after.allocated_bytes,
                after.reclaimed_bytes - before.reclaimed_bytes, gc_after.peak_arena_used,
                gc_after.live_bytes, gc_after.live_blocks, gc_after.peak_live_bytes);
        for (unsigned bucket = 0; bucket < NEOCLR_GC_PAUSE_BUCKETS; bucket++)
            fprintf(stderr, "%s%" PRIu64, bucket ? "," : "", gc_after.pause_buckets[bucket]);
        fputc(']', stderr);
#endif
        fputs("}\n", stderr);
    }
    return 0;
}

#ifdef NEOCLR_BENCH_AOT
extern int32_t neoclr_entry_v4(int32_t, int32_t *, neoclr_aot_context *);
int main(int argc, const char **argv) {
    entry_v4 function = neoclr_entry_v4;
    const void *entry;
    _Static_assert(sizeof(entry) == sizeof(function), "entry pointer size");
    memcpy(&entry, &function, sizeof(entry));
    return neoclr_jit_run_v1(entry, 0, argc, argv);
}
#endif
