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
#endif
