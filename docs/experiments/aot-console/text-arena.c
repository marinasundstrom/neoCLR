#include "text-arena.h"
#include <inttypes.h>
#include <string.h>

static int32_t store_text(const char *buffer, size_t length,
                          neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    if (arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t padding = (8 - (arena->used & 7)) & 7;
    uint64_t needed = padding + sizeof(uint64_t) + (uint64_t)length;
    if (needed > arena->capacity - arena->used) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    neoclr_aot_text *text = (neoclr_aot_text *)(arena->data + arena->used + padding);
    text->length = (uint64_t)length;
    memcpy(text->bytes, buffer, length);
    arena->used += needed;
    *output = text;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_int32_to_string_v1(int32_t value, neoclr_aot_text_arena *arena,
                                const neoclr_aot_text **output) {
    char buffer[12]; /* sign + ten digits + terminator */
    int length = snprintf(buffer, sizeof(buffer), "%" PRId32, value);
    if (length < 0 || (size_t)length >= sizeof(buffer)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return store_text(buffer, (size_t)length, arena, output);
}

int32_t neoclr_int64_to_string_v1(int64_t value, neoclr_aot_text_arena *arena,
                                const neoclr_aot_text **output) {
    char buffer[21]; /* sign + nineteen digits + terminator */
    int length = snprintf(buffer, sizeof(buffer), "%" PRId64, value);
    if (length < 0 || (size_t)length >= sizeof(buffer)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return store_text(buffer, (size_t)length, arena, output);
}

int32_t neoclr_uint64_to_string_v1(uint64_t value, neoclr_aot_text_arena *arena,
                                 const neoclr_aot_text **output) {
    char buffer[21]; /* twenty digits + terminator */
    int length = snprintf(buffer, sizeof(buffer), "%" PRIu64, value);
    if (length < 0 || (size_t)length >= sizeof(buffer)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return store_text(buffer, (size_t)length, arena, output);
}

int32_t neoclr_allocate_object_v1(neoclr_aot_text_arena *arena, uint32_t type,
                                uint32_t bytes, void **output) {
    if (arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7) || bytes < 8 || bytes > 264 || (bytes & 7))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t padding = (8 - (arena->used & 7)) & 7;
    uint64_t needed = padding + bytes;
    if (needed > arena->capacity - arena->used) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    unsigned char *object = arena->data + arena->used + padding;
    uint64_t identity = type;
    memcpy(object, &identity, sizeof(identity));
    memset(object + 8, 0, bytes - 8);
    arena->used += needed;
    *output = object;
    return NEOCLR_AOT_FAULT_NONE;
}

static int32_t allocate_bytes(neoclr_aot_text_arena *arena, int32_t length, int reserved, void **output) {
    if (length < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (length > 65536) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    if (arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t padding = (8 - (arena->used & 7)) & 7;
    uint64_t needed = padding + 16 + (uint64_t)length * (reserved ? 2 : 1);
    if (needed > arena->capacity - arena->used) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    unsigned char *array = arena->data + arena->used + padding;
    uint64_t kind = reserved ? UINT64_C(0x80000002) : UINT64_C(0x80000001), count = (uint64_t)length;
    memcpy(array, &kind, 8);
    memcpy(array + 8, &count, 8);
    memset(array + 16, 0, (size_t)length * (reserved ? 2 : 1));
    arena->used += needed;
    *output = array;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_allocate_bytes_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_bytes(arena, length, 0, output);
}
int32_t neoclr_reserve_bytes_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_bytes(arena, length, 1, output);
}

int32_t neoclr_check_bytes_initialized_v1(const void *array, int32_t offset, int32_t count) {
    uint64_t kind, length;
    memcpy(&kind, array, 8);
    memcpy(&length, (const unsigned char *)array + 8, 8);
    /* Preserve the service's range/limit errors before examining any slots. */
    if (kind != UINT64_C(0x80000002) || offset < 0 || count < 0 ||
        (uint64_t)offset > length || (uint64_t)count > length - (uint64_t)offset || count > 65536)
        return NEOCLR_AOT_FAULT_NONE;
    const unsigned char *initialized = (const unsigned char *)array + 16 + length;
    for (int32_t i = 0; i < count; i++) {
        if (!initialized[(uint64_t)offset + (uint64_t)i]) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    }
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_string_byte_count_v1(const neoclr_aot_text *text, int32_t *output) {
    if (!text || text->length > INT32_MAX) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    *output = (int32_t)text->length;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_string_slice_utf8_v1(const neoclr_aot_text *text, int32_t start, int32_t length,
                                   neoclr_aot_text_arena *arena, void *output) {
    if (!text) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint32_t tag = 2; /* private erased Byte */
    uint64_t payload;
    if (start < 0 || length < 0 || (uint64_t)start > text->length ||
        (uint64_t)length > text->length - (uint64_t)start) {
        payload = 1; /* OutOfRange takes precedence over boundary validation. */
    } else {
        uint64_t end = (uint64_t)start + (uint64_t)length;
        /* Every input String is valid UTF-8. Continuation bytes cannot start/end a slice. */
        if (((uint64_t)start < text->length && (text->bytes[start] & 0xc0) == 0x80) ||
            (end < text->length && (text->bytes[end] & 0xc0) == 0x80)) {
            payload = 2; /* InvalidBoundary */
        } else {
            const neoclr_aot_text *slice;
            int32_t status = store_text((const char *)text->bytes + start, (size_t)length, arena, &slice);
            if (status) return status;
            tag = 4; /* private erased String; preserve full pointer width */
            payload = (uint64_t)(uintptr_t)slice;
        }
    }
    memcpy(output, &tag, sizeof(tag));
    memcpy((unsigned char *)output + 8, &payload, sizeof(payload));
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_utf8_encode_v1(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
                             const void **output) {
    if (!text) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (text->length > 65536) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    void *array;
    int32_t status = allocate_bytes(arena, (int32_t)text->length, 0, &array);
    if (status) return status;
    /* Distinct private kind. No admitted operation mutates these value snapshots;
     * copies can safely share storage until the invocation arena is released. */
    uint64_t kind = UINT64_C(0x80000003);
    memcpy(array, &kind, 8);
    memcpy((unsigned char *)array + 16, text->bytes, (size_t)text->length);
    *output = array;
    return NEOCLR_AOT_FAULT_NONE;
}

/* Strict scalar UTF-8, matching the interpreter's String::from_utf8 contract. */
static int valid_utf8(const unsigned char *bytes, size_t length) {
    size_t i = 0;
    while (i < length) {
        unsigned char first = bytes[i++];
        if (first < 0x80) continue;
        size_t count;
        unsigned char low = 0x80, high = 0xbf;
        if (first >= 0xc2 && first <= 0xdf) count = 1;
        else if (first >= 0xe0 && first <= 0xef) {
            count = 2;
            if (first == 0xe0) low = 0xa0;
            if (first == 0xed) high = 0x9f;
        } else if (first >= 0xf0 && first <= 0xf4) {
            count = 3;
            if (first == 0xf0) low = 0x90;
            if (first == 0xf4) high = 0x8f;
        } else return 0;
        if (count > length - i || bytes[i] < low || bytes[i] > high) return 0;
        i++;
        while (--count) {
            if ((bytes[i++] & 0xc0) != 0x80) return 0;
        }
    }
    return 1;
}

int32_t neoclr_utf8_decode_v1(const void *array, neoclr_aot_text_arena *arena, void *output) {
    if (!array) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t kind, length;
    memcpy(&kind, array, 8);
    memcpy(&length, (const unsigned char *)array + 8, 8);
    if ((kind != UINT64_C(0x80000001) && kind != UINT64_C(0x80000002)) || length > 65536)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    /* The interpreter reads every slot before checking UTF-8, even after an
     * invalid leading byte. Preserve that failure precedence. */
    int32_t status = neoclr_check_bytes_initialized_v1(array, 0, (int32_t)length);
    if (status) return status;
    const unsigned char *bytes = (const unsigned char *)array + 16;
    uint32_t tag = 2;
    uint64_t payload = 1;
    if (valid_utf8(bytes, (size_t)length)) {
        const neoclr_aot_text *text;
        status = store_text((const char *)bytes, (size_t)length, arena, &text);
        if (status) return status;
        tag = 4;
        payload = (uint64_t)(uintptr_t)text;
    }
    memcpy(output, &tag, sizeof(tag));
    memcpy((unsigned char *)output + 8, &payload, sizeof(payload));
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_string_concat_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
                               neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    if (!left || !right || !arena || !output || arena->used > arena->capacity ||
        (arena->capacity && !arena->data) || ((uintptr_t)arena->data & 7))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    /* Preserve the runtime's size-overflow fault before applying the native budget. */
    if (right->length > UINT64_MAX - left->length) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t length = left->length + right->length;
    uint64_t padding = (8 - (arena->used & 7)) & 7;
    uint64_t available = arena->capacity - arena->used;
    if (padding > available || 8 > available - padding || length > available - padding - 8)
        return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    neoclr_aot_text *text = (neoclr_aot_text *)(arena->data + arena->used + padding);
    text->length = length;
    memcpy(text->bytes, left->bytes, (size_t)left->length);
    memcpy(text->bytes + left->length, right->bytes, (size_t)right->length);
    arena->used += padding + 8 + length;
    *output = text;
    return NEOCLR_AOT_FAULT_NONE;
}


int32_t neoclr_parse_int32_v1(const neoclr_aot_text *text, void *output) {
    if (!text || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t start = 0;
    int negative = text->length && text->bytes[0] == '-';
    if (text->length && (negative || text->bytes[0] == '+')) start = 1;
    uint64_t tag = 2, payload = 1; /* malformed, including empty/sign-only */
    if (start < text->length) {
        uint32_t magnitude = 0;
        uint32_t limit = negative ? UINT32_C(2147483648) : UINT32_C(2147483647);
        int overflow = 0;
        for (uint64_t i = start; i < text->length; ++i) {
            unsigned char ch = text->bytes[i];
            if (ch < '0' || ch > '9') goto publish;
            uint32_t digit = ch - '0';
            if (!overflow) {
                if (magnitude > (limit - digit) / 10) overflow = 1;
                else magnitude = magnitude * 10 + digit;
            }
        }
        if (overflow) payload = 2;
        else {
            tag = 1;
            /* Unsigned arithmetic preserves INT32_MIN without signed overflow. */
            payload = negative ? UINT32_C(0) - magnitude : magnitude;
        }
    }
publish:;
    uint64_t lanes[2] = {tag, payload};
    memcpy(output, lanes, sizeof(lanes));
    return NEOCLR_AOT_FAULT_NONE;
}
