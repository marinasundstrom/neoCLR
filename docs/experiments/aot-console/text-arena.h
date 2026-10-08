#ifndef NEOCLR_AOT_TEXT_ARENA_H
#define NEOCLR_AOT_TEXT_ARENA_H
#include "../aot-fault-details/fault-details.h"
#define NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT 5
#define NEOCLR_AOT_FAULT_NULL_REFERENCE 6
#define NEOCLR_AOT_FAULT_ARRAY_LIMIT 7
#define NEOCLR_AOT_FAULT_INDEX_OUT_OF_RANGE 8
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
int32_t neoclr_int64_to_string_v1(int64_t value, neoclr_aot_text_arena *arena,
                                const neoclr_aot_text **output);
int32_t neoclr_uint64_to_string_v1(uint64_t value, neoclr_aot_text_arena *arena,
                                 const neoclr_aot_text **output);
/* UTF-8 services: invalid native arguments fault; slices return erased String or
 * Byte(1=range, 2=boundary). Success alone publishes output, slices use arena storage. */
int32_t neoclr_string_byte_count_v1(const neoclr_aot_text *text, int32_t *output);
int32_t neoclr_string_slice_utf8_v1(const neoclr_aot_text *text, int32_t start, int32_t length,
                                   neoclr_aot_text_arena *arena, void *output);
/* Immutable Byte[] snapshot, length <=65536; copied values share read-only storage. */
int32_t neoclr_utf8_encode_v1(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
                             const void **output);
/* Internal object allocation for the explicit reference-arena profile. Header is
 * a private type index; bytes includes header and padded payload (8..264 bytes).
 * The arena owns objects and cycles until the next entry/reset, just like text.
 */
int32_t neoclr_allocate_object_v1(neoclr_aot_text_arena *arena, uint32_t type,
                                uint32_t bytes, void **output);
/* Packed byte arrays, zero-initialized, length 0..65536. Negative length is
 * RuntimeError (matching the interpreter); excessive length is ArrayLimitExceeded. */
int32_t neoclr_allocate_bytes_v1(neoclr_aot_text_arena *arena, int32_t length, void **output);
/* Checked reserved capacity uses one initialization byte per payload byte.
 * Indexed stores publish slots; unchecked interior borrows remain unsupported. */
int32_t neoclr_reserve_bytes_v1(neoclr_aot_text_arena *arena, int32_t length, void **output);
int32_t neoclr_check_bytes_initialized_v1(const void *array, int32_t offset, int32_t count);
/* Strict UTF-8 from initialized managed bytes; erased String or Byte(1).
 * Valid text uses arena storage; invalid encoding allocates nothing. */
int32_t neoclr_utf8_decode_v1(const void *array, neoclr_aot_text_arena *arena, void *output);
/* Immutable concatenation; null inputs fault. No output/cursor publication on failure. */
int32_t neoclr_string_concat_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
                               neoclr_aot_text_arena *arena, const neoclr_aot_text **output);
#endif
