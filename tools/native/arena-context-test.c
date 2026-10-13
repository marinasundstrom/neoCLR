/* Shared arena service contract, linked both with and without the collector. */
#include "text-arena.h"
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
int main(void) {
    uint64_t heap[128], other_heap[128];
    memset(heap, 0xa5, sizeof(heap));
    memset(other_heap, 0xa5, sizeof(other_heap));
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_aot_context other = {.text = {(unsigned char *)other_heap, sizeof(other_heap), 0}};
    c.fault.code = NEOCLR_AOT_FAULT_USER_FAULT;
    void *object = NULL, *bytes = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, 17, 24, &object));
    CHECK(((uintptr_t)object & 7) == 0 && ((uint64_t *)object)[0] == 17);
    CHECK(((uint64_t *)object)[1] == 0 && ((uint64_t *)object)[2] == 0);
    CHECK(!neoclr_allocate_bytes_v1(&c.text, 5, &bytes));
    CHECK(((uint64_t *)bytes)[1] == 5);
    for (unsigned i = 0; i < 5; i++) CHECK(((unsigned char *)bytes)[16+i] == 0);
    void *saved = bytes;
    uint64_t used = c.text.used;
    CHECK(neoclr_allocate_bytes_v1(&c.text, -1, &bytes) == NEOCLR_AOT_FAULT_RUNTIME_ERROR);
    CHECK(neoclr_allocate_bytes_v1(&c.text, NEOCLR_NATIVE_BYTE_ARRAY_LIMIT+1, &bytes) == NEOCLR_AOT_FAULT_ARRAY_LIMIT);
    CHECK(neoclr_allocate_object_v1(&c.text, 1, 9, &bytes) == NEOCLR_AOT_FAULT_RUNTIME_ERROR);
    CHECK(bytes == saved && c.text.used == used);
    static const struct { uint64_t length; unsigned char bytes[4]; } literal = {4, {0xc3,0xa9,0,'x'}};
    const neoclr_aot_text *text = (const void *)&literal, *joined = NULL;
    CHECK(!neoclr_string_concat_v1(text, text, &c.text, &joined));
    CHECK(joined->length == 8 && !memcmp(joined->bytes, literal.bytes, 4) && !memcmp(joined->bytes+4, literal.bytes, 4));
    CHECK(other.text.used == 0 && c.fault.code == NEOCLR_AOT_FAULT_USER_FAULT);
    CHECK(!neoclr_allocate_object_v1(&other.text, 23, 16, &bytes));
    CHECK(((uint64_t *)bytes)[0] == 23 && ((uint64_t *)object)[0] == 17);
    uint64_t empty_heap[1] = {0};
    neoclr_aot_text_arena empty = {(unsigned char *)empty_heap, 0, 0};
    bytes = saved;
    CHECK(neoclr_allocate_bytes_v1(&empty, 1, &bytes) == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
    CHECK(bytes == saved && empty.used == 0);
#ifdef NEOCLR_NATIVE_GC
    c.fault.code = 0;
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used == 0);
    CHECK(other.text.used != 0);
    CHECK(!neoclr_gc_collect_v1(&other, NULL) && other.text.used == 0);
#else
    /* Arena-only callers own the reset lifetime; no collector is linked. */
    c.text.used = 0;
    CHECK(!neoclr_allocate_object_v1(&c.text, 31, 24, &bytes));
    CHECK(bytes == (void *)heap && ((uint64_t *)bytes)[0] == 31);
#endif
    return 0;
}
