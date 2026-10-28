#ifndef NEOCLR_READY_QUEUE_H
#define NEOCLR_READY_QUEUE_H
#include <stdint.h>
#include <stddef.h>
/* Shared bounded runnable storage. Entries are opaque scheduler identities, never
 * unrooted managed pointers. The owner supplies storage and serializes access
 * (owner-thread affinity or an existing worker gate). This service owns ordering,
 * admission and removal, not execution affinity, root lifetime or OS readiness.
 * FIFO is the normal policy; pop-newest supports the existing worker experiment.
 * Failed operations preserve queue and output. Removed entries are cleared. */
typedef struct { uint64_t identity, order; } neoclr_ready_item;
typedef struct { uint64_t admitted, rejected, dispatched, removed; unsigned peak; } neoclr_ready_statistics;
typedef struct {
    neoclr_ready_item *items;
    unsigned capacity, head, count;
#ifdef NEOCLR_SCHEDULER_DIAGNOSTICS
    neoclr_ready_statistics statistics;
#endif
} neoclr_ready_queue;
static inline void neoclr_ready_init(neoclr_ready_queue *q,neoclr_ready_item *items,unsigned capacity) {
    *q=(neoclr_ready_queue){.items=items,.capacity=capacity};
}
static inline const neoclr_ready_item *neoclr_ready_first(const neoclr_ready_queue *q) {
    return q->count ? &q->items[q->head] : NULL;
}
static inline int neoclr_ready_push(neoclr_ready_queue *q,neoclr_ready_item item) {
    if (!item.identity || q->count==q->capacity) {
#ifdef NEOCLR_SCHEDULER_DIAGNOSTICS
        if (q->statistics.rejected!=UINT64_MAX) q->statistics.rejected++;
#endif
        return 0;
    }
    q->items[(q->head+q->count)%q->capacity]=item;
    q->count++;
#ifdef NEOCLR_SCHEDULER_DIAGNOSTICS
    if (q->statistics.admitted!=UINT64_MAX) q->statistics.admitted++;
    if (q->count>q->statistics.peak) q->statistics.peak=q->count;
#endif
    return 1;
}
static inline int neoclr_ready_pop(neoclr_ready_queue *q,int newest,neoclr_ready_item *out) {
    if (!q->count || !out) return 0;
    unsigned at=(q->head+(newest?q->count-1:0))%q->capacity;
    *out=q->items[at]; q->items[at]=(neoclr_ready_item){0};
    if (!newest) q->head=(q->head+1)%q->capacity;
    q->count--;
#ifdef NEOCLR_SCHEDULER_DIAGNOSTICS
    if (q->statistics.dispatched!=UINT64_MAX) q->statistics.dispatched++;
#endif
    return 1;
}
static inline int neoclr_ready_remove(neoclr_ready_queue *q,uint64_t identity) {
    unsigned at=0;
    while (at<q->count && q->items[(q->head+at)%q->capacity].identity!=identity) at++;
    if (at==q->count) return 0;
    for (;at+1<q->count;at++)
        q->items[(q->head+at)%q->capacity]=q->items[(q->head+at+1)%q->capacity];
    q->items[(q->head+q->count-1)%q->capacity]=(neoclr_ready_item){0};
    q->count--;
#ifdef NEOCLR_SCHEDULER_DIAGNOSTICS
    if (q->statistics.removed!=UINT64_MAX) q->statistics.removed++;
#endif
    return 1;
}
static inline neoclr_ready_statistics neoclr_ready_stats(const neoclr_ready_queue *q) {
#ifdef NEOCLR_SCHEDULER_DIAGNOSTICS
    return q->statistics;
#else
    (void)q; return (neoclr_ready_statistics){0};
#endif
}
#endif
