/* Deterministic provider failures and validation ordering; actual known answers
 * are covered by the Raven suite and shared provider tests. */
#include "../aot-console/cryptography.c"
#ifdef NEOCLR_NATIVE_GC
#include "../../../runtime/native/gc/native-gc.h"
#endif
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed at %d: %s\n", __LINE__, #x); return 1; } } while (0)
static unsigned calls;
int32_t neoclr_crypto_sha256(const uint8_t *data, uint32_t count, uint8_t *output, uint32_t capacity) {
    (void)data; (void)count; (void)capacity; calls++;
    memset(output, 42, 32); return 0;
}
int32_t neoclr_crypto_hmac_sha256(const uint8_t *key, uint32_t key_count,
    const uint8_t *data, uint32_t count, uint8_t *output, uint32_t capacity) {
    (void)key; (void)key_count;
    return neoclr_crypto_sha256(data, count, output, capacity);
}
int32_t neoclr_crypto_hmac_sha256_verify(const uint8_t *key, uint32_t key_count,
    const uint8_t *data, uint32_t count, const uint8_t *tag, uint32_t tag_count) {
    (void)key; (void)key_count; (void)data; (void)count; (void)tag; (void)tag_count;
    calls++; return 1;
}
int32_t neoclr_crypto_random_fill(uint8_t *output, uint32_t count) {
    if (!count) return 0;
    calls++; memset(output, 0, count); return -2;
}
int main(void) {
    uint64_t heap[512] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *array;
    CHECK(!neoclr_reserve_bytes_v1(&c.text, 36, &array));
    unsigned char *bytes = (unsigned char *)array + 16;
    memset(bytes, 165, 36);
    neoclr_byte_view input, output;
    CHECK(!neoclr_byte_view_create_v1(&c, array, 1, 3, NEOCLR_VIEW_READONLY_SLICE, &input));
    CHECK(!neoclr_byte_view_create_v1(&c, array, 1, 34, NEOCLR_VIEW_SLICE, &output));
    int32_t result = 99;
    CHECK(neoclr_cryptography_slices_v1(1, &c, &input, &output, NULL, &result) == 3);
    CHECK(result == 99 && !calls && bytes[1] == 165 && !bytes[37]);
    output.length = 31;
    CHECK(!neoclr_cryptography_slices_v1(1, &c, &input, &output, NULL, &result));
    CHECK(result == -1 && !calls && bytes[1] == 165);
    output.length = 34;
    memset(bytes + 37, 1, 3);
    CHECK(!neoclr_cryptography_slices_v1(1, &c, &input, &output, NULL, &result));
    CHECK(!result && calls == 1 && bytes[1] == 42 && bytes[32] == 42);
    CHECK(bytes[0] == 165 && bytes[33] == 165 && bytes[35] == 165);
    CHECK(!bytes[36] && bytes[68] && !bytes[69] && !bytes[71]);
    CHECK(!neoclr_cryptography_slices_v1(4, &c, &output, NULL, NULL, &result));
    CHECK(result == -2 && calls == 2);
    for (int i = 1; i <= 34; i++) CHECK(!bytes[i] && bytes[36 + i]);
    CHECK(bytes[0] == 165 && bytes[35] == 165 && !bytes[36] && !bytes[71]);
    output.kind = NEOCLR_VIEW_READONLY_SLICE;
    result = 99;
    CHECK(neoclr_cryptography_slices_v1(4, &c, &output, NULL, NULL, &result) == 3);
    CHECK(result == 99 && calls == 2);
    output = (neoclr_byte_view){0};
    CHECK(!neoclr_cryptography_slices_v1(4, &c, &output, NULL, NULL, &result));
    CHECK(!result && calls == 2);
#ifdef NEOCLR_NATIVE_GC
    CHECK(!neoclr_gc_collect_v1(&c, NULL));

#endif
    return 0;
}
