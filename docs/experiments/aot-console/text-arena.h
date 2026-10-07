#ifndef NEOCLR_AOT_TEXT_ARENA_H
#define NEOCLR_AOT_TEXT_ARENA_H
#include "../aot-fault-details/fault-details.h"
#define NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT 5
/* Experimental ABI v4. Host owns an aligned writable buffer for this invocation.
 * The buffer must not overlap context/result and must cover capacity bytes.
 * All arena text is immutable after creation and expires at the next entry call
 * or when the host releases the buffer. Render a fault before either action.
 * Separate contexts AND buffers are required for concurrent invocations.
 */
typedef struct {
    unsigned char *data;
    uint64_t capacity;
    uint64_t used;
} neoclr_aot_text_arena;
typedef struct {
    neoclr_aot_fault fault;
    neoclr_aot_text_arena text;
} neoclr_aot_context;
_Static_assert(offsetof(neoclr_aot_context, text) == 1048, "arena offset");
_Static_assert(offsetof(neoclr_aot_context, text.used) == 1064, "cursor offset");
_Static_assert(sizeof(neoclr_aot_context) == 1072, "context size");
int32_t neoclr_entry_v4(int32_t value, int32_t *result, neoclr_aot_context *context);
/* No allocation outside the caller buffer. Success alone publishes output.
 * Exhaustion returns 5; malformed arena/formatting failures return 3.
 */
int32_t neoclr_int32_to_string_v1(int32_t value, neoclr_aot_text_arena *arena,
                                const neoclr_aot_text **output);
#endif
