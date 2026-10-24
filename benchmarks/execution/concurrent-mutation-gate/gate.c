#include "gate.h"
#include <string.h>

static int owner(mutation_gate *g) { return pthread_equal(g->owner, pthread_self()); }
static int state(mutation_gate *g, uint64_t epoch) {
    if (!g->active || epoch != g->epoch) return GATE_STALE;
    return g->cancelled ? GATE_CANCELLED : GATE_OK;
}
int gate_init(mutation_gate *g) {
    memset(g, 0, sizeof(*g));
    g->owner = pthread_self();
    return pthread_mutex_init(&g->mutex, NULL) ? GATE_INVALID : GATE_OK;
}
int gate_begin(mutation_gate *g, uint64_t *epoch) {
    if (!owner(g) || !epoch) return GATE_INVALID;
    pthread_mutex_lock(&g->mutex);
    int result = GATE_INVALID;
    if (!g->active && g->epoch != UINT64_MAX) {
        g->active = 1;
        g->cancelled = 0;
        g->count = 0;
        *epoch = ++g->epoch;
        result = GATE_OK;
    }
    pthread_mutex_unlock(&g->mutex);
    return result;
}
int gate_stop(mutation_gate *g) {
    if (!owner(g)) return GATE_INVALID;
    pthread_mutex_lock(&g->mutex);
    int result = !g->active ? GATE_STALE : g->cancelled ? GATE_CANCELLED : GATE_OK;
    g->active = 0;
    pthread_mutex_unlock(&g->mutex);
    return result;
}
int gate_destroy(mutation_gate *g) {
    if (!owner(g) || g->active) return GATE_INVALID;
    return pthread_mutex_destroy(&g->mutex) ? GATE_INVALID : GATE_OK;
}
static void bounds(size_t bytes, size_t scan_start, const mutation_patch *p,
                   size_t *first, size_t *end) {
    *first = p->offset & ~(size_t)7;
    if (*first < scan_start) *first = scan_start;
    size_t limit = bytes & ~(size_t)7;
    size_t last = p->offset + p->length;
    /* Avoid rounding overflow even for a deliberately invalid enormous span. */
    *end = last > limit ? limit : last + ((8 - (last & 7)) & 7);
    if (!p->length || *end < *first) *end = *first;
}
int gate_write(mutation_gate *g, void *payload, size_t bytes, size_t scan_start,
               const mutation_patch *patches, size_t count) {
    if (!owner(g) || !payload || scan_start > bytes || (scan_start & 7) ||
        count > GATE_PATCHES || (count && !patches)) return GATE_INVALID;
    size_t needed = 0;
    for (size_t i = 0; i < count; i++) {
        const mutation_patch *p = &patches[i];
        if (p->offset > bytes || p->length > bytes - p->offset ||
            (p->length && !p->source)) return GATE_INVALID;
        size_t first, end;
        bounds(bytes, scan_start, p, &first, &end);
        size_t words = (end - first) / 8;
        if (words > GATE_WORDS || needed > GATE_WORDS - words) needed = GATE_WORDS + 1;
        else needed += words;
    }
    /* Only the owner changes active. gate_stop has already waited for any live
     * payload copy to leave the gate; stale marker calls cannot read payloads.
     * Thus inactive writes need no mutex. Begin/stop cannot race this owner. */
    if (!g->active) {
        for (size_t i = 0; i < count; i++)
            if (patches[i].length)
                memmove((unsigned char *)payload + patches[i].offset, patches[i].source, patches[i].length);
        return GATE_OK;
    }
    pthread_mutex_lock(&g->mutex);
    int result = GATE_OK;
    if (g->active && (g->cancelled || needed > (GATE_WORDS - g->count) / 2)) {
        g->cancelled = 1;
        result = GATE_CANCELLED;
    } else {
        /* Duplicate touched lanes are allowed: reservation accounts for both. */
        if (g->active) for (size_t i = 0; i < count; i++) {
            size_t first, end;
            bounds(bytes, scan_start, &patches[i], &first, &end);
            for (size_t at = first; at < end; at += 8)
                memcpy(&g->words[g->count++], (unsigned char *)payload + at, 8);
        }
        for (size_t i = 0; i < count; i++)
            if (patches[i].length)
                memmove((unsigned char *)payload + patches[i].offset, patches[i].source, patches[i].length);
        if (g->active) for (size_t i = 0; i < count; i++) {
            size_t first, end;
            bounds(bytes, scan_start, &patches[i], &first, &end);
            for (size_t at = first; at < end; at += 8)
                memcpy(&g->words[g->count++], (unsigned char *)payload + at, 8);
        }
    }
    pthread_mutex_unlock(&g->mutex);
    return result;
}
int gate_read(mutation_gate *g, uint64_t epoch, const void *payload, size_t bytes,
              size_t offset, void *scratch, size_t length) {
    if (!payload || !scratch || length > GATE_SCRATCH || offset > bytes || length > bytes-offset)
        return GATE_INVALID;
    pthread_mutex_lock(&g->mutex);
    int result = state(g, epoch);
    if (!result && length) memcpy(scratch, (const unsigned char *)payload + offset, length);
    pthread_mutex_unlock(&g->mutex);
    return result;
}
int gate_drain(mutation_gate *g, uint64_t epoch, uint64_t *output,
               size_t capacity, size_t *count) {
    if (!count || (capacity && !output)) return GATE_INVALID;
    pthread_mutex_lock(&g->mutex);
    int result = state(g, epoch);
    if (!result) {
        *count = capacity < g->count ? capacity : g->count;
        if (*count) memcpy(output, g->words, *count * 8);
        g->count -= *count;
        memmove(g->words, g->words + *count, g->count * 8);
    }
    pthread_mutex_unlock(&g->mutex);
    return result;
}
