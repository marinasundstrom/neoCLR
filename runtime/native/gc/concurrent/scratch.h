/* Experimental owner-local scratch storage. Only detached, completed jobs return
 * buffers. No gate, heap payload, or cross-owner pointer is cached here. */
#ifndef NEOCLR_LIVE_SCRATCH_LIMIT
#define NEOCLR_LIVE_SCRATCH_LIMIT (1024 * 1024)
#endif
#ifdef NEOCLR_LIVE_REUSE_SCRATCH
typedef struct { void *data, *active; size_t capacity; int leased; } live_scratch_slot;
static _Thread_local live_scratch_slot live_scratch[6];
static _Thread_local size_t live_scratch_retained;
#endif
static void *live_scratch_alloc(unsigned slot, size_t count, size_t bytes) {
#ifndef NEOCLR_LIVE_REUSE_SCRATCH
    (void)slot;
    return shadow_alloc(count,bytes);
#else
    if (slot>=6 || !bytes || (count && count>SIZE_MAX/bytes)) abort();
    size_t needed=(count?count:1)*bytes;
    live_scratch_slot *s=&live_scratch[slot];
    if (s->leased) abort();
    if (needed>s->capacity) {
        /* Oversized jobs use disposable storage; keep the bounded small cache. */
        size_t other=live_scratch_retained-s->capacity;
        if (needed>NEOCLR_LIVE_SCRATCH_LIMIT || other>NEOCLR_LIVE_SCRATCH_LIMIT-needed)
        {
            s->leased=1;
            s->active=shadow_alloc(count,bytes);
            return s->active;
        }
        free(s->data);
        s->data=shadow_alloc(count,bytes);
        live_scratch_retained=other+needed;
        s->capacity=needed;
    } else memset(s->data,0,needed);
    s->leased=1;
    s->active=s->data;
    return s->data;
#endif
}
static void live_scratch_free(unsigned slot, void *data) {
#ifdef NEOCLR_LIVE_REUSE_SCRATCH
    if (slot>=6) abort();
    live_scratch_slot *s=&live_scratch[slot];
    if (!s->leased || data!=s->active) abort();
    s->leased=0;
    s->active=NULL;
    if (data==s->data) return;
#else
    (void)slot;
#endif
    free(data);
}
static void live_scratch_stop(void) {
#ifdef NEOCLR_LIVE_REUSE_SCRATCH
    for (unsigned i=0;i<6;i++) {
        if (live_scratch[i].leased) abort();
        free(live_scratch[i].data);
    }
    memset(live_scratch,0,sizeof(live_scratch));
    live_scratch_retained=0;
#endif
}
