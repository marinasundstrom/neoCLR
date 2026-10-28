#ifndef NEOCLR_NATIVE_WORKER_QUEUES_H
#define NEOCLR_NATIVE_WORKER_QUEUES_H
#include "native-workers.h"
/* Private NEOCLR_WORKER_QUEUES build experiment. Still one metadata/heap gate,
 * 1..4 workers and <=16 jobs; does not migrate stacks or create parallel HTTP.
 * External submissions use global FIFO. Same-pool callback submissions use the
 * current worker's LIFO deque; idle workers steal oldest work. Global work gets
 * a turn after at most eight local dispatches. This is a dispatch-count bound,
 * not a wall-time/preemption guarantee. Queued cancellation unlinks immediately.
 * Scheduling order differs from the default FIFO scan. No production default
 * changes. Snapshot requires owner after close/join, with no external caller. */
typedef struct {
    uint64_t local, global, stolen, empty_steal_probes;
} neoclr_worker_queue_counts;
int neoclr_workers_queue_counts(neoclr_workers *, neoclr_worker_queue_counts *);
#endif
