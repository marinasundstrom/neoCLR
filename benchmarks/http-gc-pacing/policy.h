/* Experiment only: not a production collection contract. A reserve cannot
 * guarantee admission of an arbitrary next allocation. See pressure-test.c. */
#include "native-gc.h"
static _Thread_local const neoclr_aot_context *pacing_context;
static _Thread_local uint64_t pacing_allocations, pacing_collections, pacing_boundaries;
static int pacing_collect(const neoclr_aot_context *context) {
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    if (context != pacing_context) {
        pacing_context = context;
        pacing_allocations = s.allocations;
        pacing_collections = s.collections;
        pacing_boundaries = 0;
        return 1;
    }
    if (s.collections != pacing_collections) {
        pacing_collections = s.collections;
        pacing_allocations = s.allocations;
        pacing_boundaries = 0;
    }
    /* Preserve stress behavior in small/full arenas. Count bounds also collect
     * without new allocations; they do not guarantee wall-clock idle cleanup. */
    return context->text.used > context->text.capacity ||
        context->text.capacity - context->text.used < UINT64_C(1048576) ||
        s.allocations - pacing_allocations >= 16 || ++pacing_boundaries >= 64;
}
