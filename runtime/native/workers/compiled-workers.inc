#ifdef NEOCLR_WORKER_ROOT_SNAPSHOT
#include "../gc/participant-roots.h"
#endif
/* Included only by the opt-in NEOCLR_WORKER_GC pool. Generated frames live on
 * their originating stacks; a participant publishes them only while paused. */
typedef struct compiled_participant {
    neoclr_worker_job *job;
    neoclr_aot_context context;
    const neoclr_probe_frame *frames;
    uint64_t handle;
    unsigned slot;
    int running, exclusive;
#ifdef NEOCLR_WORKER_ROOT_SNAPSHOT
    int snapshot_attached;
#endif
} compiled_participant;
static _Thread_local compiled_participant *compiled_current;
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
#include "allocation-owner.inc"
#endif

/* Caller holds metadata gate and the sole generated execution lease. */
static int32_t compiled_collect_locked(compiled_participant *self, const neoclr_probe_frame *head) {
    neoclr_workers *p = self->job->pool;
    if (p->compiled_exclusive != self || p->compiled_running != 1) abort();
    neoclr_probe_storage slots[POOL_ROOT_SLOTS];
    unsigned count = pool_slots(p, slots);
    neoclr_probe_frame pool_frame = {.context = &self->context, .storage = slots, .storage_count = count};
    neoclr_gc_participant roots[JOBS + 1];
    unsigned participants = 0;
    roots[participants++] = (neoclr_gc_participant){&self->context, &pool_frame};
    for (unsigned i = 0; i < p->capacity; i++) {
        compiled_participant *other = p->participants[i];
        if (!other || other == self) continue;
        if (other->running) abort();
#ifdef NEOCLR_WORKER_ROOT_SNAPSHOT
        /* Registered waiters have no invocation roots until their first heap
         * lease. Their captures/locals are already present in pool_slots. */
        if (!other->snapshot_attached) {
            if (other->frames || other->context.text.data || other->context.static_storage || other->context.fault.code) abort();
            continue;
        }
#endif
        roots[participants++] = (neoclr_gc_participant){&other->context, other->frames};
    }
#ifdef NEOCLR_WORKER_ROOT_SNAPSHOT
    /* All participants remain paused. This opt-in equivalence step copies root
     * values now, but does not yet authorize concurrent marking or reclamation. */
    roots[participants++] = (neoclr_gc_participant){&self->context, head};
    size_t count_words, copied;
    int32_t status = neoclr_gc_participant_root_count_v1(&self->context, roots, participants, &count_words);
    if (status || count_words > UINT32_MAX || count_words > SIZE_MAX / sizeof(uint64_t)) return 3;
    uint64_t *values = malloc(count_words * sizeof(uint64_t));
    if (count_words && !values) return 3;
    status = neoclr_gc_participant_root_capture_v1(&self->context, roots, participants, values, count_words, &copied);
    if (!status) {
        neoclr_probe_frame snapshot = {.context=&self->context, .lanes=values, .lane_count=(uint32_t)copied};
        status = neoclr_gc_collect_conditional_v1(&self->context, &snapshot, NULL, 0);
    }
    free(values);
#else
    int32_t status = neoclr_gc_collect_participants_v1(&self->context, head, roots, participants);
#endif
    if (!status) ++p->compiled_collections;
    return status;
}
static int32_t compiled_collect(neoclr_aot_context *context, const neoclr_probe_frame *head) {
    compiled_participant *self = compiled_current;
    if (!self || context != &self->context) abort();
    neoclr_workers *p = self->job->pool;
    lock(p);
    int32_t status = compiled_collect_locked(self, head);
    unlock(p);
    return status;
}
static void compiled_pause_locked(compiled_participant *self, const neoclr_probe_frame *head) {
    neoclr_workers *p = self->job->pool;
    if (self->exclusive) {
        if (p->compiled_exclusive != self) abort();
        if (self->context.text.data != p->context.text.data ||
            self->context.text.capacity != p->context.text.capacity) abort();
        p->context.text = self->context.text;
        p->context.static_storage = self->context.static_storage;
        p->compiled_exclusive = NULL;
        self->exclusive = 0;
    }
    if (self->running) {
        --p->compiled_running;
        self->running = 0;
    }
    self->frames = head;
    atomic_store_explicit(&p->compiled_pause_requested,
        p->compiled_waiters != 0 || p->compiled_exclusive != NULL, memory_order_release);
    /* Only the last active participant can enable an exclusive waiter.
     * An exclusive lease is itself the sole active participant. */
    if (!p->compiled_running) pthread_cond_broadcast(&p->compiled_ready);
}
static void compiled_boundary(neoclr_aot_context *context, const neoclr_probe_frame *head, unsigned exclusive) {
    compiled_participant *self = compiled_current;
    if (!self || context != &self->context || exclusive > 1) abort();
    if (self->handle && neoclr_gc_worker_roots_check_v1(context, self->handle)) abort();
    neoclr_workers *p = self->job->pool;
    /* Keep the running lease across pure boundaries until an exclusive operation
     * requests quiescence. The collector cannot inspect this thread's changing
     * frame snapshots while it is counted as running. On the slow path, the
     * mutex publishes the complete frame chain before dropping that count. */
    if (!exclusive && self->running && !self->exclusive &&
        !atomic_load_explicit(&p->compiled_pause_requested, memory_order_acquire)) return;
    lock(p);
    compiled_pause_locked(self, head);
    if (exclusive) {
        ++p->compiled_waiters;
        atomic_store_explicit(&p->compiled_pause_requested, 1, memory_order_release);
        while (p->compiled_running) if (pthread_cond_wait(&p->compiled_ready, &p->gate)) abort();
        --p->compiled_waiters;
        p->compiled_exclusive = self;
        self->exclusive = 1;
        self->context.text = p->context.text;
        self->context.static_storage = p->context.static_storage;
#ifdef NEOCLR_WORKER_ROOT_SNAPSHOT
        self->snapshot_attached = 1;
#endif
    } else {
        while (p->compiled_exclusive || p->compiled_waiters)
            if (pthread_cond_wait(&p->compiled_ready, &p->gate)) abort();
    }
    self->running = 1;
    ++p->compiled_running;
    if (p->compiled_running > p->compiled_peak) p->compiled_peak = p->compiled_running;
    if (exclusive && self->handle && compiled_collect_locked(self, head)) abort();
    unlock(p);
}
int32_t neoclr_worker_invoke_capture_parallel(neoclr_worker_job *job) {
    if (executing != job || compiled_current || entered || reading || mutating ||
        neoclr_gc_worker_collect_hook || neoclr_gc_worker_boundary_hook) abort();
    neoclr_workers *p = job->pool;
    compiled_participant self = {.job = job, .slot = (unsigned)(job - p->jobs)};
    lock(p);
    if (!p->invoke || job->fault.code || p->context.fault.code || neoclr_root_probe_head_v1() || neoclr_roots_parked_v1()) {
        unlock(p); return 3;
    }
    while (p->readers || p->mutators || p->waiting_writers)
        if (pthread_cond_wait(&p->completion, &p->gate)) abort();
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
    if (neoclr_gc_allocation_dispatch_hook) abort();
    if (!p->allocation.started) {
        if (pthread_cond_init(&p->allocation.ready,NULL)) { unlock(p); return 3; }
        if (pthread_create(&p->allocation.thread,NULL,compiled_allocation_owner,p)) {
            pthread_cond_destroy(&p->allocation.ready);
            unlock(p); return 3;
        }
        p->allocation.started=1;
    }
#endif
    if (p->participants[self.slot]) abort();
    p->participants[self.slot] = &self;
    ++p->compiled_count;
    unlock(p);
    compiled_current = &self;
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
    neoclr_gc_allocation_dispatch_hook=compiled_allocate;
#endif
    neoclr_gc_worker_boundary_hook = compiled_boundary;
    neoclr_gc_worker_collect_hook = compiled_collect;
    compiled_boundary(&self.context, NULL, 1);
    int32_t status = neoclr_gc_host_root_create_v1(&self.context, job->capture, &self.handle);
    if (!status) {
        status = p->invoke(self.handle, &self.context);
        if (neoclr_root_probe_head_v1() || neoclr_roots_parked_v1()) abort();
        compiled_boundary(&self.context, NULL, 1);
        if (neoclr_gc_owned_roots_check_v1(&self.context, &self.handle, 1) ||
            neoclr_gc_host_root_release_v1(&self.context, self.handle)) abort();
    }
    lock(p);
    if (self.context.fault.code) job->fault = self.context.fault;
    compiled_pause_locked(&self, NULL);
    p->participants[self.slot] = NULL;
    --p->compiled_count;
    pthread_cond_broadcast(&p->completion);
    unlock(p);
    neoclr_gc_worker_collect_hook = NULL;
    neoclr_gc_worker_boundary_hook = NULL;
#ifdef NEOCLR_WORKER_ALLOCATION_OWNER
    neoclr_gc_allocation_dispatch_hook=NULL;
#endif
    compiled_current = NULL;
    return status;
}
int neoclr_workers_compiled_statistics(neoclr_workers *p, unsigned *peak, uint64_t *collections) {
    if (!peak || !collections || !pthread_equal(p->owner, pthread_self())) return 0;
    lock(p);
    *peak = p->compiled_peak; *collections = p->compiled_collections;
    unlock(p);
    return 1;
}
