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
/* Strict decode from borrowed host bytes; String or Byte(1) erased result.
 * Host storage stays valid across allocation/collection and is never retained. */
int32_t neoclr_text_decode_utf8_bytes_v1(const unsigned char *bytes, uint64_t length,
    neoclr_aot_text_arena *arena, void *output);
int32_t neoclr_utf8_decode_v1(const void *array, neoclr_aot_text_arena *arena, void *output);
/* Immutable concatenation; null inputs fault. No output/cursor publication on failure. */
int32_t neoclr_string_concat_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
                               neoclr_aot_text_arena *arena, const neoclr_aot_text **output);

/* Private bounded StringBuilder materialization. Join the initialized prefix of a
 * String[] into one immutable UTF-8 allocation. Validate the exact byte count
 * (0..Int32.MaxValue) before allocating. Invalid/null/uninitialized inputs return 3;
 * exhaustion returns 5. Failure publishes no output. No collection/reentry. */
int32_t neoclr_string_join_parts_v1(const void *parts, int32_t count, const neoclr_aot_text *separator, int32_t expected,
                                  neoclr_aot_text_arena *arena, const neoclr_aot_text **output);

/* Exact ASCII [+-]?[0-9]+; complete grammar validation precedes range.
 * Publishes erased Int32 or Byte(1=InvalidFormat, 2=Overflow), no allocation.
 * Null arguments fault without publishing output. */
int32_t neoclr_parse_int32_v1(const neoclr_aot_text *text, void *output);
/* Exact, case-sensitive UTF-8 bytes; empty patterns match. Null inputs fault.
 * Predicates allocate nothing and publish a normalized Boolean only on success. */
int32_t neoclr_string_contains_ordinal_v1(const neoclr_aot_text *text, const neoclr_aot_text *pattern, int32_t *output);
int32_t neoclr_string_starts_with_ordinal_v1(const neoclr_aot_text *text, const neoclr_aot_text *pattern, int32_t *output);
int32_t neoclr_string_ends_with_ordinal_v1(const neoclr_aot_text *text, const neoclr_aot_text *pattern, int32_t *output);
/* Private String-owner slots, default-null or checked uninitialized reservation.
 * Arena retains owners for the entire invocation; no per-slot retain/release. */
int32_t neoclr_allocate_strings_v1(neoclr_aot_text_arena *arena, int32_t length,
                                  int32_t reserved, void **output);
/* Checked snapshots of 1..64 padded lanes. No default initialization or element borrows.
 * Success alone publishes the array; markers distinguish unwritten elements. */
int32_t neoclr_reserve_records_v1(neoclr_aot_text_arena *arena, int32_t length,
                                uint32_t lanes, void **output);
/* Default-null nominal reference slots; same traced layout as one-lane records. */
int32_t neoclr_allocate_references_v1(neoclr_aot_text_arena *arena, int32_t length,
                                    void **output);
/* Atomic scalar snapshots: one padded lane and initialization byte per element. */
int32_t neoclr_allocate_scalars_v1(neoclr_aot_text_arena *arena, int32_t length,
                                 int32_t reserved, void **output);
/* Lexical Unix paths over immutable UTF-8: slash only, no normalization or I/O.
 * Always copy into owned text storage, including empty/unchanged results.
 * Null inputs fault (3); overflow/exhaustion returns 5. Failure leaves output intact.
 * No collection/reentry; inputs must describe readable immutable payloads. */
int32_t neoclr_path_combine_unix_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output);
int32_t neoclr_path_file_name_unix_v1(const neoclr_aot_text *path,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output);
int32_t neoclr_string_compare_ordinal_v1(const neoclr_aot_text *left,
    const neoclr_aot_text *right, int32_t *output);
#endif
