#ifndef EXPERIMENT_PARTICIPANT_ROOTS_H
#define EXPERIMENT_PARTICIPANT_ROOTS_H
#include "native-gc.h"
/* Private experiment. All input contexts, descriptors and slot/lane values must
 * remain valid and quiescent throughout either call. No foreign TLS enumeration.
 * Output must not alias input storage. Values are conservative roots, not copied
 * objects; heap lifetime/collection exclusion remains the caller's responsibility.
 * No allocation, root publication or managed execution. Outputs change on success
 * only. Recount/capture under the same rendezvous if sizing is done separately. */
int neoclr_gc_participant_root_count_v1(const neoclr_aot_context *, const neoclr_gc_participant *,
                           unsigned, size_t *);
int neoclr_gc_participant_root_capture_v1(const neoclr_aot_context *, const neoclr_gc_participant *,
                             unsigned, uint64_t *, size_t, size_t *);
/* Private reusable native storage; zero initialize, exclusively own, then dispose.
 * Same rendezvous/alias contract as above. Count and validate before growing, then
 * copy current values without a third validation pass. Only count current values
 * as roots; retained capacity/stale words are never a root publication.
 * Invalid inputs/allocation failure preserve the previous buffer and result.
 * No collection or managed allocation; native capacity is retained until dispose. */
typedef struct neoclr_gc_root_buffer {
    uint64_t *values;
    size_t capacity;
    uint64_t captures, growths;
} neoclr_gc_root_buffer;
int neoclr_gc_participant_root_snapshot_v1(const neoclr_aot_context *, const neoclr_gc_participant *,
                              unsigned, neoclr_gc_root_buffer *, size_t *);
void neoclr_gc_root_buffer_dispose_v1(neoclr_gc_root_buffer *);
#endif
