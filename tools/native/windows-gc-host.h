#ifndef NEOCLR_WINDOWS_GC_HOST_H
#define NEOCLR_WINDOWS_GC_HOST_H
#include "windows-host-memory.h"
#include "../../docs/experiments/aot-console/native-gc.h"
/* Private trusted-host adapter. Zero-initialize; never copy or mutate ownership.
 * Creating thread must stay alive. Raw collector calls remain thread-affine.
 * This binds existing GC/root contracts to Windows-owned memory; no guest entry,
 * scheduler, migration or native stack contract is introduced. */
typedef struct {
    neoclr_windows_heap heap;
    neoclr_aot_context context;
    uint32_t thread_id;
} neoclr_windows_gc_host;
/* Admission returns 9 without allocating on an undersized/unsupported stack. */
int32_t neoclr_windows_gc_host_open(neoclr_windows_gc_host *host, size_t capacity);
/* Rejects foreign thread, live host handles and active published guest frames.
 * Render any fault first. On success all context storage becomes invalid.
 * No suspension may retain unregistered references; no concurrent host access. */
int32_t neoclr_windows_gc_host_close(neoclr_windows_gc_host *host);
#endif
