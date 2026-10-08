#ifndef NEOCLR_AOT_ROOT_PROBE_H
#define NEOCLR_AOT_ROOT_PROBE_H
#include <stdint.h>
#include <stddef.h>
/* Private diagnostic chain, not complete GC roots or a stable hosting ABI.
 * Frame storage belongs to the active native function. No allocation, reentry,
 * pointer mutation or collection is permitted in these callbacks. Ancestor snapshots
 * represent suspended pre-operation stacks, not all locals/arguments/results.
 * JSON belongs to the image; numeric holes are zero. Output-borrow pointees may be
 * uninitialized. Never scan the chain as complete roots or retain frames after leave.
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
void neoclr_probe_leave_v1(neoclr_probe_frame *frame);
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length);
/* Phase 1: initialized constructor storage; phase 2: successful call/constructor result.
 * Phase 3: seeded caller result scratch, not a published guest result.
 * Live slot addresses persist until the next pre-operation snapshot or frame leave. */
void neoclr_probe_transient_v2(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length);
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
#endif
