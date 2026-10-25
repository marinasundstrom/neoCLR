#ifndef EXPERIMENT_LIVE_WORKER_API_H
#define EXPERIMENT_LIVE_WORKER_API_H
#include "native-gc.h"
#include "gate.h"
#ifdef NEOCLR_GATE_WORKERS
/* Immutable, owner-issued bounds for one allocation in one active epoch. This
 * trusted native descriptor is not a root or a managed capability. All users must be
 * quiescent and joined before finish disposes the gate; no use after that point. */
typedef struct {
    mutation_gate *gate;
    uint64_t epoch;
    void *payload;
    uint64_t bytes;
    uint32_t kind;
} live_worker_target;
int experiment_gc_worker_register(neoclr_aot_context *, pthread_t, gate_writer *);
int experiment_gc_worker_target_create(neoclr_aot_context *, void *, live_worker_target *);
int experiment_gc_worker_write(const gate_writer *, const live_worker_target *, const mutation_patch *, size_t);
int experiment_gc_worker_revoke(neoclr_aot_context *, const gate_writer *);
#endif
#endif
