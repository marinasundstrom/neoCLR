#ifndef NEOCLR_AOT_ROOT_PROBE_H
#define NEOCLR_AOT_ROOT_PROBE_H
/* Match the compiler selected-function budget in runtime/codegen/src/limits.rs. */
#define NEOCLR_NATIVE_FUNCTION_LIMIT 4096
#include "../faults/fault-details.h"
#include <stdint.h>
#include <stddef.h>
#ifdef NEOCLR_NATIVE_GC
#define neoclr_probe_enter_v3 neoclr_gc_enter_v1
#define neoclr_probe_leave_v1 neoclr_gc_leave_v1
#define neoclr_probe_stack_roots_v2 neoclr_gc_stack_roots_v1
#define neoclr_probe_transient_v2 neoclr_gc_transient_v1
#endif
enum { NEOCLR_GC_WORKER_READ_PHASE = 0x80, NEOCLR_GC_OWNED_STORAGE = 0x200, NEOCLR_GC_OWNED_TRANSIENT = 0x400, NEOCLR_GC_TYPED_PLAN = 0x800,
       NEOCLR_GC_OWNED_CHAIN = 0x1000, NEOCLR_GC_STORAGE_MAP = 0x2000, NEOCLR_GC_TRANSIENT_MAP = 0x4000 };
/* Private published frame chain, not a stable hosting ABI.
 * Frame storage belongs to the active native function. Diagnostic mode forbids
 * allocation, reentry, pointer mutation and collection in all callbacks.
 * NEOCLR_NATIVE_GC selects distinct hooks: completed stack snapshots collect.
 * Audited allocation helpers may also collect with a complete caller boundary and
 * their own temporary roots. Enter/leave/transient never collect.
 * Ancestor snapshots
 * represent suspended pre-operation stacks, not all locals/arguments/results.
 * Plan data belongs to the image; numeric holes are zero. Output-borrow pointees may be
 * uninitialized. Never follow stack borrow pointees or retain frames after leave.
 */
typedef struct {
    const void *address;
    uint32_t read_bytes, flags; /* 1 = discriminator, 2 = borrow address; never follow it. */
} neoclr_probe_storage;
_Static_assert(sizeof(neoclr_probe_storage) == 16, "storage entry size");
typedef struct neoclr_probe_frame {
    struct neoclr_probe_frame *previous;
    const void *context;
    const uint64_t *lanes;
    const char *plan;
    uint32_t length, lane_count, function, instruction;
    const neoclr_probe_storage *storage;
    const char *storage_plan;
    uint32_t storage_count, storage_length;
    const neoclr_probe_storage *transient;
    const char *transient_plan;
    uint32_t transient_count, transient_length, transient_phase, reserved;
} neoclr_probe_frame;
_Static_assert(sizeof(neoclr_probe_frame) == 104, "private probe frame size");
_Static_assert(offsetof(neoclr_probe_frame, function) == 40, "private probe frame layout");
void neoclr_probe_enter_v3(neoclr_probe_frame *frame, const void *context, uint32_t function,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length);
#ifdef NEOCLR_NATIVE_GC
/* Private generated-frame contract. Validate storage descriptors on entry, then
 * retain them until leave/parked-frame expiry. The caller exclusively owns the
 * table and frame: address/width/flags, table pointer and count must not change
 * during that lifetime. Described slot VALUES remain mutable and are traced at
 * collection. This is a trusted native contract, not memory protection; ordinary
 * native callers use neoclr_gc_enter_v1 and retain publication-time validation.
 * Frame linkage/context are runtime-owned and cannot be mutated by the caller.
 * OWNED_CHAIN is a runtime-only proof: this storage and every ancestor storage
 * are owned, ancestor transients are absent or owned, and contexts are identical.
 * It is established on entry from the parent proof plus the parent's current
 * transient contract. Suspended ancestors cannot republish until resumed.
 * Publication then checks the one shared fault context without walking ancestors;
 * collection still traces every frame and changing slot value. Mixed chains keep
 * publication-time validation. Current transient admission and plans stay checked. */
void neoclr_gc_enter_owned_v1(neoclr_probe_frame *frame, const void *context, uint32_t function,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length);
#endif
/* Native execution map, little-endian on the supported ARM64/x64 targets.
 * The collector traces published lanes, not these bytes. Full spill/layout
 * inspection stays in compiler metadata. A typed frame's plan points to this
 * header (length 8); otherwise plan retains its checked JSON representation. */
typedef struct { uint32_t magic, lane_count; } neoclr_gc_root_map;
#define NEOCLR_GC_ROOT_MAP_MAGIC UINT32_C(0x3143474e)
#define NEOCLR_GC_STORAGE_MAP_MAGIC UINT32_C(0x3153474e)
#define NEOCLR_GC_TRANSIENT_MAP_MAGIC UINT32_C(0x3154474e)
_Static_assert(sizeof(neoclr_gc_root_map) == 8, "root execution map size");
#ifdef NEOCLR_NATIVE_GC
/* Owned descriptor contracts with binary slot-count headers. Map bytes are
 * checked on each entry/publication; slot addresses/widths are still admitted.
 * STORAGE_MAP/TRANSIENT_MAP discriminate frame plan pointers from C strings.
 * Ordinary/legacy string hooks remain checked and clear the relevant format bit. */
/* Trusted owner-thread state imports. Entry returns an opaque runtime-instance
 * handle scoped to this generated activation. Store in an untraced compiler slot;
 * never expose through managed objects, queues or callbacks. Same-thread park/resume
 * and nested host reentry preserve it; migration or use after leave/thread exit is
 * forbidden. Ordinary native callers use the checked map hooks below. Each use
 * still validates current head, descriptors and faults; root values are not cached.
 * Shared-worker lowering retains its existing coordination imports. */
void *neoclr_gc_state_enter_v1(neoclr_probe_frame *, const void *, uint32_t,
    const neoclr_probe_storage *, uint32_t, const void *, uint32_t);
void neoclr_gc_state_stack_v1(void *, neoclr_probe_frame *, uint32_t,
    const uint64_t *, uint32_t, const void *, uint32_t);
void neoclr_gc_state_transient_v1(void *, neoclr_probe_frame *, uint32_t,
    const neoclr_probe_storage *, uint32_t, const void *, uint32_t);
void neoclr_gc_state_leave_v1(void *, neoclr_probe_frame *);
void neoclr_gc_enter_map_v1(neoclr_probe_frame *, const void *, uint32_t,
    const neoclr_probe_storage *, uint32_t, const void *, uint32_t);
void neoclr_gc_transient_map_v1(neoclr_probe_frame *, uint32_t,
    const neoclr_probe_storage *, uint32_t, const void *, uint32_t);
void neoclr_gc_stack_map_v1(neoclr_probe_frame *, uint32_t,
    const uint64_t *, uint32_t, const void *, uint32_t);
#ifdef NEOCLR_WORKER_GC
void neoclr_gc_worker_stack_map_v1(neoclr_probe_frame *, uint32_t,
    const uint64_t *, uint32_t, const void *, uint32_t, uint32_t);
/* New compiler contract: non-exclusive operations are audited local work/reads,
 * not merely operations without an allocation/collection point. */
void neoclr_gc_worker_read_phase_map_v1(neoclr_probe_frame *, uint32_t,
    const uint64_t *, uint32_t, const void *, uint32_t, uint32_t);
#endif
#endif
void neoclr_probe_leave_v1(neoclr_probe_frame *frame);
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length);
/* Phase 1: initialized constructor storage; phase 2: successful call/constructor result.
 * Phase 3: seeded caller result scratch, not a published guest result.
 * Live slot addresses persist until the next pre-operation snapshot or frame leave. */
void neoclr_probe_transient_v2(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length);
#ifdef NEOCLR_NATIVE_GC
/* Generated descriptors are exclusively owned and immutable from publication
 * until the next boundary, replacement publication or frame leave/expiry.
 * Slot values remain mutable and are read afresh by collection. Suspended/parked
 * frames retain this obligation. Ordinary native callers use the checked hook.
 * Only descriptor validation is amortized; faults and plan checks are unchanged. */
void neoclr_gc_transient_owned_v1(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length);
#endif

/* Read-only fault-context slot view for ABI v3/v4, including after frame removal.
 * Context must be initialized, alive and quiescent. Code zero returns zero without
 * reading stale frame storage. Otherwise output contains message, then frame names.
 * Returns slot count (at most 65), or -1 for null context, invalid frame count or
 * insufficient output capacity. Failure writes no entries. Never follows text pointers.
 * Slot addresses expire when the context is released; their values change on reentry.
 * Arena-backed text expires at entry reset/buffer release under the v4 contract.
 * This is diagnostic enumeration, not host root registration or a GC handle. */
int32_t neoclr_probe_fault_roots_v1(const void *context, neoclr_probe_storage *output,
                                   uint32_t capacity);
/* Diagnostic state is thread-local, cumulative counts span entry invocations.
 * Each frame records its host context; the chain can contain nested contexts. */
const neoclr_probe_frame *neoclr_root_probe_head_v1(void);
uint64_t neoclr_root_probe_count_v1(void);
uint32_t neoclr_root_probe_depth_v1(void);
/* Private cooperative-stack registration. Records and chain storage must stay
 * alive until resumed. Park/resume do not allocate or collect. */
typedef struct neoclr_parked_roots {
    neoclr_probe_frame *head;
    struct neoclr_parked_roots *next;
    /* Optional collector-owned lifetime. Never mutate while registered. */
    const void *owner_context;
    void *owner;
    int32_t expired;
    /* Private collection scratch; reset before tracing, never persists reachability. */
    int32_t traced;
} neoclr_parked_roots;
void neoclr_roots_park_v1(neoclr_parked_roots *);
void neoclr_roots_discard_v1(neoclr_parked_roots *);
void neoclr_roots_resume_v1(neoclr_parked_roots *);
const neoclr_parked_roots *neoclr_roots_parked_v1(void);
#endif
