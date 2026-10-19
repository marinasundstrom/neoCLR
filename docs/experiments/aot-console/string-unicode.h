#ifndef NEOCLR_STRING_UNICODE_H
#define NEOCLR_STRING_UNICODE_H
#include "text-arena.h"
/* Private matched AOT adapters, not a public interop ABI. Inputs must be valid
 * immutable descriptors; only rooted helpers collect; none reenter guest code. Outputs are
 * unchanged on failure. Multi-allocation operations may leave garbage for GC.
 * Unicode data and segmentation are shared with the interpreter. */
int32_t neoclr_string_upper_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_lower_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_compare_canonical_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_hash_canonical_v1(const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_compare_ignore_case_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_grapheme_count_v1(const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_slice_graphemes_v1(const neoclr_aot_text *, int32_t, int32_t, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_grapheme_at_v1(const neoclr_aot_text *, int32_t, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_graphemes_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const void **);
int32_t neoclr_string_scalars_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const void **);
int32_t neoclr_string_from_chars_v1(const void *, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_hash_ordinal_v1(const neoclr_aot_text *, int32_t *);

int32_t neoclr_string_hash_ignore_case_v1(const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_upper_rooted_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_lower_rooted_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_slice_graphemes_rooted_v1(const neoclr_aot_text *, int32_t, int32_t, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_grapheme_at_rooted_v1(const neoclr_aot_text *, int32_t, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_graphemes_rooted_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const void **);
int32_t neoclr_string_scalars_rooted_v1(const neoclr_aot_text *, neoclr_aot_text_arena *, const void **);
int32_t neoclr_string_from_chars_rooted_v1(const void *, neoclr_aot_text_arena *, const neoclr_aot_text **);
int32_t neoclr_string_contains_canonical_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_starts_with_canonical_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_ends_with_canonical_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_contains_ignore_case_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_starts_with_ignore_case_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
int32_t neoclr_string_ends_with_ignore_case_v1(const neoclr_aot_text *, const neoclr_aot_text *, int32_t *);
#endif
