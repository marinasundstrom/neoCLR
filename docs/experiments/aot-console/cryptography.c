/* Private synchronous byte-array adapter shared by JIT and AOT. Never collects,
 * calls guest code, retains pointers, or owns guest storage. */
#include "text-arena.h"
#include "../../../tools/native-crypto/include/neoclr_crypto.h"
#include <string.h>

typedef struct { unsigned char *bytes, *initialized; uint32_t count; } crypto_bytes;
static int32_t array(void *value, crypto_bytes *out) {
    if (!value) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    uint64_t kind, count;
    memcpy(&kind, value, 8);
    memcpy(&count, (unsigned char *)value + 8, 8);
    kind &= UINT64_C(0xffffffff);
    if ((kind != UINT64_C(0x80000001) && kind != UINT64_C(0x80000002)) ||
        count > NEOCLR_NATIVE_BYTE_ARRAY_LIMIT) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    out->count = (uint32_t)count;
    out->bytes = (unsigned char *)value + 16;
    out->initialized = kind == UINT64_C(0x80000002) ? out->bytes + count : NULL;
    return 0;
}
static int32_t input(void *value, crypto_bytes *out) {
    int32_t status = array(value, out);
    if (status) return status;
    /* Unlike stream operations, check the entire admitted array, also above 64 KiB. */
    if (out->initialized) for (uint32_t i = 0; i < out->count; ++i)
        if (!out->initialized[i]) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return 0;
}
int32_t neoclr_cryptography_v1(int32_t operation, void *first, void *second,
                             void *third, int32_t *result) {
    if (!result || operation < 1 || operation > 4) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    crypto_bytes a = {0}, b = {0}, c = {0}, output = {0};
    int32_t status;
    if (operation != 3) {
        void *destination = operation == 1 ? second : operation == 2 ? third : first;
        status = array(destination, &output);
        if (status) return status;
        if (operation != 4 && output.count < 32) { *result = -1; return 0; }
    }
    if (operation != 4) {
        status = input(first, &a);
        if (status) return status;
        if (operation != 1) {
            status = input(second, &b);
            if (status) return status;
        }
    }
    if (operation == 1) status = neoclr_crypto_sha256(a.bytes, a.count, output.bytes, output.count);
    else if (operation == 2) status = neoclr_crypto_hmac_sha256(a.bytes, a.count, b.bytes, b.count, output.bytes, output.count);
    else if (operation == 3) {
        status = input(third, &c);
        if (status) return status;
        status = neoclr_crypto_hmac_sha256_verify(a.bytes, a.count, b.bytes, b.count, c.bytes, c.count);
        if (status < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        *result = status == 0;
        return 0;
    } else status = neoclr_crypto_random_fill(output.bytes, output.count);
    if (status != 0 && !(operation == 4 && status == -2)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (output.initialized) memset(output.initialized, 1, operation == 4 ? output.count : 32);
    *result = status;
    return 0;
}
