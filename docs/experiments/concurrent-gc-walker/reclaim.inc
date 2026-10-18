/* Owner-only POC coordinator. Background tracing never touches the live heap. */
#ifndef RECLAIM_PHASE
#define RECLAIM_PHASE(phase) ((void)0)
#endif
static uint64_t cycles_applied, cycles_discarded, cycle_reclaimed_bytes;
typedef enum { CYCLE_PENDING, CYCLE_APPLIED, CYCLE_DISCARDED, CYCLE_EMPTY } cycle_result;
typedef struct {
    gc_worker worker;
    pthread_t owner;
    snapshot *job;
    neoclr_aot_context *context;
    uint64_t epoch;
} collection_cycle;
static void cycle_owner(collection_cycle *cycle) {
    if (!pthread_equal(cycle->owner, pthread_self())) abort();
}
static bool cycle_init(collection_cycle *cycle) {
    *cycle = (collection_cycle){.owner = pthread_self()};
    return gc_worker_init(&cycle->worker) == 0;
}
static bool cycle_start(collection_cycle *cycle, neoclr_aot_context *context) {
    cycle_owner(cycle);
    if (cycle->job) return false;
    snapshot *s = malloc(sizeof(*s));
    if (!s) return false;
    if (!capture(s, context)) { free(s); return false; }
    cycle->context = context;
    cycle->epoch = experiment_epoch;
    cycle->job = s;
    if (!gc_worker_submit(&cycle->worker, trace_snapshot, s)) abort();
    return true;
}
static bool cycle_apply(collection_cycle *cycle) {
    snapshot *s = cycle->job;
    neoclr_aot_context *context = cycle->context;
    if (cycle->epoch != experiment_epoch) return false;
    neoclr_aot_text_arena *a = &context->text;
    if ((uintptr_t)a->data != s->source || a->used < s->copy.text.used || !valid(a)) return false;
    /* Validate live integrity and captured identities in one read-only walk.
     * A matching prefix must never hide a corrupt tail. Epoch rejection still
     * protects against same-address replacements with identical descriptors. */
    uint64_t checked = 0;
    while (checked < s->copy.text.used) {
        const block *old = (const void *)(s->copy.text.data + checked);
        uint64_t end = checked + old->span;
        if (old->state & ALLOCATED) {
            if (!block_valid(a, checked)) return false;
            const block *live = (const void *)(a->data + checked);
            if (live->span != old->span || live->bytes != old->bytes ||
                live->kind != old->kind || live->state != ALLOCATED) return false;
            checked = end;
        } else {
            /* New blocks may subdivide, but cannot cross, a captured free span. */
            while (checked < end) {
                if (!block_valid(a, checked)) return false;
                const block *live = (const void *)(a->data + checked);
                if (live->span > end - checked) return false;
                checked += live->span;
            }
        }
    }
    while (checked < a->used) {
        if (!block_valid(a, checked)) return false;
        checked += ((const block *)(const void *)(a->data + checked))->span;
    }
    RECLAIM_PHASE(2); /* Validation complete, no live state changed yet. */
    /* Import captured marks once. Allocations in captured free spans and the
     * appended tail are new, so retain them without overwriting captured marks.
     * Validation above finishes before any live state is changed. */
    uint64_t at = 0;
    while (at < s->copy.text.used) {
        const block *old = (const void *)(s->copy.text.data + at);
        uint64_t end = at + old->span;
        if (old->state & ALLOCATED) {
            ((block *)(void *)(a->data + at))->state = ALLOCATED | (old->state & MARKED);
            at = end;
        } else {
            /* Allocation can split one captured free span into several blocks. */
            while (at < end) {
                block *live = (void *)(a->data + at);
                if (live->state) live->state = ALLOCATED | MARKED;
                at += live->span;
            }
        }
    }
    while (at < a->used) {
        block *live = (void *)(a->data + at);
        if (live->state) live->state = ALLOCATED | MARKED;
        at += live->span;
    }
    RECLAIM_PHASE(3);
    uint64_t reclaimed_before = statistics.reclaimed_bytes;
    experiment_invalidate(); /* Other outstanding cycles cannot reuse old identity. */
    if (experiment_finish(context, NULL, 0)) abort();
    RECLAIM_PHASE(4);
    cycle_reclaimed_bytes += statistics.reclaimed_bytes - reclaimed_before;
    return true;
}
static cycle_result cycle_poll(collection_cycle *cycle, bool wait, bool cancel) {
    cycle_owner(cycle);
    if (!cycle->job) return CYCLE_EMPTY;
    int status;
    RECLAIM_PHASE(0);
    if (!gc_worker_take(&cycle->worker, wait, cancel, &status)) return CYCLE_PENDING;
    RECLAIM_PHASE(1);
    bool applied = !cancel && status == 0 && cycle_apply(cycle);
    if (applied) cycles_applied++;
    else cycles_discarded++;
    free(cycle->job->copy.text.data);
    free(cycle->job);
    cycle->job = NULL;
    cycle->context = NULL;
    RECLAIM_PHASE(5);
    return applied ? CYCLE_APPLIED : CYCLE_DISCARDED;
}
static void cycle_destroy(collection_cycle *cycle) {
    cycle_owner(cycle);
    gc_worker_destroy(&cycle->worker); /* Cancel and join before freeing any job. */
    if (cycle->job) {
        free(cycle->job->copy.text.data);
        free(cycle->job);
        cycle->job = NULL;
    }
}
