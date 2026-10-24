/* Diagnostic-only immutable capture. Included into a generated native-gc.c.
 * The worker owns copies, never mutator TLS, frame addresses or live payloads.
 * Synchronous GC remains solely responsible for cleanup and reclamation. */
#include <pthread.h>
#include <stdlib.h>
#include <time.h>
typedef struct { uint64_t owner, value; } shadow_edge;
typedef struct {
    uintptr_t base;
    unsigned char *data, *marked, *expected;
    uint64_t used, *offsets, *pending;
    size_t count, pending_count, edge_count, edge_capacity;
    shadow_edge *edges;
    int reference_valid, valid;
    uint64_t cpu_ns;
} shadow_snapshot;
typedef struct {
    pthread_t thread;
    pthread_mutex_t mutex;
    pthread_cond_t condition;
    shadow_snapshot *job;
    int ready, done, stop;
} shadow_worker;
#ifdef NEOCLR_SHADOW_TEST
#include <sched.h>
static _Atomic int shadow_test_gate;
void experiment_gc_shadow_gate(int closed) { atomic_store(&shadow_test_gate, closed); }
#endif
static _Thread_local shadow_worker *shadow_owner;
static _Thread_local uint64_t shadow_stats[5]; /* compared, copied bytes, capture ns, worker CPU ns, invalid */
static uint64_t shadow_clock(clockid_t clock) {
    struct timespec t;
    if (clock_gettime(clock, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
static void *shadow_alloc(size_t count, size_t bytes) {
    if (bytes && count > SIZE_MAX / bytes) abort();
    void *value = calloc(count ? count : 1, bytes);
    if (!value) abort();
    return value;
}
static size_t shadow_find(const shadow_snapshot *s, uint64_t word) {
    if (word < s->base || word - s->base >= s->used) return SIZE_MAX;
    uint64_t offset = word - s->base;
    size_t low = 0, high = s->count;
    while (low < high) {
        size_t mid = low + (high - low) / 2;
        if (s->offsets[mid] <= offset) low = mid + 1;
        else high = mid;
    }
    if (!low) return SIZE_MAX;
    size_t i = low - 1;
    const block *b = (const void *)(s->data + s->offsets[i]);
    uint64_t payload = s->offsets[i] + sizeof(block);
    return (b->state & ALLOCATED) && offset >= payload && offset - payload < b->bytes ? i : SIZE_MAX;
}
static void shadow_mark(shadow_snapshot *s, uint64_t word) {
    size_t i = shadow_find(s, word);
    if (i != SIZE_MAX && !s->marked[i]) {
        s->marked[i] = 1;
        s->pending[s->pending_count++] = i;
    }
}
static void shadow_edge_add(shadow_snapshot *s, uint64_t owner, uint64_t value) {
    if (s->edge_count == s->edge_capacity) {
        size_t next = s->edge_capacity ? s->edge_capacity * 2 : 64;
        if (next < s->edge_capacity || next > SIZE_MAX / sizeof(shadow_edge)) abort();
        void *edges = realloc(s->edges, next * sizeof(shadow_edge));
        if (!edges) abort();
        s->edges = edges;
        s->edge_capacity = next;
    }
    s->edges[s->edge_count++] = (shadow_edge){owner, value};
}
static void shadow_slots(shadow_snapshot *s, uint64_t owner,
    const neoclr_probe_storage *slots, uint32_t count) {
    for (uint32_t i = 0; i < count; i++) {
        if (slots[i].read_bytes != 8) continue;
        uint64_t value;
        memcpy(&value, slots[i].address, 8);
        shadow_edge_add(s, owner, value);
    }
}
/* Generated scanner below shares the production payload semantics. */
static int shadow_scan(shadow_snapshot *s, const block *b);
static void *shadow_run(void *argument) {
    shadow_worker *w = argument;
    if (pthread_mutex_lock(&w->mutex)) abort();
    for (;;) {
        while (!w->ready && !w->stop)
            if (pthread_cond_wait(&w->condition, &w->mutex)) abort();
        if (w->stop) break;
        shadow_snapshot *s = w->job;
        w->ready = 0;
        if (pthread_mutex_unlock(&w->mutex)) abort();
        #ifdef NEOCLR_SHADOW_TEST
        while (atomic_load(&shadow_test_gate)) sched_yield();
#endif
        uint64_t start = shadow_clock(CLOCK_THREAD_CPUTIME_ID);
        s->valid = 1;
        for (;;) {
            while (s->pending_count) {
                size_t i = s->pending[--s->pending_count];
                if (!shadow_scan(s, (const void *)(s->data + s->offsets[i]))) {
                    s->valid = 0;
                    break;
                }
            }
            if (!s->valid) break;
            for (size_t i = 0; i < s->edge_count; i++) {
                size_t owner = shadow_find(s, s->edges[i].owner);
                if (owner != SIZE_MAX && s->marked[owner]) shadow_mark(s, s->edges[i].value);
            }
            if (!s->pending_count) break;
        }
        s->cpu_ns = shadow_clock(CLOCK_THREAD_CPUTIME_ID) - start;
        if (pthread_mutex_lock(&w->mutex)) abort();
        w->done = 1;
        if (pthread_cond_broadcast(&w->condition)) abort();
    }
    if (pthread_mutex_unlock(&w->mutex)) abort();
    return NULL;
}
static void shadow_drain(void) {
    shadow_worker *w = shadow_owner;
    if (!w || !w->job) return;
    if (pthread_mutex_lock(&w->mutex)) abort();
    while (!w->done)
        if (pthread_cond_wait(&w->condition, &w->mutex)) abort();
    shadow_snapshot *s = w->job;
    w->job = NULL;
    if (pthread_mutex_unlock(&w->mutex)) abort();
    if (s->reference_valid) {
        if (!s->valid || memcmp(s->marked, s->expected, s->count)) abort();
        shadow_stats[0]++;
    } else shadow_stats[4]++;
    shadow_stats[3] += s->cpu_ns;
    free(s->edges); free(s->pending); free(s->offsets);
    free(s->expected); free(s->marked); free(s->data); free(s);
}
static void shadow_stop(void) {
    shadow_worker *w = shadow_owner;
    if (!w) return;
    shadow_drain();
    if (pthread_mutex_lock(&w->mutex)) abort();
    w->stop = 1;
    if (pthread_cond_broadcast(&w->condition) || pthread_mutex_unlock(&w->mutex)) abort();
    if (pthread_join(w->thread, NULL)) abort();
    if (pthread_cond_destroy(&w->condition) || pthread_mutex_destroy(&w->mutex)) abort();
    free(w);
    shadow_owner = NULL;
}
void experiment_gc_shadow(uint64_t *output) {
    shadow_stop();
    memcpy(output, shadow_stats, sizeof(shadow_stats));
}
static void shadow_begin(neoclr_aot_context *context,
    const neoclr_gc_conditional_root *conditional, uint32_t count) {
    shadow_drain();
    uint64_t start = shadow_clock(CLOCK_MONOTONIC);
    shadow_snapshot *s = shadow_alloc(1, sizeof(*s));
    s->base = (uintptr_t)context->text.data;
    s->used = context->text.used;
    s->data = shadow_alloc(s->used, 1);
    if (s->used) memcpy(s->data, context->text.data, s->used);
    for (uint64_t at = 0; at < s->used;) {
        const block *b = (const void *)(s->data + at);
        s->count++;
        at += b->span;
    }
    s->offsets = shadow_alloc(s->count, sizeof(uint64_t));
    s->pending = shadow_alloc(s->count, sizeof(uint64_t));
    s->marked = shadow_alloc(s->count, 1);
    s->expected = shadow_alloc(s->count, 1);
    size_t n = 0;
    for (uint64_t at = 0; at < s->used;) {
        const block *b = (const void *)(s->data + at);
        s->offsets[n++] = at;
        at += b->span;
    }
    /* Production has seeded every strong root and intern before this capture.
     * Copy the seed bits, not queue links into the original arena. */
    for (size_t i = 0; i < s->count; i++) {
        const block *b = (const void *)(s->data + s->offsets[i]);
        if (b->state & MARKED) shadow_mark(s, s->base + s->offsets[i] + sizeof(block));
    }
    for (uint32_t i = 0; i < count; i++)
        shadow_edge_add(s, (uintptr_t)conditional[i].owner, (uintptr_t)conditional[i].value);
    for (host_root *r = host_head; r; r = r->next)
        if (r->context == context && r->conditional && !r->expired)
            shadow_edge_add(s, (uintptr_t)r->owner, (uintptr_t)r->value);
    for (const neoclr_parked_roots *p = neoclr_roots_parked_v1(); p; p = p->next) {
        if (p->owner_context != context || p->expired) continue;
        for (const neoclr_probe_frame *f = p->head; f; f = f->previous) {
            if (f->context != context) continue;
            shadow_slots(s, (uintptr_t)p->owner, f->storage, f->storage_count);
            shadow_slots(s, (uintptr_t)p->owner, f->transient, f->transient_count);
            for (uint32_t i = 0; i < f->lane_count; i++)
                shadow_edge_add(s, (uintptr_t)p->owner, f->lanes[i]);
        }
    }
    if (!shadow_owner) {
        shadow_owner = shadow_alloc(1, sizeof(*shadow_owner));
        if (pthread_mutex_init(&shadow_owner->mutex, NULL) ||
            pthread_cond_init(&shadow_owner->condition, NULL) ||
            pthread_create(&shadow_owner->thread, NULL, shadow_run, shadow_owner)) abort();
    }
    shadow_worker *w = shadow_owner;
    if (pthread_mutex_lock(&w->mutex)) abort();
    w->job = s;
    w->done = 0;
    w->ready = 1;
    if (pthread_cond_broadcast(&w->condition) || pthread_mutex_unlock(&w->mutex)) abort();
    shadow_stats[1] += s->used;
    shadow_stats[2] += shadow_clock(CLOCK_MONOTONIC) - start;
}
static void shadow_reference(neoclr_aot_context *context) {
    shadow_snapshot *s = shadow_owner->job;
    for (size_t i = 0; i < s->count; i++) {
        const block *b = (const void *)(context->text.data + s->offsets[i]);
        s->expected[i] = !!(b->state & MARKED);
    }
    s->reference_valid = 1;
}
