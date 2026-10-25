#include "native-gc.h"
/* Private owner-thread state only. The marker receives its snapshot explicitly;
 * it must never discover an owner's state through its own TLS. Heap ownership
 * and allocation proofs remain in the existing owned_heap service. */
#ifdef NEOCLR_LIVE_SPLIT_TLS
static _Thread_local int candidate_active;
static _Thread_local neoclr_aot_context *candidate_context;
static _Thread_local uint64_t live_store_counts[5];
#else
typedef struct live_owner_state {
    neoclr_aot_context *context;
    int active;
    uint64_t stores[5]; /* outside, inactive/new, guarded old, cancelled, rejected */
} live_owner_state;
static _Thread_local live_owner_state live_owner;
#define candidate_active (live_owner.active)
#define candidate_context (live_owner.context)
#define live_store_counts (live_owner.stores)
#endif
