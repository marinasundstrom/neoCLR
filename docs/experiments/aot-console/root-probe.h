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
typedef struct neoclr_probe_frame {
    struct neoclr_probe_frame *previous;
    const void *context;
    const uint64_t *lanes;
    const char *plan;
    uint32_t length, lane_count, function, instruction;
} neoclr_probe_frame;
_Static_assert(sizeof(neoclr_probe_frame) == 48, "private probe frame size");
_Static_assert(offsetof(neoclr_probe_frame, function) == 40, "private probe frame layout");
void neoclr_probe_enter_v1(neoclr_probe_frame *frame, const void *context, uint32_t function);
void neoclr_probe_leave_v1(neoclr_probe_frame *frame);
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length);
/* Diagnostic state is thread-local, cumulative counts span entry invocations.
 * Each frame records its host context; the chain can contain nested contexts. */
const neoclr_probe_frame *neoclr_root_probe_head_v1(void);
uint64_t neoclr_root_probe_count_v1(void);
uint32_t neoclr_root_probe_depth_v1(void);
#endif
