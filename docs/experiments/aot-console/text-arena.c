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
        ((uintptr_t)arena->data & 7) || bytes < 8 || bytes > 72 || (bytes & 7))
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

int32_t neoclr_allocate_bytes_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    if (length < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (length > 65536) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    if (arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t padding = (8 - (arena->used & 7)) & 7;
    uint64_t needed = padding + 16 + (uint64_t)length;
    if (needed > arena->capacity - arena->used) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    unsigned char *array = arena->data + arena->used + padding;
    uint64_t kind = UINT64_C(0x80000001), count = (uint64_t)length;
    memcpy(array, &kind, 8);
    memcpy(array + 8, &count, 8);
    memset(array + 16, 0, (size_t)length);
    arena->used += needed;
    *output = array;
    return NEOCLR_AOT_FAULT_NONE;
}
