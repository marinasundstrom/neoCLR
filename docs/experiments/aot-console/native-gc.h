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
/* Opt-in collector diagnostics. Timers are compiled out unless NEOCLR_GC_PROFILE
 * is defined; separate from the experimental heap/context layout. Cumulative,
 * thread-local and inclusive of validation/mark/sweep, including failed attempts. */
#define NEOCLR_GC_PAUSE_BUCKETS 64
/* Bucket i covers [2^i, 2^(i+1)-1] ns; bucket 0 also includes zero.
 * Live bytes count requested payload, excluding headers/padding/free spans.
 * Peaks are across contexts on this thread since reset, not summed heaps. */
typedef struct {
    uint64_t attempts, nanoseconds, maximum_nanoseconds;
    uint64_t failures, allocated_bytes, peak_arena_used;
    uint64_t live_bytes, live_blocks, peak_live_bytes;
    uint64_t pause_buckets[NEOCLR_GC_PAUSE_BUCKETS];
} neoclr_gc_profile;
neoclr_gc_profile neoclr_gc_profile_v1(void);
/* Reset diagnostics only, between operations; never changes heap/GC state.
 * Disabled builds return zero diagnostics and reset is a no-op. */
void neoclr_gc_profile_reset_v1(void);
/* Explicit managed collection uses the current published root chain. */
int32_t neoclr_gc_collect_managed_v1(neoclr_aot_context *, void *output);
/* Private current logical-stack capture; names are immutable current-image text.
 * Caller supplies published roots. Returns owned text; never changes Fault state. */
int32_t neoclr_stack_trace_capture_v1(int32_t skip, neoclr_aot_context *,
    const neoclr_aot_text **output, const neoclr_aot_text *const *names, uint32_t count);
/* Private thread-affine strong handles for future host-held callbacks/buffers.
 * Context, buffer and creating thread must remain alive until every handle is released.
 * Only null or an untagged live allocation base in that context is admitted.
 * No allocation/collection/reentry occurs in these operations; outputs are atomic on failure.
 * Limits are per native thread, shared across contexts. Handles are never reused.
 * This does not permit cross-thread context use or extend an entry/arena lifetime. */
#define NEOCLR_GC_HOST_ROOT_LIMIT 256
int32_t neoclr_gc_host_root_create_v1(neoclr_aot_context *context, void *value, uint64_t *output);
/* Private dependent handle: nonnull live owner retains value only while owner
 * is independently reachable. Participates in every collection, including stack
 * safepoints. On owner death, pointers clear before sweep; read/callback/replace
 * then reject without changing outputs. Release remains required and valid.
 * Same thread/context/quota rules as strong handles. Reading does not strengthen
 * ownership: callback dispatch rejects dependent handles. Read and create a
 * strong handle without an intervening collection before dispatch. Replacing a live value preserves its original owner.
 * No managed cleanup runs. Ordinary strong handles are unchanged. */
int32_t neoclr_gc_host_root_create_owned_v1(neoclr_aot_context *, void *owner,
    void *value, uint64_t *output);
int32_t neoclr_gc_host_root_replace_v1(neoclr_aot_context *context, uint64_t handle, void *value);
int32_t neoclr_gc_host_root_read_v1(neoclr_aot_context *context, uint64_t handle, void **output);
int32_t neoclr_gc_host_root_release_v1(neoclr_aot_context *context, uint64_t handle);
/* Private native resource ownership (not guest finalization). Registration is
 * caller-owned, zero-initialized and stable until release. It does not root owner.
 * On successful collection of owner, or explicit release, unlink/clear first,
 * then call cleanup exactly once. Cleanup may free the registration itself.
 * Cleanup must not perform I/O, call managed code, allocate in the managed heap,
 * reenter runtime services or fail. This is a trusted adapter contract.
 * Context and creating thread must outlive every registration. Before discarding
 * an arena, adapters explicitly release their records; entry reset rejects live
 * registrations. No callbacks run on failed tracing. Up to 1024 per context. */
typedef struct neoclr_gc_resource {
    neoclr_aot_context *context;
    void *owner, *state;
    void (*cleanup)(void *);
    struct neoclr_gc_resource *next;
} neoclr_gc_resource;
int32_t neoclr_gc_resource_attach_v1(neoclr_aot_context *, void *owner,
    neoclr_gc_resource *, void *state, void (*cleanup)(void *));
int32_t neoclr_gc_resource_release_v1(neoclr_aot_context *, neoclr_gc_resource *);
/* Entry reset is forbidden while this thread holds roots or guest frames for the context. */
int32_t neoclr_gc_entry_check_v1(neoclr_aot_context *context);
/* Quiescent exclusive ownership check. The supplied distinct handles must be
 * exactly this thread's complete root set for the context; no mutation occurs. */
int32_t neoclr_gc_owned_roots_check_v1(neoclr_aot_context *, const uint64_t *, uint32_t);
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
/* Private collector experiment, not a registration or stable host ABI.
 * Entries retain value only when owner is independently marked. Both pointers
 * must be null, or owner must be a live allocation base and value null/live in
 * this context. Values may point back to owners without keeping a cycle alive.
 * Up to NEOCLR_GC_HOST_ROOT_LIMIT entries; graph/array mutation and reentry are
 * forbidden throughout collection. On success, dead entries are cleared before
 * their allocations are reclaimed; retained is the result of this collection.
 * On failure entries are unchanged. Ordinary strong/parked roots still apply. */
typedef struct {
    void *owner, *value;
    int32_t retained;
} neoclr_gc_conditional_root;
int32_t neoclr_gc_collect_conditional_v1(neoclr_aot_context *,
    const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
/* Private conditional parked-stack registration. Park the active chain only if
 * all frames belong to context and owner is a nonnull live allocation base.
 * Record and frames remain native caller-owned until resume/discard; no collection
 * or allocation here. Failure preserves both the record and active chain.
 * Every collection traces these frames only after the owner becomes reachable.
 * On expiration, the collector clears head/owner and sets expired before sweeping.
 * Expired records cannot resume and still block entry reset until discarded.
 * No cleanup callback runs: the caller must discard and release native resources.
 * This neither weakens host callback handles nor changes ordinary parked stacks. */
int32_t neoclr_gc_roots_park_owned_v1(neoclr_aot_context *, void *, neoclr_parked_roots *);
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
