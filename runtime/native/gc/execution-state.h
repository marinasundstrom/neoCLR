#ifndef NEOCLR_NATIVE_EXECUTION_STATE_H
#define NEOCLR_NATIVE_EXECUTION_STATE_H
#include "native-gc.h"
/* Private per-native-thread state, shared only by the root and collector units.
 * Consolidation does not merge lifetimes: no whole-state reset is permitted.
 * Park/resume changes roots only; pacing epochs and statistics survive it.
 * A published boundary may check pacing using the state it already resolved.
 * No pointer to this state may be retained or transferred to another thread. */
typedef struct {
    struct {
        neoclr_probe_frame *head;
        uint64_t calls;
        uint32_t depth;
    } roots;
    struct {
        neoclr_gc_statistics statistics;
        struct {
            const neoclr_aot_context *context;
            uint64_t allocations, collections, boundaries;
        } pacing;
        int cleanup_active;
    } gc;
    /* Cold checked-string search hints follow the hot root/pacing fields. */
    uint32_t spill_marker_offsets[256];
} neoclr_execution_state;
/* This storage must not interpose between separately loaded runtime libraries. */
#if defined(__GNUC__) && !defined(_WIN32)
__attribute__((visibility("hidden")))
#endif
extern _Thread_local neoclr_execution_state neoclr_execution_state_v1;

#ifdef NEOCLR_GC_PROFILE
/* Diagnostic-only slow path classifies the first failed pacing condition. */
int32_t neoclr_gc_paced_collect_v1(neoclr_aot_context *, const neoclr_probe_frame *);
#endif

static inline int32_t neoclr_paced_boundary(neoclr_execution_state *state,
    neoclr_aot_context *context, const neoclr_probe_frame *boundary) {
    if (state->gc.pacing.context == context &&
        state->gc.statistics.collections == state->gc.pacing.collections &&
        state->gc.statistics.allocations - state->gc.pacing.allocations < 256 &&
        ++state->gc.pacing.boundaries < 4096) return 0;
#ifdef NEOCLR_GC_PROFILE
    return neoclr_gc_paced_collect_v1(context, boundary);
#else
    return neoclr_gc_idle_v1(context, boundary);
#endif
}
/* Only immediately after root publication's head/context proof, with no callback,
 * guest execution or collection intervening. Ordinary native callers stay checked. */
static inline int32_t neoclr_published_boundary(neoclr_execution_state *state,
    neoclr_aot_context *context, const neoclr_probe_frame *boundary) {
    if (!context || !boundary || state->gc.cleanup_active || boundary->context != context) return 3;
    return neoclr_paced_boundary(state, context, boundary);
}
#endif
