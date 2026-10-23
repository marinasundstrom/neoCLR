#ifndef NEOCLR_RUNTIME_SNAPSHOT_H
#define NEOCLR_RUNTIME_SNAPSHOT_H
#include <stdint.h>
/* Private native diagnostics, not a managed/public ABI. Owner-thread cumulative
 * counters only: not process totals or live heap sizes. Version and byte size
 * are checked before arithmetic. Availability distinguishes unsupported from zero.
 * Capture on the owning thread; publish copies with normal synchronization.
 * No allocation, clocks, I/O or cross-thread reads. Sources must use matching flags.
 * Delta requires the same owner and no intervening counter reset/wrap. Merge is
 * only for disjoint intervals/owners; shared heaps must not be counted per worker.
 * Invalid/overflowing arithmetic returns 0 with output unchanged. No saturation. */
enum {
    NEOCLR_METRIC_COLLECTIONS, NEOCLR_METRIC_ALLOCATIONS,
    NEOCLR_METRIC_RECLAIMED_ALLOCATIONS, NEOCLR_METRIC_RECLAIMED_BYTES,
    NEOCLR_METRIC_ROOT_PUBLICATIONS, NEOCLR_METRIC_GC_ATTEMPTS,
    NEOCLR_METRIC_GC_NANOSECONDS, NEOCLR_METRIC_GC_FAILURES,
    NEOCLR_METRIC_ALLOCATED_BYTES, NEOCLR_METRIC_ASYNC_FIBERS,
    NEOCLR_METRIC_ASYNC_RESUMES, NEOCLR_METRIC_STACK_MAPPINGS,
    NEOCLR_METRIC_STACK_UNMAPPINGS, NEOCLR_METRIC_COUNT
};
typedef struct {
    uint32_t version, size;
    uint64_t available;
    uint64_t values[NEOCLR_METRIC_COUNT];
} neoclr_runtime_snapshot;
neoclr_runtime_snapshot neoclr_runtime_snapshot_v1(void);
int neoclr_runtime_snapshot_delta_v1(const neoclr_runtime_snapshot *before,
    const neoclr_runtime_snapshot *after, neoclr_runtime_snapshot *output);
int neoclr_runtime_snapshot_merge_v1(const neoclr_runtime_snapshot *left,
    const neoclr_runtime_snapshot *right, neoclr_runtime_snapshot *output);
#endif
