/* Owner-only POC coordinator. Background tracing never touches the live heap. */
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
    if ((uintptr_t)a->data != s->source || a->used < s->copy.text.used || !blocks_valid(a)) return false;
    /* Verify original allocated descriptors before changing any live state.
     * An intervening collection/teardown is rejected by the epoch even if its
     * replacement objects have identical descriptors at identical addresses. */
    for (uint64_t at = 0; at < s->copy.text.used;) {
        const block *old = (const void *)(s->copy.text.data + at);
        if (old->state & ALLOCATED) {
            const block *live = (const void *)(a->data + at);
            if (live->span != old->span || live->bytes != old->bytes ||
                live->kind != old->kind || live->state != ALLOCATED) return false;
        }
        at += old->span;
    }
    /* New allocations are black for this cycle, including allocations in free
     * spans that existed at capture. Only captured allocations can be reclaimed. */
    for (uint64_t at = 0; at < a->used;) {
        block *live = (void *)(a->data + at);
        if (live->state) live->state = ALLOCATED | MARKED;
        at += live->span;
    }
    for (uint64_t at = 0; at < s->copy.text.used;) {
        const block *old = (const void *)(s->copy.text.data + at);
        if (old->state & ALLOCATED)
            ((block *)(void *)(a->data + at))->state = ALLOCATED | (old->state & MARKED);
        at += old->span;
    }
    uint64_t reclaimed_before = statistics.reclaimed_bytes;
    experiment_invalidate(); /* Other outstanding cycles cannot reuse old identity. */
    if (experiment_finish(context, NULL, 0)) abort();
    cycle_reclaimed_bytes += statistics.reclaimed_bytes - reclaimed_before;
    return true;
}
static cycle_result cycle_poll(collection_cycle *cycle, bool wait, bool cancel) {
    cycle_owner(cycle);
    if (!cycle->job) return CYCLE_EMPTY;
    int status;
    if (!gc_worker_take(&cycle->worker, wait, cancel, &status)) return CYCLE_PENDING;
    bool applied = !cancel && status == 0 && cycle_apply(cycle);
    if (applied) cycles_applied++;
    else cycles_discarded++;
    free(cycle->job->copy.text.data);
    free(cycle->job);
    cycle->job = NULL;
    cycle->context = NULL;
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
