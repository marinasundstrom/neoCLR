#ifndef NEOCLR_NATIVE_WORKER_TRACE_H
#define NEOCLR_NATIVE_WORKER_TRACE_H
#include "native-workers.h"
/* Private POSIX experiment, not a managed API. Functions require a runtime built
 * with NEOCLR_WORKER_TRACE; ordinary builds omit recorder/storage/hooks entirely. Configure before worker creation.
 * Zero disables recording; otherwise 1..65536 records per lane. Lane 0 contains
 * external callers serialized by the existing pool gate; lanes 1..N belong to
 * workers (logical IDs, not OS thread IDs). No live reads or draining.
 * Snapshot requires the owner after close, with no concurrent external callers.
 * Borrowed records remain valid until destroy. No managed pointers are recorded.
 * Buffers retain the first records and drop subsequent records, never wrap.
 * No additional locks, allocation or I/O in emission; timestamps use MONOTONIC.
 * Pool hooks are not signal handlers and must not be reentered from a signal. */
enum neoclr_worker_trace_kind {
    NEOCLR_TRACE_START = 1, NEOCLR_TRACE_STOP, NEOCLR_TRACE_IDLE_BEGIN,
    NEOCLR_TRACE_IDLE_END, NEOCLR_TRACE_ENQUEUE, NEOCLR_TRACE_RUN,
    NEOCLR_TRACE_RETURN, NEOCLR_TRACE_SETTLE, NEOCLR_TRACE_CANCEL
};
typedef struct {
    uint64_t ns, sequence, generation;
    uint32_t kind, slot, detail;
} neoclr_worker_trace_event;
typedef struct {
    const neoclr_worker_trace_event *events;
    uint64_t pool, origin_ns, attempted, dropped;
    size_t count, capacity;
    unsigned workers;
} neoclr_worker_trace_view;
neoclr_workers *neoclr_workers_create_traced(unsigned workers, unsigned capacity,
    size_t heap_bytes, size_t events_per_lane);
int neoclr_workers_trace_view(neoclr_workers *, unsigned lane, neoclr_worker_trace_view *);
#endif
