#include "text-arena.h"
#include "memory-views.h"
#include <inttypes.h>
#include <string.h>

#ifdef NEOCLR_NATIVE_GC
#include "../gc/native-gc.h"
#endif
#ifdef NEOCLR_NATIVE_GC
/* Use the collector's canonical kind constants directly. MSVC diagnoses even
 * explicitly cast cross-enum comparisons in C static assertions (C5287). */
enum { TEXT_STORAGE = NEOCLR_GC_TEXT, OBJECT_STORAGE = NEOCLR_GC_OBJECT,
       BYTES_STORAGE = NEOCLR_GC_BYTES, STRINGS_STORAGE = NEOCLR_GC_STRINGS,
       RECORDS_STORAGE = NEOCLR_GC_RECORDS };
#else
enum { TEXT_STORAGE = 1, OBJECT_STORAGE = 2, BYTES_STORAGE = 3, STRINGS_STORAGE = 4, RECORDS_STORAGE = 5 };
#endif
static int32_t reserve_storage(neoclr_aot_text_arena *arena, uint64_t bytes,
                               uint32_t kind, void **output) {
#ifdef NEOCLR_NATIVE_GC
    return neoclr_gc_allocate_v1(arena, bytes, kind, output);
#else
    (void)kind;
    if (!arena || !output || arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t padding = (8 - (arena->used & 7)) & 7;
    uint64_t available = arena->capacity - arena->used;
    if (padding > available || bytes > available - padding) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    *output = arena->data + arena->used + padding;
    arena->used += padding + bytes;
    return 0;
#endif
}

/* Primitive-array initialization has no managed inputs and performs no callbacks.
 * Only generated calls with a complete active boundary opt into collection. */
static int32_t reserve_primitive_storage(neoclr_aot_text_arena *arena, uint64_t bytes,
    void **output, int rooted) {
#ifdef NEOCLR_NATIVE_GC
    if (rooted) {
        const neoclr_probe_frame *head = neoclr_root_probe_head_v1();
        neoclr_aot_context *context = head ? (neoclr_aot_context *)head->context : NULL;
        if (!context || &context->text != arena) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        return neoclr_gc_allocate_rooted_v1(context, head, bytes, BYTES_STORAGE, output);
    }
#else
    (void)rooted;
#endif
    return reserve_storage(arena, bytes, BYTES_STORAGE, output);
}

/* Explicitly selected only by audited generated calls. Arena-only native callers
 * retain the noncollecting services until their entire temporary lifetime is audited. */
static int32_t reserve_values(neoclr_aot_text_arena *arena, uint64_t bytes, uint32_t kind,
    void **output, int rooted, const void *const *values, uint32_t count) {
#ifdef NEOCLR_NATIVE_GC
    if (rooted) {
        const neoclr_probe_frame *head = neoclr_root_probe_head_v1();
        neoclr_aot_context *context = head ? (neoclr_aot_context *)head->context : NULL;
        if (!context || &context->text != arena) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        return neoclr_gc_allocate_rooted_values_v1(context, head, values, count, bytes, kind, output);
    }
#else
    (void)rooted; (void)values; (void)count;
#endif
    return reserve_storage(arena, bytes, kind, output);
}

static int32_t reserve_inputs(neoclr_aot_text_arena *arena, uint64_t bytes, uint32_t kind,
    void **output, int rooted, const void *left, const void *right) {
    const void *values[] = {left, right};
    return reserve_values(arena, bytes, kind, output, rooted, values, 2);
}

static int32_t reserve_text_inputs(neoclr_aot_text_arena *arena, uint64_t bytes,
    void **output, int rooted, const void *left, const void *right) {
    return reserve_inputs(arena, bytes, TEXT_STORAGE, output, rooted, left, right);
}

static int32_t store_text_input(const char *buffer, size_t length,
                          neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted, const void *owner) {
    if (arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (length > UINT64_MAX - 8) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    void *storage;
    int32_t status = reserve_text_inputs(arena, 8 + (uint64_t)length, &storage, rooted, owner, NULL);
    if (status) return status;
    neoclr_aot_text *text = storage;
    text->length = (uint64_t)length;
    memcpy(text->bytes, buffer, length);
    *output = text;
    return NEOCLR_AOT_FAULT_NONE;
}

static int32_t int32_to_string(int32_t value, neoclr_aot_text_arena *arena,
                                const neoclr_aot_text **output, int rooted) {
    char buffer[12]; /* sign + ten digits + terminator */
    int length = snprintf(buffer, sizeof(buffer), "%" PRId32, value);
    if (length < 0 || (size_t)length >= sizeof(buffer)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return store_text_input(buffer, (size_t)length, arena, output, rooted, NULL);
}

int32_t neoclr_int32_to_string_v1(int32_t value, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) {
    return int32_to_string(value, arena, output, 0);
}
int32_t neoclr_int32_to_string_rooted_v1(int32_t value, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) {
    return int32_to_string(value, arena, output, 1);
}

static int32_t int64_to_string(int64_t value, neoclr_aot_text_arena *arena,
                                const neoclr_aot_text **output, int rooted) {
    char buffer[21]; /* sign + nineteen digits + terminator */
    int length = snprintf(buffer, sizeof(buffer), "%" PRId64, value);
    if (length < 0 || (size_t)length >= sizeof(buffer)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return store_text_input(buffer, (size_t)length, arena, output, rooted, NULL);
}

int32_t neoclr_int64_to_string_v1(int64_t value, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) {
    return int64_to_string(value, arena, output, 0);
}
int32_t neoclr_int64_to_string_rooted_v1(int64_t value, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) {
    return int64_to_string(value, arena, output, 1);
}

static int32_t uint64_to_string(uint64_t value, neoclr_aot_text_arena *arena,
                                 const neoclr_aot_text **output, int rooted) {
    char buffer[21]; /* twenty digits + terminator */
    int length = snprintf(buffer, sizeof(buffer), "%" PRIu64, value);
    if (length < 0 || (size_t)length >= sizeof(buffer)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return store_text_input(buffer, (size_t)length, arena, output, rooted, NULL);
}

int32_t neoclr_uint64_to_string_v1(uint64_t value, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) {
    return uint64_to_string(value, arena, output, 0);
}
int32_t neoclr_uint64_to_string_rooted_v1(uint64_t value, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) {
    return uint64_to_string(value, arena, output, 1);
}

static int32_t allocate_object(neoclr_aot_text_arena *arena, uint32_t type,
                                uint32_t bytes, void **output, int rooted) {
    if (!arena || !output || arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7) || bytes < 8 || bytes > NEOCLR_NATIVE_OBJECT_BYTES || (bytes & 7))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    int32_t status = reserve_values(arena, bytes, OBJECT_STORAGE, &storage, rooted, NULL, 0);
    if (status) return status;
    unsigned char *object = storage;
    uint64_t identity = type;
    memcpy(object, &identity, sizeof(identity));
    memset(object + 8, 0, bytes - 8);
    *output = object;
    return NEOCLR_AOT_FAULT_NONE;
}

/* Static data has the same nonmoving, conservative word tracing as ordinary
 * object storage. The table is a context root, never a process-global dictionary.
 * Generated code supplies the specialized image's exact type index and lane count.
 * Context execution is thread-affine; no callback/yield occurs during publication.
 * The high word of each storage header starts at zero and is reserved for the
 * generated type-initialization guard. It is not part of the traced field payload. */
static int32_t static_storage(neoclr_aot_context *context, uint32_t type,
    uint32_t count, uint32_t bytes, void **output, int rooted) {
    if (!context || !output || !count || count > NEOCLR_NATIVE_TYPE_LIMIT || type >= count ||
        bytes < 8 || bytes > NEOCLR_NATIVE_OBJECT_BYTES || (bytes & 7)) return 3;
    if (!context->static_storage) {
        void *table;
        int32_t status = reserve_values(&context->text, 8 + 8 * (uint64_t)count,
            OBJECT_STORAGE, &table, rooted, NULL, 0);
        if (status) return status;
        memset(table, 0, 8 + 8 * (size_t)count);
        uint64_t length = count; memcpy(table, &length, 8);
        context->static_storage = table;
    }
    uint64_t *table = context->static_storage;
    if (table[0] != count) return 3;
    if (!table[type + 1]) {
        void *storage;
        int32_t status = allocate_object(&context->text, type, bytes, &storage, rooted);
        if (status) return status;
        table[type + 1] = (uintptr_t)storage;
    }
    *output = (void *)(uintptr_t)table[type + 1];
    return 0;
}
int32_t neoclr_static_storage_v1(neoclr_aot_context *c, uint32_t type, uint32_t count,
    uint32_t bytes, void **output) { return static_storage(c, type, count, bytes, output, 0); }
int32_t neoclr_static_storage_rooted_v1(neoclr_aot_context *c, uint32_t type, uint32_t count,
    uint32_t bytes, void **output) { return static_storage(c, type, count, bytes, output, 1); }

/* No callbacks or collecting work may occur between reservation, zeroing and
 * publication. Generated caller frames retain constructor arguments/receivers. */
int32_t neoclr_allocate_object_v1(neoclr_aot_text_arena *arena, uint32_t type,
    uint32_t bytes, void **output) { return allocate_object(arena, type, bytes, output, 0); }
int32_t neoclr_allocate_object_rooted_v1(neoclr_aot_text_arena *arena, uint32_t type,
    uint32_t bytes, void **output) { return allocate_object(arena, type, bytes, output, 1); }

static int32_t allocate_bytes(neoclr_aot_text_arena *arena, int32_t length, int reserved, void **output, int rooted, const void *input) {
    if (length < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (length > NEOCLR_NATIVE_BYTE_ARRAY_LIMIT) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    if (arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    uint64_t bytes = 16 + (uint64_t)length * (reserved ? 2 : 1);
    int32_t status = input
        ? reserve_inputs(arena, bytes, BYTES_STORAGE, &storage, rooted, input, NULL)
        : reserve_primitive_storage(arena, bytes, &storage, rooted);
    if (status) return status;
    unsigned char *array = storage;
    uint64_t kind = reserved ? UINT64_C(0x80000002) : UINT64_C(0x80000001), count = (uint64_t)length;
    memcpy(array, &kind, 8);
    memcpy(array + 8, &count, 8);
    memset(array + 16, 0, (size_t)length * (reserved ? 2 : 1));
    *output = array;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_allocate_bytes_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_bytes(arena, length, 0, output, 0, NULL);
}
int32_t neoclr_reserve_bytes_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_bytes(arena, length, 1, output, 0, NULL);
}

int32_t neoclr_allocate_bytes_rooted_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_bytes(arena, length, 0, output, 1, NULL);
}
int32_t neoclr_reserve_bytes_rooted_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_bytes(arena, length, 1, output, 1, NULL);
}

static int32_t check_bytes_initialized(const void *array, int32_t offset, int32_t count) {
    uint64_t kind, length;
    memcpy(&kind, array, 8);
    kind &= UINT64_C(0xffffffff);
    memcpy(&length, (const unsigned char *)array + 8, 8);
    /* Preserve the service's range/limit errors before examining any slots. */
    if (kind != UINT64_C(0x80000002) || offset < 0 || count < 0 ||
        (uint64_t)offset > length || (uint64_t)count > length - (uint64_t)offset)
        return NEOCLR_AOT_FAULT_NONE;
    const unsigned char *initialized = (const unsigned char *)array + 16 + length;
    for (int32_t i = 0; i < count; i++) {
        if (!initialized[(uint64_t)offset + (uint64_t)i]) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    }
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_check_bytes_initialized_v1(const void *array, int32_t offset, int32_t count) {
    /* Stream services retain their own 64 KiB request limit and error precedence. */
    if (count > 65536) return NEOCLR_AOT_FAULT_NONE;
    return check_bytes_initialized(array, offset, count);
}

int32_t neoclr_string_byte_count_v1(const neoclr_aot_text *text, int32_t *output) {
    if (!text || text->length > INT32_MAX) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    *output = (int32_t)text->length;
    return NEOCLR_AOT_FAULT_NONE;
}

static int32_t string_slice_utf8(const neoclr_aot_text *text, int32_t start, int32_t length,
    neoclr_aot_text_arena *arena, void *output, int rooted) {
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
            void *storage;
            int32_t status = reserve_text_inputs(arena, 8 + (uint64_t)length,
                &storage, rooted, text, NULL);
            if (status) return status;
            neoclr_aot_text *slice = storage;
            slice->length = (uint64_t)length;
            memcpy(slice->bytes, text->bytes + start, (size_t)length);
            tag = 4; /* private erased String; preserve full pointer width */
            payload = (uint64_t)(uintptr_t)slice;
        }
    }
    memcpy(output, &tag, sizeof(tag));
    memcpy((unsigned char *)output + 8, &payload, sizeof(payload));
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_string_slice_utf8_v1(const neoclr_aot_text *text, int32_t start, int32_t length,
    neoclr_aot_text_arena *arena, void *output) {
    return string_slice_utf8(text, start, length, arena, output, 0);
}
int32_t neoclr_string_slice_utf8_rooted_v1(const neoclr_aot_text *text, int32_t start, int32_t length,
    neoclr_aot_text_arena *arena, void *output) {
    return string_slice_utf8(text, start, length, arena, output, 1);
}

static int32_t utf8_encode(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
                             const void **output, int rooted) {
    if (!text) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (text->length > NEOCLR_NATIVE_BYTE_ARRAY_LIMIT) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    void *array;
    int32_t status = allocate_bytes(arena, (int32_t)text->length, 0, &array, rooted, text);
    if (status) return status;
    /* Distinct private kind. No admitted operation mutates these value snapshots;
     * copies can safely share storage until the invocation arena is released. */
    uint64_t kind = UINT64_C(0x80000003);
    memcpy(array, &kind, 8);
    memcpy((unsigned char *)array + 16, text->bytes, (size_t)text->length);
    *output = array;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_utf8_encode_v1(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
    const void **output) {
    return utf8_encode(text, arena, output, 0);
}
int32_t neoclr_utf8_encode_rooted_v1(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
    const void **output) {
    return utf8_encode(text, arena, output, 1);
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

static int32_t text_decode_utf8_bytes(const unsigned char *, uint64_t,
    neoclr_aot_text_arena *, void *, int, const void *);

static int32_t utf8_decode(const void *array, neoclr_aot_text_arena *arena, void *output, int rooted) {
    if (!array) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t kind, length;
    memcpy(&kind, array, 8);
    kind &= UINT64_C(0xffffffff);
    memcpy(&length, (const unsigned char *)array + 8, 8);
    if ((kind != UINT64_C(0x80000001) && kind != UINT64_C(0x80000002)) || length > NEOCLR_NATIVE_BYTE_ARRAY_LIMIT)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    /* The interpreter reads every slot before checking UTF-8, even after an
     * invalid leading byte. Preserve that failure precedence. */
    int32_t status = check_bytes_initialized(array, 0, (int32_t)length);
    if (status) return status;
    const unsigned char *bytes = (const unsigned char *)array + 16;
    return text_decode_utf8_bytes(bytes, length, arena, output, rooted, array);
}

static int32_t text_decode_utf8_bytes(const unsigned char *bytes, uint64_t length,
    neoclr_aot_text_arena *arena, void *output, int rooted, const void *owner) {
    if ((!bytes && length) || !arena || !output || length > SIZE_MAX)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint32_t tag = 2;
    uint64_t payload = 1;
    if (valid_utf8(bytes, (size_t)length)) {
        const neoclr_aot_text *text;
        int32_t status = store_text_input(bytes ? (const char *)bytes : "", (size_t)length, arena, &text, rooted, owner);
        if (status) return status;
        tag = 4;
        payload = (uint64_t)(uintptr_t)text;
    }
    memcpy(output, &tag, sizeof(tag));
    memcpy((unsigned char *)output + 8, &payload, sizeof(payload));
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_utf8_decode_v1(const void *array, neoclr_aot_text_arena *arena, void *output) {
    return utf8_decode(array, arena, output, 0);
}
int32_t neoclr_utf8_decode_rooted_v1(const void *array, neoclr_aot_text_arena *arena, void *output) {
    return utf8_decode(array, arena, output, 1);
}
int32_t neoclr_text_decode_utf8_bytes_v1(const unsigned char *bytes, uint64_t length,
    neoclr_aot_text_arena *arena, void *output) {
    /* Raw native buffers do not provide a managed owner; retain noncollecting admission. */
    return text_decode_utf8_bytes(bytes, length, arena, output, 0, NULL);
}

/* Native-owned bytes must stay alive and stable for the duration of this call.
 * Managed interior pointers must use the owner-retaining decoder instead. */
int32_t neoclr_text_decode_utf8_host_bytes_rooted_v1(const unsigned char *bytes,
    uint64_t length, neoclr_aot_text_arena *arena, void *output) {
    return text_decode_utf8_bytes(bytes, length, arena, output, 1, NULL);
}

static int32_t string_concat(const neoclr_aot_text *left, const neoclr_aot_text *right,
                               neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    if (!left || !right || !arena || !output || arena->used > arena->capacity ||
        (arena->capacity && !arena->data) || ((uintptr_t)arena->data & 7))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    /* Preserve the runtime's size-overflow fault before applying the native budget. */
    if (right->length > UINT64_MAX - left->length) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t length = left->length + right->length;
    if (length > UINT64_MAX - 8) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    void *storage;
    int32_t status = reserve_text_inputs(arena, 8 + length, &storage, rooted, left, right);
    if (status) return status;
    neoclr_aot_text *text = storage;
    text->length = length;
    memcpy(text->bytes, left->bytes, (size_t)left->length);
    memcpy(text->bytes + left->length, right->bytes, (size_t)right->length);
    *output = text;
    return NEOCLR_AOT_FAULT_NONE;
}


int32_t neoclr_string_concat_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return string_concat(left, right, arena, output, 0);
}
int32_t neoclr_string_concat_rooted_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return string_concat(left, right, arena, output, 1);
}

static int32_t string_join_parts(const void *parts, int32_t count, const neoclr_aot_text *separator, int32_t expected,
                                  neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    if (!parts || !separator || !output || count < 0 || expected < 0)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    const unsigned char *array = parts;
    uint64_t kind, capacity;
    memcpy(&kind, array, 8);
    kind &= UINT64_C(0xffffffff);
    memcpy(&capacity, array + 8, 8);
    if ((kind != UINT64_C(0x80000003) && kind != UINT64_C(0x80000004)) ||
        capacity > 65536 || (uint64_t)count > capacity)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t separators = count > 0 ? (uint64_t)(count - 1) : 0;
    if (separators && separator->length > (uint64_t)expected / separators)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t length = separators * separator->length;
    for (int32_t i = 0; i < count; i++) {
        if (kind == UINT64_C(0x80000004) && !array[16 + capacity * 8 + (uint32_t)i])
            return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        const neoclr_aot_text *part;
        memcpy(&part, array + 16 + (uint32_t)i * 8, 8);
        if (!part || part->length > (uint64_t)expected - length)
            return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        length += part->length;
    }
    if (length != (uint64_t)expected) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    int32_t status = reserve_text_inputs(arena, 8 + length, &storage, rooted, parts, separator);
    if (status) return status;
    neoclr_aot_text *text = storage;
    text->length = length;
    uint64_t offset = 0;
    for (int32_t i = 0; i < count; i++) {
        const neoclr_aot_text *part;
        memcpy(&part, array + 16 + (uint32_t)i * 8, 8);
        if (i) {
            memcpy(text->bytes + offset, separator->bytes, (size_t)separator->length);
            offset += separator->length;
        }
        memcpy(text->bytes + offset, part->bytes, (size_t)part->length);
        offset += part->length;
    }
    *output = text;
    return NEOCLR_AOT_FAULT_NONE;
}


int32_t neoclr_string_join_parts_v1(const void *parts, int32_t count, const neoclr_aot_text *separator,
    int32_t expected, neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return string_join_parts(parts, count, separator, expected, arena, output, 0);
}
int32_t neoclr_string_join_parts_rooted_v1(const void *parts, int32_t count, const neoclr_aot_text *separator,
    int32_t expected, neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return string_join_parts(parts, count, separator, expected, arena, output, 1);
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


int32_t neoclr_parse_int64_v1(const neoclr_aot_text *text, void *output) {
    if (!text || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t start = 0;
    int negative = text->length && text->bytes[0] == '-';
    if (text->length && (negative || text->bytes[0] == '+')) start = 1;
    uint64_t tag = 2, payload = 1; /* malformed, including empty/sign-only */
    if (start < text->length) {
        uint64_t magnitude = 0;
        uint64_t limit = negative ? UINT64_C(9223372036854775808) : UINT64_C(9223372036854775807);
        int overflow = 0;
        for (uint64_t i = start; i < text->length; ++i) {
            unsigned char ch = text->bytes[i];
            if (ch < '0' || ch > '9') goto publish;
            uint64_t digit = ch - '0';
            if (!overflow) {
                if (magnitude > (limit - digit) / 10) overflow = 1;
                else magnitude = magnitude * 10 + digit;
            }
        }
        if (overflow) payload = 2;
        else {
            tag = 5;
            /* Unsigned arithmetic preserves INT64_MIN without signed overflow. */
            payload = negative ? UINT64_C(0) - magnitude : magnitude;
        }
    }
publish:;
    uint64_t lanes[2] = {tag, payload};
    memcpy(output, lanes, sizeof(lanes));
    return NEOCLR_AOT_FAULT_NONE;
}


int32_t neoclr_string_starts_with_ordinal_v1(const neoclr_aot_text *text,
        const neoclr_aot_text *pattern, int32_t *output) {
    if (!text || !pattern || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    *output = pattern->length <= text->length &&
        memcmp(text->bytes, pattern->bytes, (size_t)pattern->length) == 0;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_string_ends_with_ordinal_v1(const neoclr_aot_text *text,
        const neoclr_aot_text *pattern, int32_t *output) {
    if (!text || !pattern || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    *output = pattern->length <= text->length &&
        memcmp(text->bytes + (text->length - pattern->length), pattern->bytes, (size_t)pattern->length) == 0;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_string_contains_ordinal_v1(const neoclr_aot_text *text,
        const neoclr_aot_text *pattern, int32_t *output) {
    if (!text || !pattern || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    int32_t found = pattern->length == 0;
    if (!found && pattern->length <= text->length) {
        uint64_t last = text->length - pattern->length;
        for (uint64_t at = 0; at <= last; ++at) {
            if (memcmp(text->bytes + at, pattern->bytes, (size_t)pattern->length) == 0) {
                found = 1;
                break;
            }
        }
    }
    *output = found;
    return NEOCLR_AOT_FAULT_NONE;
}


static int32_t allocate_strings(neoclr_aot_text_arena *arena, int32_t length,
                                  int32_t reserved, void **output, int rooted) {
    if (length < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (length > 65536) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    if (!arena || !output || (reserved != 0 && reserved != 1) ||
        arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    int32_t status = reserve_inputs(arena, 16 + (uint64_t)length * (reserved ? 9 : 8), STRINGS_STORAGE, &storage, rooted, NULL, NULL);
    if (status) return status;
    unsigned char *array = storage;
    uint64_t kind = reserved ? UINT64_C(0x80000004) : UINT64_C(0x80000003);
    uint64_t count = (uint64_t)length;
    memcpy(array, &kind, 8);
    memcpy(array + 8, &count, 8);
    memset(array + 16, 0, (size_t)length * (reserved ? 9 : 8));
    *output = array;
    return NEOCLR_AOT_FAULT_NONE;
}

/* Private checked value-array storage: header, length, padded lane count, snapshots,
 * then one initialization byte per element. Initialization does not collect. */
static int32_t reserve_records(neoclr_aot_text_arena *arena, int32_t length,
                                uint32_t lanes, void **output, int rooted) {
    if (length < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (length > 65536) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    if (!arena || !output || !lanes || lanes > NEOCLR_NATIVE_LAYOUT_LANES) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    uint64_t bytes = 24 + (uint64_t)length * (8 * lanes + 1);
    int32_t status = reserve_inputs(arena, bytes, RECORDS_STORAGE, &storage, rooted, NULL, NULL);
    if (status) return status;
    uint64_t header[] = {UINT64_C(0x80000005), (uint64_t)length, lanes};
    memcpy(storage, header, sizeof(header));
    memset((unsigned char *)storage + 24, 0, (size_t)bytes - 24);
    *output = storage;
    return NEOCLR_AOT_FAULT_NONE;
}

/* Ordinary nominal reference arrays use the same traced one-lane record layout.
 * Mark null slots initialized before publication; reservation remains unchanged. */
static int32_t allocate_references(neoclr_aot_text_arena *arena, int32_t length,
                                    void **output, int rooted) {
    if (!output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    int32_t status = reserve_records(arena, length, 1, &storage, rooted);
    if (status) return status;
    memset((unsigned char *)storage + 24 + (size_t)length * 8, 1, (size_t)length);
    *output = storage;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_allocate_strings_v1(neoclr_aot_text_arena *arena, int32_t length, int32_t reserved, void **output) {
    return allocate_strings(arena, length, reserved, output, 0);
}
int32_t neoclr_allocate_strings_rooted_v1(neoclr_aot_text_arena *arena, int32_t length, int32_t reserved, void **output) {
    return allocate_strings(arena, length, reserved, output, 1);
}
int32_t neoclr_reserve_records_v1(neoclr_aot_text_arena *arena, int32_t length, uint32_t lanes, void **output) {
    return reserve_records(arena, length, lanes, output, 0);
}
int32_t neoclr_reserve_records_rooted_v1(neoclr_aot_text_arena *arena, int32_t length, uint32_t lanes, void **output) {
    return reserve_records(arena, length, lanes, output, 1);
}
int32_t neoclr_allocate_references_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_references(arena, length, output, 0);
}
int32_t neoclr_allocate_references_rooted_v1(neoclr_aot_text_arena *arena, int32_t length, void **output) {
    return allocate_references(arena, length, output, 1);
}

static int32_t allocate_scalars(neoclr_aot_text_arena *arena, int32_t length,
                                 int32_t reserved, void **output, int rooted) {
    if (length < 0 || !arena || !output || (reserved != 0 && reserved != 1))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (length > 65536) return NEOCLR_AOT_FAULT_ARRAY_LIMIT;
    void *storage;
    uint64_t bytes = 24 + (uint64_t)length * 9;
    /* Scalar bits, even pointer-shaped integers, must not retain other objects. */
    int32_t status = reserve_primitive_storage(arena, bytes, &storage, rooted);
    if (status) return status;
    uint64_t header[] = {UINT64_C(0x80000006), (uint64_t)length, 1};
    memcpy(storage, header, sizeof(header));
    memset((unsigned char *)storage + 24, 0, (size_t)length * 8);
    memset((unsigned char *)storage + 24 + (size_t)length * 8, reserved ? 0 : 1, (size_t)length);
    *output = storage;
    return NEOCLR_AOT_FAULT_NONE;
}

int32_t neoclr_allocate_scalars_v1(neoclr_aot_text_arena *arena, int32_t length,
    int32_t reserved, void **output) {
    return allocate_scalars(arena, length, reserved, output, 0);
}
int32_t neoclr_allocate_scalars_rooted_v1(neoclr_aot_text_arena *arena, int32_t length,
    int32_t reserved, void **output) {
    return allocate_scalars(arena, length, reserved, output, 1);
}

int32_t neoclr_string_compare_ordinal_v1(const neoclr_aot_text *left,
        const neoclr_aot_text *right, int32_t *output) {
    if (!left || !right || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t length = left->length < right->length ? left->length : right->length;
    int order = memcmp(left->bytes, right->bytes, (size_t)length);
    *output = order < 0 ? -1 : order > 0 ? 1 :
        left->length < right->length ? -1 : left->length > right->length ? 1 : 0;
    return NEOCLR_AOT_FAULT_NONE;
}


static int32_t path_combine_unix(const neoclr_aot_text *left, const neoclr_aot_text *right,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    if (!left || !right || !arena || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (!left->length || (right->length && right->bytes[0] == '/'))
        return store_text_input((const char *)right->bytes, (size_t)right->length, arena, output, rooted, right);
    if (!right->length)
        return store_text_input((const char *)left->bytes, (size_t)left->length, arena, output, rooted, left);
    if (left->length > UINT64_MAX - 9 || right->length > UINT64_MAX - 9 - left->length)
        return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    int separator = left->bytes[left->length - 1] != '/';
    uint64_t length = left->length + right->length + (uint64_t)separator;
    void *storage;
    int32_t status = reserve_text_inputs(arena, 8 + length, &storage, rooted, left, right);
    if (status) return status;
    neoclr_aot_text *text = storage;
    text->length = length;
    memcpy(text->bytes, left->bytes, (size_t)left->length);
    if (separator) text->bytes[left->length] = '/';
    memcpy(text->bytes + left->length + separator, right->bytes, (size_t)right->length);
    *output = text;
    return NEOCLR_AOT_FAULT_NONE;
}

static int32_t path_file_name_unix(const neoclr_aot_text *path,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    if (!path || !arena || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t start = path->length;
    while (start && path->bytes[start - 1] != '/') start--;
    return store_text_input((const char *)path->bytes + start, (size_t)(path->length - start), arena, output, rooted, path);
}


static int32_t string_replace_ordinal(const neoclr_aot_text *text,
    const neoclr_aot_text *old_value, const neoclr_aot_text *new_value,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    if (!text || !old_value || !new_value || !old_value->length || !arena || !output ||
        arena->used > arena->capacity || (arena->capacity && !arena->data) ||
        ((uintptr_t)arena->data & 7)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (old_value->length > text->length ||
        (old_value->length == new_value->length &&
         !memcmp(old_value->bytes, new_value->bytes, (size_t)old_value->length))) {
        *output = text;
        return 0;
    }
    uint64_t length = text->length, count = 0;
    for (uint64_t at = 0; at <= text->length - old_value->length;) {
        if (memcmp(text->bytes + at, old_value->bytes, (size_t)old_value->length)) { ++at; continue; }
        if (new_value->length >= old_value->length) {
            uint64_t growth = new_value->length - old_value->length;
            if (growth > UINT64_MAX - length) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
            length += growth;
        } else { length -= old_value->length - new_value->length; }
        ++count;
        at += old_value->length;
    }
    if (!count) { *output = text; return 0; }
    if (length > UINT64_MAX - 8) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    void *storage;
    const void *values[] = {text, old_value, new_value};
    int32_t status = reserve_values(arena, 8 + length, TEXT_STORAGE, &storage, rooted, values, 3);
    if (status) return status;
    neoclr_aot_text *result = storage;
    result->length = length;
    uint64_t start = 0, written = 0;
    for (uint64_t at = 0; at <= text->length - old_value->length;) {
        if (memcmp(text->bytes + at, old_value->bytes, (size_t)old_value->length)) { ++at; continue; }
        memcpy(result->bytes + written, text->bytes + start, (size_t)(at - start));
        written += at - start;
        memcpy(result->bytes + written, new_value->bytes, (size_t)new_value->length);
        written += new_value->length;
        at += old_value->length;
        start = at;
    }
    memcpy(result->bytes + written, text->bytes + start, (size_t)(text->length - start));
    *output = result;
    return 0;
}

int32_t neoclr_path_combine_unix_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return path_combine_unix(left, right, arena, output, 0);
}
int32_t neoclr_path_combine_unix_rooted_v1(const neoclr_aot_text *left, const neoclr_aot_text *right,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return path_combine_unix(left, right, arena, output, 1);
}
int32_t neoclr_path_file_name_unix_v1(const neoclr_aot_text *path,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return path_file_name_unix(path, arena, output, 0);
}
int32_t neoclr_path_file_name_unix_rooted_v1(const neoclr_aot_text *path,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return path_file_name_unix(path, arena, output, 1);
}
int32_t neoclr_string_replace_ordinal_v1(const neoclr_aot_text *text, const neoclr_aot_text *old_value, const neoclr_aot_text *new_value,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return string_replace_ordinal(text, old_value, new_value, arena, output, 0);
}
int32_t neoclr_string_replace_ordinal_rooted_v1(const neoclr_aot_text *text, const neoclr_aot_text *old_value, const neoclr_aot_text *new_value,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return string_replace_ordinal(text, old_value, new_value, arena, output, 1);
}

/* Private writable construction buffer. The caller initializes every byte before
 * publishing; no collection/reentry is allowed while construction is in flight. */
int32_t neoclr_text_allocate_v1(neoclr_aot_text_arena *arena, uint64_t length,
                               neoclr_aot_text **output) {
    if (!output || length > INT32_MAX) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    int32_t status = reserve_storage(arena, 8 + length, TEXT_STORAGE, &storage);
    if (status) return status;
    neoclr_aot_text *text = storage;
    text->length = length;
    *output = text;
    return 0;
}

/* Writable text construction with explicit borrowed owners. No collection may
 * occur between returning this buffer and initializing/publishing its payload. */
int32_t neoclr_text_allocate_rooted_values_v1(neoclr_aot_text_arena *arena,
    uint64_t length, const void *const *owners, uint32_t count, neoclr_aot_text **output) {
    if (!output || length > INT32_MAX) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    void *storage;
    int32_t status = reserve_values(arena, 8 + length, TEXT_STORAGE, &storage, 1, owners, count);
    if (status) return status;
    neoclr_aot_text *text = storage;
    text->length = length;
    *output = text;
    return 0;
}

/* Byte-window kernel. Keep the allocation base rather than an interior pointer:
 * GC can trace exactly one owner while bounds stay relative to the current view. */
static int32_t byte_view_storage(neoclr_aot_context *context, const void *owner,
    uint64_t *length, int *reserved) {
    if (!context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (!owner) { *length = 0; *reserved = 0; return 0; }
    uint64_t extent;
#ifdef NEOCLR_NATIVE_GC
    uint32_t allocation_kind;
    int32_t status = neoclr_gc_allocation_info_v1(context, owner, &extent, &allocation_kind);
    if (status || allocation_kind != NEOCLR_GC_BYTES) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
#else
    const neoclr_aot_text_arena *arena = &context->text;
    uintptr_t base = (uintptr_t)arena->data, address = (uintptr_t)owner;
    if (!base || arena->used > arena->capacity || address < base || (address & 7) ||
        address - base > arena->used) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    extent = arena->used - (address - base);
#endif
    if (extent < 16) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t tag, count;
    memcpy(&tag, owner, 8);
    memcpy(&count, (const unsigned char *)owner + 8, 8);
    tag &= UINT64_C(0xffffffff);
    if ((tag != UINT64_C(0x80000001) && tag != UINT64_C(0x80000002)) ||
        count > NEOCLR_NATIVE_BYTE_ARRAY_LIMIT) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    int pending = tag == UINT64_C(0x80000002);
    if (count > (extent - 16) / (pending ? 2 : 1)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    *length = count; *reserved = pending;
    return 0;
}
static int32_t byte_view_validate(neoclr_aot_context *context, const neoclr_byte_view *view,
    uint64_t *total, int *reserved) {
    if (!view || view->kind > NEOCLR_VIEW_READONLY_MEMORY) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    int32_t status = byte_view_storage(context, view->owner, total, reserved);
    if (status) return status;
    if (view->offset > *total || view->length > *total - view->offset)
        return NEOCLR_AOT_FAULT_INDEX_OUT_OF_RANGE;
    return 0;
}
int32_t neoclr_byte_view_create_v1(neoclr_aot_context *context, void *owner,
    int32_t start, int32_t count, uint64_t kind, neoclr_byte_view *output) {
    if (!output || kind > NEOCLR_VIEW_READONLY_MEMORY) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t total; int reserved;
    int32_t status = byte_view_storage(context, owner, &total, &reserved);
    if (status) return status;
    if (start < 0 || count < 0 || (uint64_t)start > total || (uint64_t)count > total - (uint64_t)start)
        return NEOCLR_AOT_FAULT_INDEX_OUT_OF_RANGE;
    *output = (neoclr_byte_view){owner, (uint32_t)start, (uint32_t)count, kind};
    return 0;
}
int32_t neoclr_byte_view_create_all_v1(neoclr_aot_context *context, void *owner,
    uint64_t kind, neoclr_byte_view *output) {
    uint64_t total; int reserved;
    int32_t status = byte_view_storage(context, owner, &total, &reserved);
    if (status) return status;
    if (total > INT32_MAX) return NEOCLR_AOT_FAULT_INDEX_OUT_OF_RANGE;
    return neoclr_byte_view_create_v1(context, owner, 0, (int32_t)total, kind, output);
}
int32_t neoclr_byte_view_slice_v1(neoclr_aot_context *context, const neoclr_byte_view *view,
    int32_t start, int32_t count, neoclr_byte_view *output) {
    if (!output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t total; int reserved;
    int32_t status = byte_view_validate(context, view, &total, &reserved);
    if (status) return status;
    if (start < 0 || count < 0 || (uint32_t)start > view->length || (uint32_t)count > view->length - (uint32_t)start)
        return NEOCLR_AOT_FAULT_INDEX_OUT_OF_RANGE;
    neoclr_byte_view result = *view;
    result.offset += (uint32_t)start; result.length = (uint32_t)count;
    *output = result;
    return 0;
}
int32_t neoclr_byte_view_convert_v1(neoclr_aot_context *context, const neoclr_byte_view *view,
    uint64_t kind, neoclr_byte_view *output) {
    if (!output || kind > NEOCLR_VIEW_READONLY_MEMORY) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t total; int reserved;
    int32_t status = byte_view_validate(context, view, &total, &reserved);
    if (status) return status;
    if (((view->kind & 1) && !(kind & 1)) || (view->kind < NEOCLR_VIEW_MEMORY && kind >= NEOCLR_VIEW_MEMORY))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    neoclr_byte_view result = *view; result.kind = kind; *output = result;
    return 0;
}
static int32_t byte_view_position(neoclr_aot_context *context, const neoclr_byte_view *view,
    int32_t index, uint64_t *position, uint64_t *total, int *reserved) {
    int32_t status = byte_view_validate(context, view, total, reserved);
    if (status) return status;
    if (index < 0 || (uint32_t)index >= view->length) return NEOCLR_AOT_FAULT_INDEX_OUT_OF_RANGE;
    *position = (uint64_t)view->offset + (uint32_t)index;
    return 0;
}
int32_t neoclr_byte_view_get_v1(neoclr_aot_context *context, const neoclr_byte_view *view,
    int32_t index, uint8_t *output) {
    if (!output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t position, total; int reserved;
    int32_t status = byte_view_position(context, view, index, &position, &total, &reserved);
    if (status) return status;
    const unsigned char *data = (const unsigned char *)view->owner + 16;
    if (reserved && !data[total + position]) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    *output = data[position];
    return 0;
}
int32_t neoclr_byte_view_set_v1(neoclr_aot_context *context, const neoclr_byte_view *view,
    int32_t index, uint8_t value) {
    uint64_t position, total; int reserved;
    int32_t status = byte_view_position(context, view, index, &position, &total, &reserved);
    if (status) return status;
    if (view->kind & 1) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    unsigned char *data = (unsigned char *)view->owner + 16;
    data[position] = value;
    if (reserved) data[total + position] = 1;
    return 0;
}
int32_t neoclr_byte_view_copy_v1(neoclr_aot_context *context,
    const neoclr_byte_view *source, const neoclr_byte_view *destination) {
    uint64_t source_total, destination_total;
    int source_reserved, destination_reserved;
    int32_t status = byte_view_validate(context, source, &source_total, &source_reserved);
    if (status) return status;
    status = byte_view_validate(context, destination, &destination_total, &destination_reserved);
    if (status) return status;
    if (destination->kind & 1) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (source->length > destination->length) return NEOCLR_AOT_FAULT_INDEX_OUT_OF_RANGE;
    /* Avoid pointer arithmetic on the null owner of a valid empty view. */
    if (!source->length) return 0;
    const unsigned char *input = (const unsigned char *)source->owner + 16;
    unsigned char *output = (unsigned char *)destination->owner + 16;
    if (source_reserved) {
        for (uint32_t i = 0; i < source->length; i++) {
            if (!input[source_total + source->offset + i])
                return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        }
    }
    /* Snapshot semantics without a temporary allocation, in either direction.
     * Publish initialization only after the full source has been checked. */
    memmove(output + destination->offset, input + source->offset, source->length);
    if (destination_reserved)
        memset(output + destination_total + destination->offset, 1, source->length);
    return 0;
}
