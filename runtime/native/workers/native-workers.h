#ifndef NEOCLR_NATIVE_WORKERS_H
#define NEOCLR_NATIVE_WORKERS_H
/* Opt-in POSIX native experiment. Not a generated-code/Task/HTTP ABI.
 * All managed access is serialized through enter/leave. Outside that interval
 * callbacks may compute on copied primitives or block, but may not dereference
 * pool-managed pointers. Independent invocation arenas need their own ownership
 * and entry/cleanup qualification; they are never collected by this pool.
 * The pool roots capture/result slots through owner collection.
 * Never call pool APIs recursively while entered except capture/allocate/result.
 * Do not use ordinary TLS host-root, socket, fiber or compiler GC hooks on the
 * pool context. This API does not expose that context to callbacks.
 * Native pointers/pool state must be valid and outlive all calls; destroy requires
 * joined workers and no concurrent external caller. No automatic collection on
 * allocation pressure. Per-thread GC counters do not represent pool totals. */
#include "../gc/native-gc.h"
#include <stddef.h>
typedef struct neoclr_workers neoclr_workers;
typedef struct neoclr_worker_job neoclr_worker_job;
typedef struct { uint64_t pool, generation; unsigned slot, kind; } neoclr_worker_handle;
typedef struct { int tag; int64_t scalar; void *reference; } neoclr_worker_value;
typedef int (*neoclr_worker_callback)(neoclr_worker_job *, void *native_state);
enum { NEOCLR_WORKER_OK, NEOCLR_WORKER_CANCELLED, NEOCLR_WORKER_FAILED };
/* Caller-owned native_state must remain alive until completion/join. No managed
 * pointers in it. Worker count 1..4, capacity 1..16, consumer root capacity 32.
 * Constructor owns its arena; only the creating thread may collect/close/destroy. */
neoclr_workers *neoclr_workers_create(unsigned workers, unsigned capacity, size_t heap_bytes);
int neoclr_workers_root_new(neoclr_workers *, uint64_t initial, neoclr_worker_handle *);
int neoclr_workers_root_read(neoclr_workers *, neoclr_worker_handle, uint64_t *);
int neoclr_workers_root_release(neoclr_workers *, neoclr_worker_handle);
int neoclr_workers_submit(neoclr_workers *, neoclr_worker_handle capture,
    neoclr_worker_callback, void *, neoclr_worker_handle *ticket);
/* Cancellation is cooperative. Pending cancellation settles without invoking.
 * Running cancellation wins only before worker settlement under the gate. */
int neoclr_workers_cancel(neoclr_workers *, neoclr_worker_handle);
int neoclr_worker_cancelled(neoclr_worker_job *);
void neoclr_worker_enter(neoclr_worker_job *);
void *neoclr_worker_capture(neoclr_worker_job *);
void *neoclr_worker_allocate(neoclr_worker_job *, uint64_t bytes, uint32_t kind);
void neoclr_worker_result(neoclr_worker_job *, neoclr_worker_value);
void neoclr_worker_leave(neoclr_worker_job *);
/* Nonblocking take adopts reference into a consumer root before freeing ticket.
 * Returns 1 ready, 0 pending or consumer-root capacity, -1 invalid/stale ticket.
 * Output value.reference is deliberately cleared; use rooted handle to read.
 * Null references return an all-zero root handle. Failure/cancel have no payload. */
/* Owner-side wait releases gate; returns 1 ready, 0 timeout, -1 stale/invalid.
 * Worker callbacks cannot wait on their own pool (bounded-pool deadlock risk). */
int neoclr_workers_wait(neoclr_workers *, neoclr_worker_handle, unsigned timeout_ms);
int neoclr_workers_take(neoclr_workers *, neoclr_worker_handle, int *outcome,
    neoclr_worker_value *, neoclr_worker_handle *root);
int neoclr_workers_collect(neoclr_workers *, uint64_t *live_extent);
/* Owner-only close: stop admission, request cancellation, join outside gate.
 * Cooperative callbacks must return; no forced interruption or bounded join. */
int neoclr_workers_close(neoclr_workers *);
/* Reject outstanding tickets/roots or unjoined workers; no implicit discard. */
int neoclr_workers_destroy(neoclr_workers *);
#endif
