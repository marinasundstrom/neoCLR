/* Explicit experimental lifecycle; owner-thread use only. No collection or arena
 * release is permitted between begin and finish. Allocation/mutation is allowed.
 * A failed finish discards the snapshot and leaves synchronous recovery available. */
static _Thread_local int candidate_phase;
static _Thread_local neoclr_aot_context *candidate_context;
static void candidate_wait(void) {
    shadow_worker *w = shadow_owner;
    if (!w || !w->job) abort();
    if (pthread_mutex_lock(&w->mutex)) abort();
    while (!w->done)
        if (pthread_cond_wait(&w->condition, &w->mutex)) abort();
    if (pthread_mutex_unlock(&w->mutex)) abort();
}
static void candidate_dispose(void) {
    candidate_wait();
    shadow_worker *w = shadow_owner;
    if (pthread_mutex_lock(&w->mutex)) abort();
    shadow_snapshot *s = w->job;
    w->job = NULL;
    if (pthread_mutex_unlock(&w->mutex)) abort();
#ifdef NEOCLR_SHADOW_SPARSE_INDEX
    free(s->starts);
#endif
    free(s->edges); free(s->pending); free(s->offsets);
    free(s->expected); free(s->marked); free(s->data); free(s);
}
static int candidate_seed(neoclr_aot_context *context, block_index *index, block **pending) {
    shadow_snapshot *s = shadow_owner->job;
    neoclr_aot_text_arena *a = &context->text;
    if (!s->valid || (uintptr_t)a->data != s->base || a->used < s->used) return 3;
    /* Old allocated blocks cannot move, split or disappear before remark.
     * Validate that proof before introducing any snapshot-derived mark. */
    for (size_t i = 0; i < s->count; i++) {
        const block *old = (const void *)(s->data + s->offsets[i]);
        if (!(old->state & ALLOCATED)) continue;
        block *now = indexed_block(a, index, s->base + s->offsets[i]);
        if (!now || (unsigned char *)now != a->data + s->offsets[i] ||
            !(now->state & ALLOCATED) || now->span != old->span ||
            now->bytes != old->bytes || now->kind != old->kind) return 3;
    }
    size_t previous = 0;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        while (previous < s->count && s->offsets[previous] < at) previous++;
        const block *old = previous < s->count && s->offsets[previous] == at
            ? (const void *)(s->data + at) : NULL;
        if (b->state & ALLOCATED) {
            int existed = old && (old->state & ALLOCATED);
            int changed = !existed || memcmp(b + 1, old + 1, b->bytes);
            if (changed) {
                /* Conservatively retain all new/changed objects and their current
                 * closure. This also covers native writes and weak-root exposure. */
                mark(a, index, (uintptr_t)(b + 1), pending);
            } else if (s->marked[previous] && !(b->state & MARKED)) {
                /* Its old closure is already in the immutable snapshot marks.
                 * A root already on the current worklist is left queued. */
                b->state = ALLOCATED | MARKED | SCANNED;
            }
        }
        at += b->span;
    }
    return 0;
}
