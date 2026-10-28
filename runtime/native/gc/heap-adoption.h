#ifndef NEOCLR_PRIVATE_HEAP_ADOPTION_H
#define NEOCLR_PRIVATE_HEAP_ADOPTION_H
#include "native-gc.h"
#ifdef NEOCLR_EXCLUSIVE_HEAP_ADOPTION
/* Private exclusive lease, not general cross-thread heap sharing. Caller holds
 * the heap gate, proves the previous owner has released/joined its collector,
 * and supplies all roots before any collection. No foreign TLS roots/resources
 * may refer to this heap. Current thread must have no heap, frames or resources.
 * Validate before mutation; reconstruct free-list/cache/index from descriptors.
 * Release with heap_release on this thread BEFORE withdrawing roots, destroying
 * the invocation context or unlocking. Concurrent release joins/cancels marking.
 * Native state is not transferred and this does not admit async/socket services. */
/* Compiler image must match this collector before generated callbacks run. */
int32_t neoclr_gc_heap_callback_contract_v1(void);
int32_t neoclr_gc_heap_adopt_exclusive_v1(neoclr_aot_text_arena *);
#endif
#endif
