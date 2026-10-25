#include "native-gc.h"
#if defined(NEOCLR_LIVE_GROUP_WORKER_TLS) && defined(NEOCLR_LIVE_SPLIT_TLS)
#error Grouped worker TLS requires grouped candidate state
#endif
/* Private owner-thread state only. The marker receives its snapshot explicitly;
 * it must never discover an owner's state through its own TLS. Heap ownership
 * and allocation proofs remain in the existing owned_heap service. */
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE)
#ifndef NEOCLR_LIVE_STORE_RANGES
#define NEOCLR_LIVE_STORE_RANGES 4
#endif
#if NEOCLR_LIVE_STORE_RANGES < 1 || NEOCLR_LIVE_STORE_RANGES > 16
#error "store range count must be between 1 and 16"
#endif
#endif
#ifdef NEOCLR_LIVE_SPLIT_TLS
static _Thread_local int candidate_active;
static _Thread_local neoclr_aot_context *candidate_context;
static _Thread_local uint64_t live_store_counts[5];
#else
typedef struct live_owner_state {
#ifdef NEOCLR_LIVE_GROUP_WORKER_TLS
    shadow_worker *worker; /* Owner handle; marker still receives its argument explicitly. */
#endif
    neoclr_aot_context *context;
    int active;
    uint64_t stores[5]; /* outside, inactive/new, guarded old, cancelled, rejected */
#ifndef NEOCLR_LIVE_SPLIT_STORE_TLS
    struct {
        uint64_t lookups[2];
#ifdef NEOCLR_LIVE_WALK_PROFILE
        uint64_t walks[3];
#endif
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE)
#ifdef NEOCLR_LIVE_HEADER_PROOF
        struct { const void *payload; uint64_t span, bytes, kind; } header;
#else
        const void *payload;
#endif
#if NEOCLR_LIVE_STORE_RANGES > 1
        const void *recent[NEOCLR_LIVE_STORE_RANGES-1];
        unsigned next;
#endif
#endif
    } store;
#endif
} live_owner_state;
static _Thread_local live_owner_state live_owner;
#ifdef NEOCLR_LIVE_GROUP_WORKER_TLS
#define shadow_owner (live_owner.worker)
#endif
#define candidate_active (live_owner.active)
#define candidate_context (live_owner.context)
#define live_store_counts (live_owner.stores)
#endif
