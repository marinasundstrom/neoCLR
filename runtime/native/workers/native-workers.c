#define _POSIX_C_SOURCE 200809L
#include "native-worker-trace.h"
#ifdef NEOCLR_WORKER_QUEUES
#include "native-worker-queues.h"
#endif
#ifdef NEOCLR_WORKER_DELIVERY
#include "native-worker-delivery.h"
#include <fcntl.h>
#include <unistd.h>
#endif
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <time.h>
#if defined(NEOCLR_WORKER_ALLOCATION_OWNER) && (!defined(NEOCLR_WORKER_GC) || !defined(NEOCLR_GC_ALLOCATION_DISPATCH))
#error Worker allocation owner requires worker GC and allocation dispatch
#endif
#if defined(NEOCLR_WORKER_LOCAL_ALLOCATION) && !defined(NEOCLR_WORKER_COLLECTION_OWNER)
#error Local allocation requires the separate collection owner contract
#endif
#if defined(NEOCLR_WORKER_COLLECTION_OWNER) && (!defined(NEOCLR_WORKER_ALLOCATION_OWNER) || !defined(NEOCLR_WORKER_ROOT_SNAPSHOT))
#error Worker collection owner requires allocation owner and copied roots
#endif
#if defined(NEOCLR_WORKER_PACED) && !defined(NEOCLR_WORKER_ALLOCATION_OWNER)
#error Worker pacing requires pool allocation accounting
#endif
#if defined(NEOCLR_WORKER_READ_EPOCHS) && (!defined(NEOCLR_WORKER_COLLECTION_OWNER) || !defined(NEOCLR_WORKER_PACED))
#error Read epochs require a paced collection owner
#endif
#if defined(NEOCLR_WORKER_ROOT_REUSE) && !defined(NEOCLR_WORKER_ROOT_SNAPSHOT)
#error Root buffer reuse requires copied participant roots
#endif
#ifdef NEOCLR_WORKER_ROOT_REUSE
#include "../gc/participant-roots.h"
#endif
#define JOBS 16
#define ROOTS 32
#define THREADS 4
#define POOL_ROOT_SLOTS (ROOTS + (2 + NEOCLR_WORKER_LOCAL_ROOTS + 65) * JOBS)
enum { FREE, QUEUED, RUNNING, DONE };
struct neoclr_worker_job {
    struct neoclr_workers *pool;
    uint64_t generation, order, completion_order;
    unsigned state;
#ifdef NEOCLR_WORKER_QUEUES
    unsigned queue;
#endif
    int cancelled, outcome, delivered;
    void *capture;
    void *local_roots[NEOCLR_WORKER_LOCAL_ROOTS];
    neoclr_worker_value result;
    neoclr_aot_fault fault;
    neoclr_worker_callback callback;
    void *native_state;
};
struct compiled_participant;
#if defined(NEOCLR_WORKER_TRACE) || defined(NEOCLR_WORKER_QUEUES)
struct worker_start { struct neoclr_workers *pool; unsigned lane; };
#endif
#ifdef NEOCLR_WORKER_TRACE
struct trace_lane { size_t count; uint64_t attempted, dropped; };
#endif
struct root { uint64_t generation; void *value; };
struct neoclr_workers {
    uint64_t identity, submission, settlement;
    pthread_t owner, threads[THREADS];
#if defined(NEOCLR_WORKER_TRACE) || defined(NEOCLR_WORKER_QUEUES)
    struct worker_start starts[THREADS];
#endif
#ifdef NEOCLR_WORKER_QUEUES
    struct { neoclr_worker_job *jobs[JOBS]; unsigned head, count; } queues[THREADS + 1];
    unsigned queue_workers, local_turns[THREADS], victim[THREADS];
    neoclr_worker_queue_counts queue_counts;
#endif
#ifdef NEOCLR_WORKER_TRACE
    neoclr_worker_trace_event *trace_events;
    size_t trace_capacity;
    uint64_t trace_origin;
    struct trace_lane trace_lanes[THREADS + 1];
#endif
    pthread_mutex_t gate;
    pthread_cond_t ready, completion;
    unsigned thread_count, capacity, readers, mutators, waiting_writers;
    int closing, joined, dispatching;
#ifdef NEOCLR_WORKER_DELIVERY
    int delivery_pipe[2], delivery_signaled;
#endif
#ifdef NEOCLR_WORKER_GC
    pthread_cond_t compiled_ready;
    _Atomic unsigned compiled_pause_requested;
    unsigned compiled_count, compiled_running, compiled_waiters, compiled_peak;
    uint64_t compiled_collections;
#ifdef NEOCLR_WORKER_ROOT_REUSE
    neoclr_gc_root_buffer root_buffer;
#endif
    struct compiled_participant *participants[JOBS], *compiled_exclusive;
#endif
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
    struct {
        pthread_t thread;
        pthread_cond_t ready;
        int started, stop, pending;
        neoclr_aot_text_arena *arena;
        uint64_t bytes, used, requests;
#ifdef NEOCLR_WORKER_PACED
        uint64_t allocations, allocated_bytes, last_allocations, last_bytes, boundaries;
#endif
        uint32_t kind;
        int32_t status;
        void *result;
#ifdef NEOCLR_WORKER_COLLECTION_OWNER
        neoclr_aot_context collector_context;
        const uint64_t *roots;
        uint32_t root_count;
        uint64_t collections;
#ifdef NEOCLR_WORKER_READ_EPOCHS
        int epoch_active, epoch_wanted;
        uint64_t epoch_started, epoch_finished, epoch_readers, epoch_payload_bytes;
#endif
#endif
    } allocation;
#endif
    neoclr_aot_context context;
    neoclr_worker_invoke invoke;
    struct neoclr_worker_job jobs[JOBS];
    struct root roots[ROOTS];
};
static _Atomic uint64_t next_identity = 1;
static _Thread_local neoclr_worker_job *entered, *executing, *reading, *mutating;
#ifdef NEOCLR_WORKER_TRACE
static _Thread_local neoclr_workers *trace_pool;
static _Thread_local unsigned trace_worker;
static uint64_t trace_now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) return 0;
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
/* External emissions require the existing gate. Each worker is the sole writer
 * to its lane, including RETURN outside that gate. No reader until joined. */
static void trace_emit(neoclr_workers *p, unsigned kind, neoclr_worker_job *j, unsigned detail) {
    if (!p->trace_events) return;
    unsigned lane = trace_pool == p ? trace_worker : 0;
    struct trace_lane *b = &p->trace_lanes[lane];
    uint64_t sequence = ++b->attempted;
    if (b->count == p->trace_capacity) { ++b->dropped; return; }
    uint64_t ns = trace_now();
    if (!ns) { ++b->dropped; return; }
    p->trace_events[lane * p->trace_capacity + b->count++] =
        (neoclr_worker_trace_event){ns, sequence, j ? j->generation : 0,
            kind, j ? (uint32_t)(j - p->jobs) : UINT32_MAX, detail};
}
int neoclr_workers_trace_view(neoclr_workers *p, unsigned lane, neoclr_worker_trace_view *out) {
    if (!out || !pthread_equal(p->owner, pthread_self()) || !p->joined ||
        !p->trace_events || lane > p->thread_count) return 0;
    struct trace_lane *b = &p->trace_lanes[lane];
    *out = (neoclr_worker_trace_view){p->trace_events + lane * p->trace_capacity,
        p->identity, p->trace_origin, b->attempted, b->dropped, b->count,
        p->trace_capacity, p->thread_count};
    return 1;
}
#else
#define trace_emit(p, kind, job, detail) ((void)0)
#endif
static void lock(neoclr_workers *p) { if (entered || reading || mutating || pthread_mutex_lock(&p->gate)) abort(); }
static void unlock(neoclr_workers *p) { if (pthread_mutex_unlock(&p->gate)) abort(); }
/* Metadata remains mutex-protected. Only operations touching arena/object bytes
 * need phase quiescence. Pending exclusive work prevents fresh phase admission. */
static void heap_lock(neoclr_workers *p) {
    lock(p);
    ++p->waiting_writers;
    while (p->readers || p->mutators
#ifdef NEOCLR_WORKER_GC
        || p->compiled_count
#endif
    ) if (pthread_cond_wait(&p->completion, &p->gate)) abort();
#ifdef NEOCLR_WORKER_READ_EPOCHS
    if (p->allocation.epoch_active) abort();
#endif
    --p->waiting_writers;
    pthread_cond_broadcast(&p->completion);
}
static neoclr_worker_job *job_for(neoclr_workers *p, neoclr_worker_handle h) {
    if (h.kind != 2 || h.pool != p->identity || h.slot >= p->capacity) return NULL;
    neoclr_worker_job *j = &p->jobs[h.slot];
    return j->state != FREE && j->generation == h.generation ? j : NULL;
}
static struct root *root_for(neoclr_workers *p, neoclr_worker_handle h) {
    if (h.kind != 1 || h.pool != p->identity || h.slot >= ROOTS) return NULL;
    struct root *r = &p->roots[h.slot];
    return r->value && r->generation == h.generation ? r : NULL;
}
static int adopt(neoclr_workers *p, void *value, neoclr_worker_handle *out) {
    if (!value) { *out = (neoclr_worker_handle){0}; return 1; }
    for (unsigned i = 0; i < ROOTS; i++) {
        struct root *r = &p->roots[i];
        if (!r->value && r->generation != UINT64_MAX) {
            ++r->generation; r->value = value;
            *out = (neoclr_worker_handle){p->identity, r->generation, i, 1};
            return 1;
        }
    }
    return 0;
}
#ifdef NEOCLR_WORKER_QUEUES
static _Thread_local neoclr_workers *queue_pool;
static _Thread_local unsigned queue_worker;
/* All deque metadata and job state change under the existing pool gate. */
static void queue_push(neoclr_workers *p, neoclr_worker_job *j) {
    unsigned q = queue_pool == p && executing ? queue_worker + 1 : 0;
    unsigned count = p->queues[q].count;
    if (count == JOBS) abort();
    p->queues[q].jobs[(p->queues[q].head + count) % JOBS] = j;
    p->queues[q].count++;
    j->queue = q;
}
static neoclr_worker_job *queue_pop(neoclr_workers *p, unsigned q, int newest) {
    if (!p->queues[q].count) return NULL;
    unsigned offset = newest ? p->queues[q].count - 1 : 0;
    neoclr_worker_job *j = p->queues[q].jobs[(p->queues[q].head + offset) % JOBS];
    if (!newest) p->queues[q].head = (p->queues[q].head + 1) % JOBS;
    p->queues[q].count--;
    if (j->state != QUEUED || j->queue != q) abort();
    return j;
}
static void queue_remove(neoclr_worker_job *j) {
    neoclr_workers *p = j->pool;
    unsigned q = j->queue, count = p->queues[q].count, at = 0;
    while (at < count && p->queues[q].jobs[(p->queues[q].head + at) % JOBS] != j) at++;
    if (at == count) abort();
    for (; at + 1 < count; at++)
        p->queues[q].jobs[(p->queues[q].head + at) % JOBS] =
            p->queues[q].jobs[(p->queues[q].head + at + 1) % JOBS];
    p->queues[q].count--;
}
static neoclr_worker_job *queue_next(neoclr_workers *p) {
    unsigned w = queue_worker;
    neoclr_worker_job *j;
    if (p->local_turns[w] >= 8 && (j = queue_pop(p, 0, 0))) {
        p->local_turns[w] = 0; p->queue_counts.global++; return j;
    }
    if ((j = queue_pop(p, w + 1, 1))) {
        if (p->local_turns[w] < 8) p->local_turns[w]++;
        p->queue_counts.local++; return j;
    }
    if ((j = queue_pop(p, 0, 0))) {
        p->local_turns[w] = 0; p->queue_counts.global++; return j;
    }
    for (unsigned n = 0; n < p->queue_workers; n++) {
        unsigned victim = p->victim[w]++ % p->queue_workers;
        if (victim == w) continue;
        if ((j = queue_pop(p, victim + 1, 0))) {
            p->local_turns[w] = 0; p->queue_counts.stolen++; return j;
        }
        p->queue_counts.empty_steal_probes++;
    }
    return NULL;
}
int neoclr_workers_queue_counts(neoclr_workers *p, neoclr_worker_queue_counts *out) {
    if (!out || !pthread_equal(p->owner, pthread_self()) || !p->joined) return 0;
    *out = p->queue_counts;
    return 1;
}
#endif
#ifdef NEOCLR_WORKER_DELIVERY
/* One byte represents the level, not one byte per completion. All transitions
 * share the job-state gate, so dispatch budget exhaustion cannot lose a wake. */
static void delivery_refresh(neoclr_workers *p) {
    if (p->delivery_pipe[0] < 0) return;
    int pending = 0;
    for (unsigned i = 0; i < p->capacity; i++)
        pending |= p->jobs[i].state == DONE && !p->jobs[i].delivered;
    if (pending == p->delivery_signaled) return;
    unsigned char byte = 1;
    ssize_t n;
    do {
        n = pending ? write(p->delivery_pipe[1], &byte, 1)
                    : read(p->delivery_pipe[0], &byte, 1);
    } while (n < 0 && errno == EINTR);
    if (n != 1) abort(); /* Violated borrowed-descriptor/lifetime contract. */
    p->delivery_signaled = pending;
}
int neoclr_workers_completion_descriptor(neoclr_workers *p) {
    if (!pthread_equal(p->owner, pthread_self())) return -1;
    lock(p);
    if (p->delivery_pipe[0] < 0) {
        int descriptors[2];
        if (pipe(descriptors)) { unlock(p); return -1; }
        for (unsigned i = 0; i < 2; i++) {
            int flags = fcntl(descriptors[i], F_GETFL);
            if (flags < 0 || fcntl(descriptors[i], F_SETFL, flags | O_NONBLOCK) ||
                fcntl(descriptors[i], F_SETFD, FD_CLOEXEC)) {
                close(descriptors[0]); close(descriptors[1]); unlock(p); return -1;
            }
        }
        p->delivery_pipe[0] = descriptors[0]; p->delivery_pipe[1] = descriptors[1];
        delivery_refresh(p);
    }
    int descriptor = p->delivery_pipe[0];
    unlock(p); return descriptor;
}
#else
#define delivery_refresh(p) ((void)0)
#endif
/* Called only under the gate; completion order is independent of submission. */
static void complete(neoclr_worker_job *j) {
    if (j->pool->settlement == UINT64_MAX) abort();
    j->completion_order = ++j->pool->settlement;
#ifdef NEOCLR_WORKER_QUEUES
    if (j->state == QUEUED) queue_remove(j);
#endif
    j->state = DONE;
    delivery_refresh(j->pool);
    trace_emit(j->pool, NEOCLR_TRACE_SETTLE, j, (unsigned)j->outcome);
    pthread_cond_broadcast(&j->pool->completion);
}
static void *run(void *argument) {
#if defined(NEOCLR_WORKER_TRACE) || defined(NEOCLR_WORKER_QUEUES)
    struct worker_start *start = argument;
    neoclr_workers *p = start->pool;
#ifdef NEOCLR_WORKER_TRACE
    trace_pool = p; trace_worker = start->lane;
#endif
#ifdef NEOCLR_WORKER_QUEUES
    queue_pool = p; queue_worker = start->lane - 1;
#endif
#else
    neoclr_workers *p = argument;
#endif
    trace_emit(p, NEOCLR_TRACE_START, NULL, 0);
    for (;;) {
        lock(p);
        neoclr_worker_job *j = NULL;
        while (!j && !p->closing) {
#ifdef NEOCLR_WORKER_QUEUES
            j = queue_next(p);
#else
            for (unsigned i = 0; i < p->capacity; i++)
                if (p->jobs[i].state == QUEUED && (!j || p->jobs[i].order < j->order)) j = &p->jobs[i];
#endif
            if (!j) {
                trace_emit(p, NEOCLR_TRACE_IDLE_BEGIN, NULL, 0);
                if (pthread_cond_wait(&p->ready, &p->gate)) abort();
                trace_emit(p, NEOCLR_TRACE_IDLE_END, NULL, 0);
            }
        }
        if (!j) { trace_emit(p, NEOCLR_TRACE_STOP, NULL, 0); unlock(p); return NULL; }
        j->state = RUNNING;
        trace_emit(p, NEOCLR_TRACE_RUN, j, 0);
        unlock(p);
        executing = j;
        int outcome = j->callback(j, j->native_state);
        trace_emit(p, NEOCLR_TRACE_RETURN, j, 0);
        executing = NULL;
        if (entered || reading || mutating) abort();
        lock(p);
        j->outcome = j->cancelled ? NEOCLR_WORKER_CANCELLED :
            (outcome || j->fault.code) ? NEOCLR_WORKER_FAILED : NEOCLR_WORKER_OK;
        if (j->outcome != NEOCLR_WORKER_OK) j->result = (neoclr_worker_value){0};
        if (j->cancelled) memset(&j->fault, 0, sizeof(j->fault));
        j->capture = NULL;
        memset(j->local_roots, 0, sizeof(j->local_roots));
        complete(j);
        unlock(p);
    }
}
#ifdef NEOCLR_WORKER_TRACE
neoclr_workers *neoclr_workers_create(unsigned workers, unsigned capacity, size_t bytes) {
    return neoclr_workers_create_traced(workers, capacity, bytes, 0);
}
neoclr_workers *neoclr_workers_create_traced(unsigned workers, unsigned capacity, size_t bytes, size_t events) {
    if (events > 65536) return NULL;
#else
neoclr_workers *neoclr_workers_create(unsigned workers, unsigned capacity, size_t bytes) {
#endif
    if (!workers || workers > THREADS || !capacity || capacity > JOBS || bytes < 256 || (bytes & 7)) return NULL;
    neoclr_workers *p = calloc(1, sizeof(*p));
    if (!p) return NULL;
#ifdef NEOCLR_WORKER_DELIVERY
    p->delivery_pipe[0] = p->delivery_pipe[1] = -1;
#endif
    p->context.text.data = calloc(1, bytes);
    if (!p->context.text.data) { free(p); return NULL; }
    p->context.text.capacity = bytes;
    p->identity = atomic_fetch_add(&next_identity, 1);
    if (!p->identity || p->identity == UINT64_MAX) abort();
    p->owner = pthread_self(); p->capacity = capacity;
    if (pthread_mutex_init(&p->gate, NULL)) { free(p->context.text.data); free(p); return NULL; }
    if (pthread_cond_init(&p->ready, NULL)) {
        pthread_mutex_destroy(&p->gate); free(p->context.text.data); free(p); return NULL;
    }
    if (pthread_cond_init(&p->completion, NULL)) {
        pthread_cond_destroy(&p->ready); pthread_mutex_destroy(&p->gate);
        free(p->context.text.data); free(p); return NULL;
    }
#ifdef NEOCLR_WORKER_GC
    atomic_init(&p->compiled_pause_requested, 0);
    if (pthread_cond_init(&p->compiled_ready, NULL)) {
        pthread_cond_destroy(&p->completion); pthread_cond_destroy(&p->ready);
        pthread_mutex_destroy(&p->gate); free(p->context.text.data); free(p); return NULL;
    }
#endif
#ifdef NEOCLR_WORKER_TRACE
    if (events) {
        p->trace_origin = trace_now();
        p->trace_events = calloc((workers + 1) * events, sizeof(*p->trace_events));
        if (!p->trace_origin || !p->trace_events) {
            neoclr_workers_close(p); neoclr_workers_destroy(p); return NULL;
        }
        p->trace_capacity = events;
    }
#endif
#ifdef NEOCLR_WORKER_QUEUES
    p->queue_workers = workers;
#endif
    for (unsigned i = 0; i < capacity; i++) p->jobs[i].pool = p;
    for (; p->thread_count < workers; p->thread_count++) {
#if defined(NEOCLR_WORKER_TRACE) || defined(NEOCLR_WORKER_QUEUES)
        p->starts[p->thread_count] = (struct worker_start){p, p->thread_count + 1};
        if (pthread_create(&p->threads[p->thread_count], NULL, run, &p->starts[p->thread_count])) {
#else
        if (pthread_create(&p->threads[p->thread_count], NULL, run, p)) {
#endif
            neoclr_workers_close(p); neoclr_workers_destroy(p); return NULL;
        }
    }
    return p;
}
int32_t neoclr_workers_bootstrap(neoclr_workers *p, neoclr_worker_bootstrap bootstrap,
    neoclr_worker_invoke invoke, int32_t argument, neoclr_worker_handle *out) {
    if (!pthread_equal(p->owner, pthread_self()) || !bootstrap || !invoke || !out) return 3;
    heap_lock(p);
    if (p->closing || p->invoke || p->context.text.used || neoclr_root_probe_head_v1() || neoclr_roots_parked_v1()) {
        unlock(p); return 3;
    }
    uint64_t handle = 0;
    int32_t status = bootstrap(argument, &handle, &p->context);
    if (neoclr_gc_owned_roots_check_v1(&p->context, handle ? &handle : NULL, handle ? 1 : 0)) abort();
    if (!status) {
        void *value = NULL;
        status = neoclr_gc_callback_read_v1(&p->context, handle, &value);
        if (!status && !adopt(p, value, out)) status = 5;
    }
    if (handle && neoclr_gc_host_root_release_v1(&p->context, handle)) abort();
    /* Keep the image identity even on failure; teardown owns its static/intern state. */
    p->invoke = invoke;
    unlock(p); return status;
}
static unsigned pool_slots(neoclr_workers *p, neoclr_probe_storage *slots) {
    unsigned count = 0;
    for (unsigned i = 0; i < ROOTS; i++) slots[count++] = (neoclr_probe_storage){&p->roots[i].value, 8, 0};
    for (unsigned i = 0; i < p->capacity; i++) {
        slots[count++] = (neoclr_probe_storage){&p->jobs[i].capture, 8, 0};
        slots[count++] = (neoclr_probe_storage){&p->jobs[i].result.reference, 8, 0};
        if (p->jobs[i].fault.code) {
            if (p->jobs[i].fault.frame_count > 64) abort();
            slots[count++] = (neoclr_probe_storage){&p->jobs[i].fault.message, 8, 0};
            for (unsigned n = 0; n < p->jobs[i].fault.frame_count; n++)
                slots[count++] = (neoclr_probe_storage){&p->jobs[i].fault.frames[n].function, 8, 0};
        }
        for (unsigned n = 0; n < NEOCLR_WORKER_LOCAL_ROOTS; n++)
            slots[count++] = (neoclr_probe_storage){&p->jobs[i].local_roots[n], 8, 0};
    }
    return count;
}
int32_t neoclr_worker_invoke_capture(neoclr_worker_job *j) {
    if (executing != j) abort();
    neoclr_workers *p = j->pool;
    neoclr_worker_enter(j);
    if (!p->invoke || j->fault.code || p->context.fault.code || neoclr_root_probe_head_v1() || neoclr_roots_parked_v1()) {
        neoclr_worker_leave(j); return 3;
    }
    /* ABI v4 embeds its arena cursor/static root beside per-invocation faults.
     * Transfer exclusive ownership, rather than making two active cursor copies.
     * The gate excludes readers, native mutators, allocation and owner collection
     * until the invocation returns its heap state. No context identity escapes. */
    neoclr_aot_context invocation = {
        .text = p->context.text, .static_storage = p->context.static_storage
    };
    unsigned char *arena = invocation.text.data;
    uint64_t capacity = invocation.text.capacity;
    p->context.text = (neoclr_aot_text_arena){0};
    p->context.static_storage = NULL;
    uint64_t handle = 0;
    int32_t status = neoclr_gc_host_root_create_v1(&invocation, j->capture, &handle);
    if (!status) {
        neoclr_probe_storage slots[POOL_ROOT_SLOTS];
        unsigned count = pool_slots(p, slots);
        neoclr_probe_frame frame;
        neoclr_parked_roots parked = {0};
        /* Park only this adapter frame, leaving generated callback admission
         * quiescent. The gate keeps all referenced pool slots stable. */
        neoclr_gc_enter_v1(&frame, &invocation, 0, slots, count, "[]", 2);
        neoclr_roots_park_v1(&parked);
        status = p->invoke(handle, &invocation);
        if (neoclr_root_probe_head_v1() || neoclr_roots_parked_v1() != &parked) abort();
        neoclr_roots_resume_v1(&parked);
        neoclr_gc_leave_v1(&frame);
        if (neoclr_gc_owned_roots_check_v1(&invocation, &handle, 1)) abort();
        if (neoclr_gc_host_root_release_v1(&invocation, handle)) abort();
    }
    if (invocation.text.data != arena || invocation.text.capacity != capacity ||
        invocation.text.used > capacity || p->context.text.data ||
        p->context.text.capacity || p->context.text.used || p->context.static_storage) abort();
    p->context.text = invocation.text;
    p->context.static_storage = invocation.static_storage;
    if (invocation.fault.code) j->fault = invocation.fault;
    memset(&invocation, 0, sizeof(invocation));
    neoclr_worker_leave(j);
    return status;
}
int neoclr_workers_root_new(neoclr_workers *p, uint64_t initial, neoclr_worker_handle *out) {
    heap_lock(p);
    int available = 0;
    for (unsigned i = 0; i < ROOTS; i++) available |= !p->roots[i].value && p->roots[i].generation != UINT64_MAX;
    void *value = NULL;
    int ok = !p->closing && available && !neoclr_gc_allocate_v1(&p->context.text, 8, NEOCLR_GC_BYTES, &value);
    if (ok) { memcpy(value, &initial, 8); ok = adopt(p, value, out); }
    unlock(p); return ok;
}
int neoclr_workers_root_read(neoclr_workers *p, neoclr_worker_handle h, uint64_t *out) {
    heap_lock(p); struct root *r = root_for(p, h);
    uint64_t bytes = 0; uint32_t kind = 0;
    int ok = r && !neoclr_gc_allocation_info_v1(&p->context, r->value, &bytes, &kind) && bytes >= 8;
    if (ok) memcpy(out, r->value, 8);
    unlock(p); return ok;
}
int neoclr_workers_root_release(neoclr_workers *p, neoclr_worker_handle h) {
    lock(p); struct root *r = root_for(p, h);
    if (r) r->value = NULL;
    unlock(p); return r != NULL;
}
int neoclr_workers_submit(neoclr_workers *p, neoclr_worker_handle capture,
    neoclr_worker_callback callback, void *native_state, neoclr_worker_handle *out) {
    lock(p); struct root *r = root_for(p, capture); int ok = 0;
    if (!p->closing && r && callback && p->submission != UINT64_MAX) for (unsigned i = 0; i < p->capacity; i++) {
        neoclr_worker_job *j = &p->jobs[i];
        if (j->state != FREE || j->generation == UINT64_MAX) continue;
        j->order = ++p->submission; ++j->generation; j->capture = r->value; j->callback = callback;
        j->native_state = native_state; j->cancelled = 0; j->delivered = 0; j->result = (neoclr_worker_value){0};
        j->state = QUEUED; *out = (neoclr_worker_handle){p->identity, j->generation, i, 2};
#ifdef NEOCLR_WORKER_QUEUES
        queue_push(p, j);
#endif
        trace_emit(p, NEOCLR_TRACE_ENQUEUE, j, 0);
        pthread_cond_signal(&p->ready); ok = 1; break;
    }
    unlock(p); return ok;
}
int neoclr_workers_cancel(neoclr_workers *p, neoclr_worker_handle h) {
    lock(p); neoclr_worker_job *j = job_for(p, h); int ok = j && j->state != DONE && !j->cancelled;
    if (ok) {
        j->cancelled = 1;
        trace_emit(p, NEOCLR_TRACE_CANCEL, j, 0);
        if (j->state == QUEUED) { j->capture = NULL; j->outcome = NEOCLR_WORKER_CANCELLED; complete(j); }
    }
    if (ok) pthread_cond_broadcast(&p->completion);
    unlock(p); return ok;
}
int neoclr_worker_cancelled(neoclr_worker_job *j) {
    lock(j->pool); int result = j->cancelled; unlock(j->pool); return result;
}
void neoclr_worker_enter(neoclr_worker_job *j) { if (executing != j) abort(); heap_lock(j->pool); entered = j; }
const void *neoclr_worker_read_begin(neoclr_worker_job *j) {
    if (executing != j) abort();
    neoclr_workers *p = j->pool;
    lock(p);
    while (p->waiting_writers || p->mutators
#ifdef NEOCLR_WORKER_GC
        || p->compiled_count
#endif
    ) if (pthread_cond_wait(&p->completion, &p->gate)) abort();
#ifdef NEOCLR_WORKER_READ_EPOCHS
    if (p->allocation.epoch_active) abort();
#endif
    ++p->readers;
    reading = j;
    const void *capture = j->capture;
    unlock(p);
    return capture;
}
void neoclr_worker_read_end(neoclr_worker_job *j) {
    if (reading != j || executing != j) abort();
    neoclr_workers *p = j->pool;
    if (pthread_mutex_lock(&p->gate)) abort();
    if (!p->readers) abort();
    --p->readers;
    reading = NULL;
    if (!p->readers) pthread_cond_broadcast(&p->completion);
    unlock(p);
}
void *neoclr_worker_mutate_begin(neoclr_worker_job *j) {
    if (executing != j) abort();
    neoclr_workers *p = j->pool;
    lock(p);
    while (p->waiting_writers || p->readers
#ifdef NEOCLR_WORKER_GC
        || p->compiled_count
#endif
    )
        if (pthread_cond_wait(&p->completion, &p->gate)) abort();
#ifdef NEOCLR_WORKER_READ_EPOCHS
    if (p->allocation.epoch_active) abort();
#endif
    ++p->mutators;
    mutating = j;
    void *capture = j->capture;
    unlock(p);
    return capture;
}
void neoclr_worker_mutate_end(neoclr_worker_job *j) {
    if (mutating != j || executing != j) abort();
    neoclr_workers *p = j->pool;
    if (pthread_mutex_lock(&p->gate)) abort();
    if (!p->mutators) abort();
    --p->mutators;
    mutating = NULL;
    if (!p->mutators) pthread_cond_broadcast(&p->completion);
    unlock(p);
}
void *neoclr_worker_mutate_poll(neoclr_worker_job *j) {
    neoclr_worker_mutate_end(j);
    return neoclr_worker_mutate_begin(j);
}
void *neoclr_worker_capture(neoclr_worker_job *j) { if (entered != j) abort(); return j->capture; }
void *neoclr_worker_allocate(neoclr_worker_job *j, uint64_t bytes, uint32_t kind) {
    if (entered != j) abort();
    void *result = NULL;
    if (neoclr_gc_allocate_v1(&j->pool->context.text, bytes, kind, &result)) return NULL;
    return result;
}
void neoclr_worker_result(neoclr_worker_job *j, neoclr_worker_value value) {
    if (entered != j) abort();
    uint64_t bytes; uint32_t kind;
    if (value.reference && neoclr_gc_allocation_info_v1(&j->pool->context, value.reference, &bytes, &kind)) abort();
    j->result = value;
}
int neoclr_worker_local_store(neoclr_worker_job *j, unsigned slot, void *value) {
    if (entered != j && mutating != j) abort();
    /* Arena headers are stable throughout a mutation phase; synchronize only
     * root metadata here. Never wait for quiescence while counted as a mutator. */
    if (mutating == j && pthread_mutex_lock(&j->pool->gate)) abort();
    uint64_t bytes; uint32_t kind;
    int valid = slot < NEOCLR_WORKER_LOCAL_ROOTS &&
        (!value || !neoclr_gc_allocation_info_v1(&j->pool->context, value, &bytes, &kind));
    if (valid) j->local_roots[slot] = value;
    if (mutating == j) unlock(j->pool);
    return valid;
}
void *neoclr_worker_local_load(neoclr_worker_job *j, unsigned slot) {
    if (entered != j && mutating != j) abort();
    return slot < NEOCLR_WORKER_LOCAL_ROOTS ? j->local_roots[slot] : NULL;
}
void neoclr_worker_leave(neoclr_worker_job *j) { if (entered != j) abort(); entered = NULL; unlock(j->pool); }
int neoclr_workers_wait(neoclr_workers *p, neoclr_worker_handle h, unsigned milliseconds) {
    if ((executing && executing->pool == p) || milliseconds > 60000) return -1;
    struct timespec until;
    if (clock_gettime(CLOCK_REALTIME, &until)) return -1;
    until.tv_sec += milliseconds / 1000;
    until.tv_nsec += (long)(milliseconds % 1000) * 1000000;
    if (until.tv_nsec >= 1000000000) { ++until.tv_sec; until.tv_nsec -= 1000000000; }
    lock(p);
    int status = -1;
    for (;;) {
        neoclr_worker_job *j = job_for(p, h);
        if (!j) break;
        if (j->state == DONE) { status = 1; break; }
        int error = pthread_cond_timedwait(&p->completion, &p->gate, &until);
        if (error == ETIMEDOUT) {
            j = job_for(p, h);
            status = !j ? -1 : j->state == DONE ? 1 : 0;
            break;
        }
        if (error) break;
    }
    unlock(p); return status;
}
int neoclr_workers_dispatch(neoclr_workers *p, unsigned limit,
    neoclr_worker_completion callback, void *state) {
    if (!pthread_equal(p->owner, pthread_self()) || !callback || limit > JOBS) return -1;
    lock(p);
    if (p->dispatching) { unlock(p); return -1; }
    p->dispatching = 1;
    unsigned delivered = 0;
    while (delivered < limit) {
        neoclr_worker_job *next = NULL;
        unsigned slot = 0;
        for (unsigned i = 0; i < p->capacity; i++) {
            neoclr_worker_job *j = &p->jobs[i];
            if (j->state == DONE && !j->delivered &&
                (!next || j->completion_order < next->completion_order)) {
                next = j; slot = i;
            }
        }
        if (!next) break;
        neoclr_worker_handle ticket = {p->identity, next->generation, slot, 2};
        next->delivered = 1;
        ++delivered;
        unlock(p);
        callback(p, ticket, state);
        lock(p);
    }
    p->dispatching = 0;
    delivery_refresh(p);
    unlock(p);
    return (int)delivered;
}
int neoclr_workers_fault_copy(neoclr_workers *p, neoclr_worker_handle h,
    neoclr_worker_fault_info *info, unsigned char *message, size_t capacity) {
    if (!info || (!message && capacity)) return -1;
    heap_lock(p);
    neoclr_worker_job *j = job_for(p, h);
    int status = -1;
    if (j) {
        status = 0;
        if (j->state == DONE) {
            uint64_t length = j->fault.message ? j->fault.message->length : 0;
            status = -2;
            if (length <= capacity) {
                if (length) memcpy(message, j->fault.message->bytes, (size_t)length);
                *info = (neoclr_worker_fault_info){j->fault.code, j->fault.frame_count, length};
                status = 1;
            }
        }
    }
    unlock(p); return status;
}
int neoclr_workers_take(neoclr_workers *p, neoclr_worker_handle h, int *outcome,
    neoclr_worker_value *value, neoclr_worker_handle *root) {
    lock(p); neoclr_worker_job *j = job_for(p, h); int status = -1;
    if (j) {
        status = 0;
        if (j->state == DONE && adopt(p, j->result.reference, root)) {
            *outcome = j->outcome; *value = j->result; value->reference = NULL;
            memset(&j->fault, 0, sizeof(j->fault));
            j->result = (neoclr_worker_value){0}; j->callback = NULL; j->native_state = NULL; j->state = FREE; status = 1;
        }
    }
    delivery_refresh(p);
    unlock(p); return status;
}
int neoclr_workers_collect(neoclr_workers *p, uint64_t *extent) {
    if (!pthread_equal(p->owner, pthread_self())) return 0;
    heap_lock(p);
    neoclr_probe_storage slots[POOL_ROOT_SLOTS]; unsigned count = pool_slots(p, slots);
    neoclr_probe_frame frame = {.context = &p->context, .storage = slots, .storage_count = count};
    int ok = !neoclr_gc_collect_v1(&p->context, &frame);
    if (ok) *extent = p->context.text.used;
    unlock(p); return ok;
}
int neoclr_workers_close(neoclr_workers *p) {
    if (!pthread_equal(p->owner, pthread_self()) || entered) return 0;
    lock(p);
    if (p->joined) { unlock(p); return 1; }
    p->closing = 1;
    for (unsigned i = 0; i < p->capacity; i++) {
        neoclr_worker_job *j = &p->jobs[i];
        if ((j->state == RUNNING || j->state == QUEUED) && !j->cancelled)
            trace_emit(p, NEOCLR_TRACE_CANCEL, j, 1);
        if (j->state == RUNNING) j->cancelled = 1;
        if (j->state == QUEUED) {
            j->cancelled = 1; j->capture = NULL; j->outcome = NEOCLR_WORKER_CANCELLED; complete(j);
        }
    }
    pthread_cond_broadcast(&p->ready); pthread_cond_broadcast(&p->completion); unlock(p);
    for (unsigned i = 0; i < p->thread_count; i++) if (pthread_join(p->threads[i], NULL)) abort();
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
    lock(p);
    if (p->allocation.pending || p->compiled_count) abort();
    p->allocation.stop=1;
    if (p->allocation.started) pthread_cond_signal(&p->allocation.ready);
    unlock(p);
    if (p->allocation.started) {
        if (pthread_join(p->allocation.thread,NULL)) abort();
        if (pthread_cond_destroy(&p->allocation.ready)) abort();
    }
#endif
    p->joined = 1; return 1;
}
int neoclr_workers_destroy(neoclr_workers *p) {
    if (!pthread_equal(p->owner, pthread_self()) || !p->joined || entered || p->dispatching) return 0;
    for (unsigned i = 0; i < p->capacity; i++) if (p->jobs[i].state != FREE) return 0;
    for (unsigned i = 0; i < ROOTS; i++) if (p->roots[i].value) return 0;
    if (p->invoke) {
        if (neoclr_gc_complete_invocation_v1(&p->context)) return 0;
        memset(&p->context.fault, 0, sizeof(p->context.fault));
    }
    uint64_t extent;
    if (!neoclr_workers_collect(p, &extent) || extent) return 0;
#ifdef NEOCLR_WORKER_GC
#ifdef NEOCLR_WORKER_ROOT_REUSE
    neoclr_gc_root_buffer_dispose_v1(&p->root_buffer);
#endif
    pthread_cond_destroy(&p->compiled_ready);
#endif
    pthread_cond_destroy(&p->completion); pthread_cond_destroy(&p->ready); pthread_mutex_destroy(&p->gate);
#ifdef NEOCLR_WORKER_TRACE
    free(p->trace_events);
#endif
#ifdef NEOCLR_WORKER_DELIVERY
    if (p->delivery_pipe[0] >= 0) {
        if (close(p->delivery_pipe[0]) || close(p->delivery_pipe[1])) abort();
    }
#endif
    free(p->context.text.data); free(p); return 1;
}

#ifdef NEOCLR_WORKER_GC
#include "compiled-workers.inc"
#endif
