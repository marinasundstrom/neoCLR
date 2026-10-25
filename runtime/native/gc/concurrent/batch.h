#ifdef NEOCLR_LIVE_BATCH
/* One heap gate, held for bounded work. No callbacks or allocations here. */
static void live_batch_leave(shadow_snapshot *s) {
    if (!s->read_held) return;
    if (s->read_bytes > s->max_read_bytes) s->max_read_bytes=s->read_bytes;
    if (s->read_calls > s->max_read_calls) s->max_read_calls=s->read_calls;
#ifdef NEOCLR_LIVE_GATE_PROFILE
    uint64_t elapsed=shadow_clock(CLOCK_MONOTONIC)-s->hold_start;
    s->hold_total+=elapsed;
    if (elapsed>s->hold_max) s->hold_max=elapsed;
#endif
    s->read_held=0;
    if (pthread_mutex_unlock(&s->gate->mutex)) abort();
}
static int live_batch_enter(shadow_snapshot *s) {
    if (s->read_held) return 1;
    if (pthread_mutex_lock(&s->gate->mutex)) abort();
    s->read_held=1;
    s->read_bytes=s->read_calls=0;
    s->read_acquisitions++;
#ifdef NEOCLR_LIVE_GATE_PROFILE
    s->hold_start=shadow_clock(CLOCK_MONOTONIC);
#endif
    if (!s->gate->active || s->gate->epoch!=s->epoch || s->gate->cancelled) {
        live_batch_leave(s);
        return 0;
    }
    return 1;
}
static int live_batch_drain(shadow_snapshot *s) {
    uint64_t words[GATE_WORDS]; size_t count;
    if (gate_drain(s->gate,s->epoch,words,GATE_WORDS,&count)) return 0;
    s->barrier_words+=count;
    for (size_t i=0;i<count;i++) shadow_mark(s,words[i]);
    return 1;
}
#endif
