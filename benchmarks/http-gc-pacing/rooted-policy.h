#ifdef NEOCLR_INTEGRATED_PACING
#include "native-gc.h"
typedef struct { int unused; } rooted_pacing_state;
static int rooted_pacing_boundary(neoclr_aot_context *c, const neoclr_probe_frame *h,
    rooted_pacing_state *state, int stress) {
    (void)state;
    return stress ? neoclr_gc_idle_v1(c,h) : neoclr_gc_boundary_v1(c,h);
}
static int rooted_pacing_idle(neoclr_aot_context *c, const neoclr_probe_frame *h,
    rooted_pacing_state *state) {
    (void)state; return neoclr_gc_idle_v1(c,h);
}
#else
/* Closed-workload experiment only. Every allocating path MUST use rooted
 * admission. Not selected by generated code, installed hosts or the SDK. */
#include "native-gc.h"
typedef struct {
    const neoclr_aot_context *context;
    uint64_t allocations, collections, boundaries;
} rooted_pacing_state;
static int rooted_pacing_collect(neoclr_aot_context *context, const neoclr_probe_frame *head,
    rooted_pacing_state *state) {
    int status = neoclr_gc_collect_v1(context, head);
    if (!status) {
        neoclr_gc_statistics s = neoclr_gc_statistics_v1();
        *state = (rooted_pacing_state){context, s.allocations, s.collections, 0};
    }
    return status;
}
static int rooted_pacing_boundary(neoclr_aot_context *context, const neoclr_probe_frame *head,
    rooted_pacing_state *state, int stress) {
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    if (state->context != context || stress)
        return rooted_pacing_collect(context, head, state);
    /* Statistics span contexts: an unexpected collection must not repeatedly
     * reset this context's cleanup budget and postpone its collection forever. */
    if (s.collections != state->collections)
        return rooted_pacing_collect(context, head, state);
    if (s.allocations - state->allocations >= 16 || ++state->boundaries >= 64)
        return rooted_pacing_collect(context, head, state);
    return 0;
}
/* The real scheduler must explicitly invoke equivalent cleanup before idle and
 * shutdown. A boundary count cannot bound wall-clock retention while parked. */
static int rooted_pacing_idle(neoclr_aot_context *context, const neoclr_probe_frame *head,
    rooted_pacing_state *state) {
    return rooted_pacing_collect(context, head, state);
}

#endif
