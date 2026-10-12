#ifndef NEOCLR_TLS_MANAGED_H
#define NEOCLR_TLS_MANAGED_H
#include "native-gc.h"
/* Private adapter operations matching runtime/neoCLR/Runtime/Tls.neoil.
 * Array arguments are exact live packed Byte[] allocations in context.
 * Owner is an empty Byte[]. No managed allocation/collection, I/O or callbacks.
 * Return is native Fault status; result is provider status and changes only on
 * non-Fault completion. Operation/configuration/session owners remain guest-rooted
 * across calls; the adapter's associations never root them. Thread-affine. */
enum { NEOCLR_TLS_CLIENT = 1, NEOCLR_TLS_SERVER, NEOCLR_TLS_SESSION,
       NEOCLR_TLS_RELEASE, NEOCLR_TLS_ABORT, NEOCLR_TLS_PUMP };
int32_t neoclr_tls_managed_v1(int32_t operation, neoclr_aot_context *,
    void *owner, void *first, void *second, void *third, int32_t *result);
int32_t neoclr_tls_managed_v2(int32_t operation, neoclr_aot_context *,
    void *owner, void *first, void *second, void *third, void *fourth, int32_t *result);
/* Host-only abortive teardown, after all guest execution/operations have stopped.
 * Release before destroying context/arena, including terminal Fault exits.
 * Does not clear Fault state, collect, or close an underlying transport. */
void neoclr_tls_managed_dispose_v1(neoclr_aot_context *);
#endif
