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
#ifdef NEOCLR_LIVE_FINISH_WATERMARK
        g->finish_requested = 0;
#endif
#ifdef NEOCLR_GATE_WORKERS
        for (unsigned i = 0; i < 16; i++) g->writers[i].admitted = 0;
#endif
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
/* Retain the owner-only control verbatim while qualifying shared writers. */
#ifndef NEOCLR_GATE_WORKERS
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
#ifdef NEOCLR_LIVE_FINISH_WATERMARK
    if (result==GATE_OK && g->count >= (GATE_WORDS * NEOCLR_LIVE_FINISH_WATERMARK + 99) / 100)
        g->finish_requested=1;
#endif
    pthread_mutex_unlock(&g->mutex);
    return result;
}
#else
static int patch_words(void *payload, size_t bytes, size_t scan_start,
                       const mutation_patch *patches, size_t count, size_t *out) {
    if (!payload || scan_start > bytes || (scan_start & 7) ||
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
    *out = needed;
    return GATE_OK;
}
/* Caller holds the gate; registered writers must first validate active authority. */
static int write_locked(mutation_gate *g, void *payload, size_t bytes, size_t scan_start,
                        const mutation_patch *patches, size_t count, size_t needed) {
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
    return result;
}
int gate_write(mutation_gate *g, void *payload, size_t bytes, size_t scan_start,
               const mutation_patch *patches, size_t count) {
    size_t needed;
    if (!owner(g) || patch_words(payload, bytes, scan_start, patches, count, &needed))
        return GATE_INVALID;
    /* Only this owner changes active. Registered writers never use this path;
     * stop waits for their in-gate operations and makes later tokens stale. */
    if (!g->active) {
        for (size_t i = 0; i < count; i++)
            if (patches[i].length)
                memmove((unsigned char *)payload + patches[i].offset, patches[i].source, patches[i].length);
        return GATE_OK;
    }
    pthread_mutex_lock(&g->mutex);
    int result = write_locked(g, payload, bytes, scan_start, patches, count, needed);
    pthread_mutex_unlock(&g->mutex);
    return result;
}
static int writer_state(mutation_gate *g, const gate_writer *writer) {
    if (!writer || writer->gate != g || writer->slot >= 16) return GATE_INVALID;
    int result = state(g, writer->epoch);
    if (result) return result;
    if (!g->writers[writer->slot].admitted ||
        g->writers[writer->slot].generation != writer->generation) return GATE_STALE;
    return GATE_OK;
}
int gate_register_writer(mutation_gate *g, pthread_t thread, gate_writer *out) {
    if (!owner(g) || !out || pthread_equal(thread, g->owner)) return GATE_INVALID;
    pthread_mutex_lock(&g->mutex);
    int result = state(g, g->epoch);
    if (!result) {
        result = GATE_INVALID;
        unsigned slot = 16;
        for (unsigned i = 0; i < 16; i++) {
            if (g->writers[i].admitted && pthread_equal(thread, g->writers[i].thread)) goto done;
            if (!g->writers[i].admitted && g->writers[i].generation != UINT64_MAX && slot == 16) slot = i;
        }
        if (slot != 16) {
            g->writers[slot].thread = thread;
            g->writers[slot].admitted = 1;
            *out = (gate_writer){g, g->epoch, ++g->writers[slot].generation, slot};
            result = GATE_OK;
        }
    }
done:
    pthread_mutex_unlock(&g->mutex);
    return result;
}
int gate_revoke_writer(mutation_gate *g, const gate_writer *writer) {
    if (!owner(g)) return GATE_INVALID;
    pthread_mutex_lock(&g->mutex);
    int result = writer_state(g, writer);
    /* Cancellation has already forbidden writes. Stop invalidates all entries. */
    if (!result) g->writers[writer->slot].admitted = 0;
    pthread_mutex_unlock(&g->mutex);
    return result;
}
int gate_write_registered(mutation_gate *g, const gate_writer *writer, void *payload,
                          size_t bytes, size_t scan_start, const mutation_patch *patches, size_t count) {
    size_t needed;
    if (patch_words(payload, bytes, scan_start, patches, count, &needed)) return GATE_INVALID;
    pthread_mutex_lock(&g->mutex);
    int result = writer_state(g, writer);
    if (!result && !pthread_equal(g->writers[writer->slot].thread, pthread_self())) result = GATE_INVALID;
    if (!result) result = write_locked(g, payload, bytes, scan_start, patches, count, needed);
    pthread_mutex_unlock(&g->mutex);
    return result;
}
#endif
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
