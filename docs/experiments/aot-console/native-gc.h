#ifndef NEOCLR_NATIVE_GC_H
#define NEOCLR_NATIVE_GC_H
#include "text-arena.h"
#include "root-probe.h"
/* Private nonmoving heap in the caller buffer. No malloc, external runtime or
 * arbitrary native-stack scan. All entry/arena lifetime restrictions still apply. */
enum { NEOCLR_GC_TEXT = 1, NEOCLR_GC_OBJECT = 2, NEOCLR_GC_BYTES = 3, NEOCLR_GC_STRINGS = 4, NEOCLR_GC_RECORDS = 5, NEOCLR_GC_INTERN = 6 };
typedef struct {
    uint64_t collections, allocations, reclaimed_allocations, reclaimed_bytes;
} neoclr_gc_statistics;
/* Private thread-affine strong handles for future host-held callbacks/buffers.
 * Context, buffer and creating thread must remain alive until every handle is released.
 * Only null or an untagged live allocation base in that context is admitted.
 * No allocation/collection/reentry occurs in these operations; outputs are atomic on failure.
 * Limits are per native thread, shared across contexts. Handles are never reused.
 * This does not permit cross-thread context use or extend an entry/arena lifetime. */
#define NEOCLR_GC_HOST_ROOT_LIMIT 256
int32_t neoclr_gc_host_root_create_v1(neoclr_aot_context *context, void *value, uint64_t *output);
int32_t neoclr_gc_host_root_replace_v1(neoclr_aot_context *context, uint64_t handle, void *value);
int32_t neoclr_gc_host_root_read_v1(neoclr_aot_context *context, uint64_t handle, void **output);
int32_t neoclr_gc_host_root_release_v1(neoclr_aot_context *context, uint64_t handle);
/* Entry reset is forbidden while this thread holds roots or guest frames for the context. */
int32_t neoclr_gc_entry_check_v1(neoclr_aot_context *context);
/* Private dispatcher admission: quiescent, fault-free context and rooted Function
 * descriptor. Output remains unchanged on rejection. Context belongs to one image. */
int32_t neoclr_gc_callback_read_v1(neoclr_aot_context *context, uint64_t handle, void **output);
/* Generated only for images with bound fn<Void> targets. Does not reset the heap,
 * release the root, or clear a prior fault. Host failures return 3 without a guest fault. */
int32_t neoclr_invoke_void_callback_v1(uint64_t handle, neoclr_aot_context *context);
/* Allocation never collects; output is published only on success. */
int32_t neoclr_gc_allocate_v1(neoclr_aot_text_arena *arena, uint64_t bytes,
                             uint32_t kind, void **output);
/* Only at a complete compiler root boundary, or quiescent host with no guest
 * frames. Root addresses are observed, never rewritten. No reentry/concurrency
 * on this context. Service temporaries must not be live at a collection boundary. */
int32_t neoclr_gc_collect_v1(neoclr_aot_context *context, const neoclr_probe_frame *head);
/* Thread-local cumulative diagnostics across contexts/entries, not heap state. */
neoclr_gc_statistics neoclr_gc_statistics_v1(void);
/* Per-entry strong String pool: 4096 entries / 1 MiB UTF-8 payload, matching
 * interpreter defaults. Hits remain available at quota. No collection/reentry.
 * Failure preserves output; reset at the next admitted entry releases the pool. */
int32_t neoclr_string_intern_v1(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
                               const neoclr_aot_text **output);
/* Private entry-pump reader: the owner must validate its saved suspension boundary. */
int32_t neoclr_gc_callback_read_suspended_v1(neoclr_aot_context *, uint64_t, void **, const neoclr_probe_frame *);
#endif
