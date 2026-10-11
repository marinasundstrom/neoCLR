#ifndef NEOCLR_RUNTIME_ASYNC_EXPERIMENT_H
#define NEOCLR_RUNTIME_ASYNC_EXPERIMENT_H
#include "native-gc.h"
typedef int32_t (*neoclr_async_callback)(uint64_t, neoclr_aot_context *);
typedef int32_t (*neoclr_async_drain)(neoclr_aot_context *);
int32_t neoclr_runtime_async_create_v1(neoclr_aot_context *, uint64_t *);
int32_t neoclr_runtime_async_start_v1(void *, neoclr_aot_context *, uint64_t *, neoclr_async_callback);
/* Private owner-bound carrier experiment, not a managed intrinsic yet.
 * Caller must keep owner reachable for every consumer that needs the body,
 * including pending result/advancement tasks and external completion callbacks.
 * The invoke callback may use its handle for initial admission only; it must not
 * read/release that handle after suspension. The generated dispatcher follows this.
 * Running bodies have strong owner/callback handles; parked bodies use dependent
 * roots. Expired bodies never resume. Quiescent reap (also pump/shutdown) releases
 * their stack, callback handle and body-operation slot, without managed cleanup.
 * Awaited external signal operations remain separately owned. Native ids alone
 * do not retain managed owners. A completed body's normal release is unchanged. */
int32_t neoclr_runtime_async_start_owned_v1(void *owner, void *callback,
    neoclr_aot_context *, uint64_t *, neoclr_async_callback);
int32_t neoclr_runtime_async_startmanaged_v1(void *, void *, neoclr_aot_context *, uint64_t *, neoclr_async_callback);
int32_t neoclr_runtime_async_currentowner_v1(neoclr_aot_context *, void **);
int32_t neoclr_runtime_async_createowned_v1(void *, neoclr_aot_context *, uint64_t *);
int32_t neoclr_runtime_async_reap_v1(neoclr_aot_context *);
int32_t neoclr_runtime_async_completeunit_v1(uint64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_runtime_async_awaitunit_v1(uint64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_runtime_async_release_v1(uint64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_runtime_async_iscompleted_v1(uint64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_runtime_async_waitcompletion_v1(uint64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_runtime_async_pump_v1(neoclr_aot_context *, neoclr_async_callback, neoclr_async_drain);
int32_t neoclr_runtime_async_shutdown_v1(neoclr_aot_context *);
/* Private cumulative thread-local diagnostics. Peak reservation is a process-thread
 * high-water mark, including guard pages and cached stacks; it is not resident bytes. */
typedef struct {
    uint64_t fibers, resumes, mappings, unmappings, reserved_bytes, peak_reserved_bytes;
} neoclr_async_statistics;
neoclr_async_statistics neoclr_runtime_async_statistics_v1(void);
#endif
