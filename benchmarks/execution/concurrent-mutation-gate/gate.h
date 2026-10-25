#ifndef EXPERIMENT_MUTATION_GATE_H
#define EXPERIMENT_MUTATION_GATE_H
#include <pthread.h>
#include <stddef.h>
#include <stdint.h>

#ifndef GATE_WORDS
#define GATE_WORDS 4096
#endif
#if GATE_WORDS < 8 || GATE_WORDS > 16384
#error Experimental gate log must contain 8 to 16384 words
#endif
#define GATE_SCRATCH 256
#define GATE_PATCHES 8
/* Private experimental protocol. No managed ABI or collector admission implied. */
enum gate_status { GATE_OK, GATE_INVALID, GATE_STALE, GATE_CANCELLED };
typedef struct {
    pthread_mutex_t mutex;
    pthread_t owner;
    uint64_t epoch;
    unsigned active, cancelled;
    size_t count;
    uint64_t words[GATE_WORDS];
#ifdef NEOCLR_GATE_WORKERS
    struct { pthread_t thread; uint64_t generation; unsigned admitted; } writers[16];
#endif
} mutation_gate;
typedef struct {
    size_t offset, length;
    const void *source;
} mutation_patch;
#ifdef NEOCLR_GATE_WORKERS
/* Private epoch-scoped authority, not a transferable managed/heap handle.
 * Owner publishes tokens to already-created threads via external synchronization.
 * Revoke before thread exit/reuse. Gate/payload/source lifetimes remain caller-owned.
 * Stop invalidates all tokens; join all users before destroy/reclaim. */
typedef struct { mutation_gate *gate; uint64_t epoch, generation; unsigned slot; } gate_writer;
int gate_register_writer(mutation_gate *, pthread_t, gate_writer *);
int gate_revoke_writer(mutation_gate *, const gate_writer *);
int gate_write_registered(mutation_gate *, const gate_writer *, void *, size_t, size_t,
                          const mutation_patch *, size_t);
#endif
int gate_init(mutation_gate *g);
int gate_begin(mutation_gate *g, uint64_t *epoch);
/* Owner stops new reads. Caller must join marker before destroying/reclaiming. */
int gate_stop(mutation_gate *g);
int gate_destroy(mutation_gate *g);
/* All patches form one transaction. On overflow no bytes are changed. Caller
 * stops/joins/discards the cancelled cycle before retrying with GC inactive.
 * scan_start is an aligned offset into the payload, excluding an identity lane.
 * Complete eight-byte lanes are logged; shape/marker bytes are still protected.
 * Source storage must remain valid and cannot alias gate internals. */
int gate_write(mutation_gate *g, void *payload, size_t bytes, size_t scan_start,
               const mutation_patch *patches, size_t count);
/* Marker copies under the gate, then traces only private scratch. Epoch tokens
 * prevent an old worker from reading a later cycle. No callbacks under the gate. */
int gate_read(mutation_gate *g, uint64_t epoch, const void *payload, size_t bytes,
              size_t offset, void *scratch, size_t length);
int gate_drain(mutation_gate *g, uint64_t epoch, uint64_t *output,
               size_t capacity, size_t *count);
#endif
