#ifndef NEOCLR_NATIVE_GC_H
#define NEOCLR_NATIVE_GC_H
#include "text-arena.h"
#include "root-probe.h"
/* Private nonmoving heap in the caller buffer. No malloc, external runtime or
 * arbitrary native-stack scan. All entry/arena lifetime restrictions still apply. */
enum { NEOCLR_GC_TEXT = 1, NEOCLR_GC_OBJECT = 2, NEOCLR_GC_BYTES = 3, NEOCLR_GC_STRINGS = 4 };
typedef struct {
    uint64_t collections, allocations, reclaimed_allocations, reclaimed_bytes;
} neoclr_gc_statistics;
/* Allocation never collects; output is published only on success. */
int32_t neoclr_gc_allocate_v1(neoclr_aot_text_arena *arena, uint64_t bytes,
                             uint32_t kind, void **output);
/* Only at a complete compiler root boundary, or quiescent host with no guest
 * frames. Root addresses are observed, never rewritten. No reentry/concurrency
 * on this context. Service temporaries must not be live at a collection boundary. */
int32_t neoclr_gc_collect_v1(neoclr_aot_context *context, const neoclr_probe_frame *head);
/* Thread-local cumulative diagnostics across contexts/entries, not heap state. */
neoclr_gc_statistics neoclr_gc_statistics_v1(void);
#endif
