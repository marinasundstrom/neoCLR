#ifndef NEOCLR_RUNTIME_ASYNC_EXPERIMENT_H
#define NEOCLR_RUNTIME_ASYNC_EXPERIMENT_H
#include "native-gc.h"
typedef int32_t (*neoclr_async_callback)(uint64_t, neoclr_aot_context *);
typedef int32_t (*neoclr_async_drain)(neoclr_aot_context *);
int32_t neoclr_runtime_async_create_v1(neoclr_aot_context *, uint64_t *);
int32_t neoclr_runtime_async_start_v1(void *, neoclr_aot_context *, uint64_t *, neoclr_async_callback);
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
