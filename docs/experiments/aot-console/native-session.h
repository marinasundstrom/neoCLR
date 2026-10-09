#ifndef NEOCLR_NATIVE_SESSION_H
#define NEOCLR_NATIVE_SESSION_H
#include "native-gc.h"
/* Private, single-image, thread-affine experiment. Caller owns the context,
 * buffer and code lifetime. Zero initialize; do not copy or mutate this storage.
 * Open after bootstrap returns. An anchor prevents entry reset even when empty.
 * Retain accepts live guest allocation bases, never borrowed stack pointers.
 * A guest fault poisons dispatch until close; mutations are not rolled back.
 * Read/render diagnostics before close, which discards state and fault storage.
 * Close rejects active frames, callbacks and roots owned by other services. */
#define NEOCLR_SESSION_ROOT_LIMIT 16
typedef struct neoclr_native_session {
    struct neoclr_native_session *owner;
    neoclr_aot_context *context;
    uint64_t roots[NEOCLR_SESSION_ROOT_LIMIT + 1];
    uint32_t count, phase;
} neoclr_native_session;
int32_t neoclr_session_open_v1(neoclr_native_session *, neoclr_aot_context *);
int32_t neoclr_session_retain_v1(neoclr_native_session *, void *, uint64_t *);
int32_t neoclr_session_invoke_v1(neoclr_native_session *, uint64_t);
int32_t neoclr_session_collect_v1(neoclr_native_session *);
int32_t neoclr_session_close_v1(neoclr_native_session *);
#endif
