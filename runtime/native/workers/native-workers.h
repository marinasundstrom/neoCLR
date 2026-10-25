#ifndef NEOCLR_NATIVE_WORKERS_H
#define NEOCLR_NATIVE_WORKERS_H
/* Opt-in POSIX native experiment. Not a general Task/HTTP ABI.
 * Generated execution and allocation are serialized through enter/leave; explicit
 * native read or synchronized mutation phases may overlap within their mode.
 * Outside these intervals callbacks may compute on copied primitives or block, but may not dereference
 * pool-managed pointers. Independent invocation arenas need their own ownership
 * and entry/cleanup qualification; they are never collected by this pool.
 * The pool roots capture/result slots through owner collection.
 * Never call pool APIs recursively while entered except capture/allocate/result/local-root operations.
 * Do not use ordinary TLS host-root, socket, fiber or compiler GC hooks on the
 * pool context except through the synchronous generated bridge below.
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
/* Private synchronous generated-callback bridge. Bootstrap once on an empty
 * owner pool; both exports must belong to the same compiled image. No async,
 * resources, retained TLS roots or nested pool calls. Invoke owns the heap gate
 * for its full duration, transfers the arena/static root to a fresh invocation
 * context and returns them before releasing the gate; a guest fault belongs to this job and prevents another
 * invocation in the same job. Other jobs retain the shared heap and may proceed.
 * Retained TLS resources/roots or unbalanced frames violate the bridge contract
 * and fail fast. Return native status (0 success); bootstrap output changes only
 * on success. Cancellation is observed at job settlement, not during invocation. */
typedef int32_t (*neoclr_worker_bootstrap)(int32_t, uint64_t *, neoclr_aot_context *);
typedef int32_t (*neoclr_worker_invoke)(uint64_t, neoclr_aot_context *);
int32_t neoclr_workers_bootstrap(neoclr_workers *, neoclr_worker_bootstrap,
    neoclr_worker_invoke, int32_t argument, neoclr_worker_handle *);
int32_t neoclr_worker_invoke_capture(neoclr_worker_job *);
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
/* Private native read phase: returns the current rooted capture, allowing only
 * reads of its reachable graph until read_end on the same callback/thread.
 * Multiple readers may overlap. Mutation, allocation and collection wait for all
 * readers; a waiting writer prevents fresh read admission. Do not retain pointers
 * after read_end, write object bytes, invoke generated code/GC/TLS hooks, or call
 * any pool API within this interval except read_end. No nested phases. Never wait
 * for pool operations or another reader to enter while holding a read phase:
 * pending writers can prevent that admission. Callback must end its phase before
 * returning, even when cancelled. Close waits cooperatively; no forced revocation.
 * This is not a compiler safepoint or support for concurrent managed mutation. */
const void *neoclr_worker_read_begin(neoclr_worker_job *);
void neoclr_worker_read_end(neoclr_worker_job *);
/* Private cooperative native mutators. Other mutators may overlap; callers must
 * synchronize conflicting fields/edges (e.g. a native mutex), or own disjoint
 * fields. Plain native readers are excluded. GC, allocation and generated entry
 * wait for every mutator to exit/poll. No implicit atomicity or thread safety.
 * begin/poll returns the rooted capture. Before poll/end, publish live detached
 * references using local_store; after begin/poll reload locals through local_load.
 * Only local_store/load and mutate_end/poll are allowed pool calls inside a phase.
 * No allocation, generated callbacks or TLS roots/resources. Never wait for pool
 * operations, another phase admission, or hold application locks across begin/poll/end.
 * Poll temporarily leaves the phase and lets pending exclusive work run; object
 * edges may change, so reload/revalidate them with application synchronization.
 * Cancellation does not revoke access. End on the originating callback/thread
 * before returning. This is a native safepoint prototype, not a compiler ABI. */
void *neoclr_worker_mutate_begin(neoclr_worker_job *);
void *neoclr_worker_mutate_poll(neoclr_worker_job *);
void neoclr_worker_mutate_end(neoclr_worker_job *);
void neoclr_worker_enter(neoclr_worker_job *);
void *neoclr_worker_capture(neoclr_worker_job *);
void *neoclr_worker_allocate(neoclr_worker_job *, uint64_t bytes, uint32_t kind);
void neoclr_worker_result(neoclr_worker_job *, neoclr_worker_value);
/* Private participant roots: eight fixed slots per running job. Calls require
 * enter/leave or a native mutation phase. Store copies an exact pool allocation
 * base (or NULL to release);
 * invalid slot/value returns 0 without changing the slot. Load returns NULL for
 * an invalid/empty slot. Roots survive leaving the gate and owner collection,
 * and are cleared on all settlement paths. No stack addresses are retained.
 * Publish every live temporary before leaving; reload only after re-entering.
 * These explicit slots are not compiler stack registration or a safepoint ABI. */
#define NEOCLR_WORKER_LOCAL_ROOTS 8
int neoclr_worker_local_store(neoclr_worker_job *, unsigned slot, void *value);
void *neoclr_worker_local_load(neoclr_worker_job *, unsigned slot);
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
/* Owner-side bounded completion pump, not an async scheduler binding. Invokes
 * each terminal ticket at most once in settlement order, outside the heap gate.
 * Callback may inspect diagnostics, take results, collect or submit more work.
 * Notification does not consume a ticket: explicit take still releases storage.
 * Polling take before dispatch suppresses that notification. A racing external
 * take can invalidate the ticket; use one consumer if delivery must consume it.
 * Return count, or -1 for wrong owner, nested dispatch, null callback, limit >16.
 * Zero limit is a no-op. Newly completed jobs may use the remaining budget.
 * Callback/state must remain valid for this synchronous call; no managed pointers
 * in state. Destroy is rejected during dispatch, including after callback close. */
typedef void (*neoclr_worker_completion)(neoclr_workers *, neoclr_worker_handle, void *);
int neoclr_workers_dispatch(neoclr_workers *, unsigned limit, neoclr_worker_completion, void *);
/* Terminal diagnostic snapshot before take. Copies UTF-8 bytes without a NUL;
 * never exports managed pointers. 1 copied (code 0 means no guest fault), 0 pending,
 * -1 invalid, -2 insufficient buffer. Non-success preserves outputs. Take releases
 * diagnostics; cancellation wins over faults and reports no diagnostic. */
typedef struct { uint32_t code, frame_count; uint64_t message_bytes; } neoclr_worker_fault_info;
int neoclr_workers_fault_copy(neoclr_workers *, neoclr_worker_handle,
    neoclr_worker_fault_info *, unsigned char *message, size_t capacity);
int neoclr_workers_collect(neoclr_workers *, uint64_t *live_extent);
/* Owner-only close: stop admission, request cancellation, join outside gate.
 * Cooperative callbacks must return; no forced interruption or bounded join. */
int neoclr_workers_close(neoclr_workers *);
/* Reject outstanding tickets/roots or unjoined workers; no implicit discard. */
int neoclr_workers_destroy(neoclr_workers *);
#ifdef NEOCLR_WORKER_GC
/* Restricted compiler-instrumented synchronous callbacks only. Requires an image
 * built with --native-worker-safepoints and the matching collector attachment.
 * The compiler rejects unaudited services/type initialization. Shared application
 * fields still require synchronization/disjoint ownership. No Task/I/O migration. */
int32_t neoclr_worker_invoke_capture_parallel(neoclr_worker_job *);
int neoclr_workers_compiled_statistics(neoclr_workers *, unsigned *peak, uint64_t *collections);
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
/* Private opt-in adapter diagnostic. Owner only; successful service requests,
 * including allocator failures, not allocation count or GC epochs. */
int neoclr_workers_allocation_requests(neoclr_workers *, uint64_t *);
#ifdef NEOCLR_WORKER_COLLECTION_OWNER
/* Private diagnostic: completed collections executed by the service. */
int neoclr_workers_collection_requests(neoclr_workers *, uint64_t *);
#ifdef NEOCLR_WORKER_READ_EPOCHS
int neoclr_workers_read_epoch_statistics(neoclr_workers *,uint64_t *,uint64_t *,uint64_t *,uint64_t *);
#endif
#endif
#endif
#endif
#endif
