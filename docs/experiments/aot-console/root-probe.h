#ifndef NEOCLR_AOT_ROOT_PROBE_H
#define NEOCLR_AOT_ROOT_PROBE_H
#include <stdint.h>
/* Private synchronous observation ABI, not a collector or stable hosting API.
 * lanes is readable only during the callback; retain no pointer from this call.
 * JSON plan belongs to the image, has length bytes and an extra NUL terminator.
 * Read only: no allocation, reentry, pointer mutation or collection is supported.
 * lane_count includes zero-filled numeric holes. Consult requiredSpillLanes and
 * typed root recipes; do not treat every nonzero word as a pointer or dereference
 * an output borrow's uninitialized pointee. No ancestors/locals are published here.
 */
void neoclr_probe_stack_roots_v1(uint32_t function, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length);
/* Diagnostic adapter count for the current thread, cumulative across invocations. */
uint64_t neoclr_root_probe_count_v1(void);
#endif
