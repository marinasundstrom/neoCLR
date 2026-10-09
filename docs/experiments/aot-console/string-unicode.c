/* Private native String kernels. Unicode tables/segmentation come from the same
 * pinned Rust sources as the interpreter. Helpers never collect or reenter. */
#include "string-unicode.h"
#include <limits.h>
#include <string.h>
extern int32_t neoclr_text_allocate_v1(neoclr_aot_text_arena *, uint64_t, neoclr_aot_text **);
extern int32_t neoclr_unicode_case_v1(const unsigned char *, size_t, int32_t, unsigned char *, size_t, size_t *);
extern int32_t neoclr_unicode_compare_v1(const unsigned char *, size_t, const unsigned char *, size_t, int32_t *);
extern int32_t neoclr_unicode_visit_v1(const unsigned char *, size_t, int32_t, int32_t (*)(size_t, size_t, void *), void *);
extern int32_t neoclr_is_single_grapheme_v1(const unsigned char *, size_t);
static int32_t casing(const neoclr_aot_text *text, neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int32_t upper) {
    if (!text || !arena || !output || text->length > INT32_MAX) return 3;
    size_t length;
    int32_t status = neoclr_unicode_case_v1(text->bytes, text->length, upper, NULL, 0, &length);
    if (status) return status;
    neoclr_aot_text *result;
    if ((status = neoclr_text_allocate_v1(arena, length, &result))) return status;
    if ((status = neoclr_unicode_case_v1(text->bytes, text->length, upper, result->bytes, length, &length))) return status;
    *output = result;
    return 0;
}
int32_t neoclr_string_upper_v1(const neoclr_aot_text *t, neoclr_aot_text_arena *a, const neoclr_aot_text **o) { return casing(t,a,o,1); }
int32_t neoclr_string_lower_v1(const neoclr_aot_text *t, neoclr_aot_text_arena *a, const neoclr_aot_text **o) { return casing(t,a,o,0); }
int32_t neoclr_string_compare_ignore_case_v1(const neoclr_aot_text *a, const neoclr_aot_text *b, int32_t *o) {
    if (!a || !b || !o) return 3;
    return neoclr_unicode_compare_v1(a->bytes,a->length,b->bytes,b->length,o);
}
static int32_t count_part(size_t at, size_t size, void *state) {
    (void)at; (void)size;
    uint64_t *count = state;
    if (*count == INT32_MAX) return 3;
    ++*count;
    return 0;
}
int32_t neoclr_string_grapheme_count_v1(const neoclr_aot_text *t, int32_t *output) {
    if (!t || !output) return 3;
    uint64_t count = 0;
    int32_t status = neoclr_unicode_visit_v1(t->bytes,t->length,0,count_part,&count);
    if (!status) *output = (int32_t)count;
    return status;
}
typedef struct { uint64_t position, target, offset, length; } find_state;
static int32_t find_part(size_t at, size_t length, void *state) {
    find_state *s = state;
    if (s->position++ != s->target) return 0;
    s->offset = at; s->length = length;
    return -1;
}
int32_t neoclr_string_grapheme_at_v1(const neoclr_aot_text *t, int32_t index, neoclr_aot_text_arena *a, const neoclr_aot_text **output) {
    if (!t || !a || !output) return 3;
    if (index < 0) return 8;
    find_state state = { .target = (uint32_t)index };
    int32_t status = neoclr_unicode_visit_v1(t->bytes,t->length,0,find_part,&state);
    if (status != -1) return status ? status : 8;
    neoclr_aot_text *result;
    if ((status = neoclr_text_allocate_v1(a,state.length,&result))) return status;
    memcpy(result->bytes,t->bytes+state.offset,state.length);
    *output = result;
    return 0;
}
typedef struct { const neoclr_aot_text *text; neoclr_aot_text_arena *arena; unsigned char *array; uint64_t count, index; int scalars; } vector_state;
static int32_t store_part(size_t at, size_t value, void *state) {
    vector_state *s = state;
    if (s->scalars) {
        uint64_t scalar = value;
        memcpy(s->array + 24 + at * 8, &scalar, 8);
    } else {
        neoclr_aot_text *part;
        int32_t status = neoclr_text_allocate_v1(s->arena,value,&part);
        if (status) return status;
        memcpy(part->bytes,s->text->bytes+at,value);
        memcpy(s->array + 16 + s->index++ * 8, &part, 8);
    }
    return 0;
}
static int32_t vector(const neoclr_aot_text *t, neoclr_aot_text_arena *a, const void **output, int scalars) {
    if (!t || !a || !output) return 3;
    uint64_t count = 0;
    int32_t status = neoclr_unicode_visit_v1(t->bytes,t->length,scalars,count_part,&count);
    if (status) return status;
    if (count > 65536) return 7;
    void *array;
    status = scalars ? neoclr_allocate_scalars_v1(a,(int32_t)count,0,&array) : neoclr_allocate_strings_v1(a,(int32_t)count,0,&array);
    if (status) return status;
    vector_state state = {t,a,array,count,0,scalars};
    status = neoclr_unicode_visit_v1(t->bytes,t->length,scalars,store_part,&state);
    if (!status) *output = array;
    return status;
}
int32_t neoclr_string_graphemes_v1(const neoclr_aot_text *t, neoclr_aot_text_arena *a, const void **o) { return vector(t,a,o,0); }
int32_t neoclr_string_scalars_v1(const neoclr_aot_text *t, neoclr_aot_text_arena *a, const void **o) { return vector(t,a,o,1); }
int32_t neoclr_string_from_chars_v1(const void *array, neoclr_aot_text_arena *a, const neoclr_aot_text **output) {
    if (!array || !a || !output) return 3;
    const unsigned char *data = array;
    uint64_t kind, count, length = 0;
    memcpy(&kind,data,8); memcpy(&count,data+8,8);
    kind &= UINT64_C(0xffffffff);
    if ((kind != UINT64_C(0x80000003) && kind != UINT64_C(0x80000004)) || count > 65536) return 3;
    for (uint64_t i=0;i<count;i++) {
        const neoclr_aot_text *part;
        memcpy(&part,data+16+i*8,8);
        if ((kind == UINT64_C(0x80000004) && !data[16+count*8+i]) || !part ||
            part->length > INT32_MAX-length || !neoclr_is_single_grapheme_v1(part->bytes,part->length)) return 3;
        length += part->length;
    }
    neoclr_aot_text *result;
    int32_t status = neoclr_text_allocate_v1(a,length,&result);
    if (status) return status;
    uint64_t at=0;
    for (uint64_t i=0;i<count;i++) {
        const neoclr_aot_text *part;
        memcpy(&part,data+16+i*8,8);
        memcpy(result->bytes+at,part->bytes,part->length); at+=part->length;
    }
    *output=result;
    return 0;
}

extern int32_t neoclr_unicode_hash_v1(const unsigned char *, size_t, int32_t, int32_t *);
int32_t neoclr_string_hash_ordinal_v1(const neoclr_aot_text *text, int32_t *output) {
    if (!text || !output) return 3;
    return neoclr_unicode_hash_v1(text->bytes, text->length, 0, output);
}
int32_t neoclr_string_hash_ignore_case_v1(const neoclr_aot_text *text, int32_t *output) {
    if (!text || !output) return 3;
    return neoclr_unicode_hash_v1(text->bytes, text->length, 1, output);
}
