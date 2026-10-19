/* Private synchronous byte-slice adapter shared by JIT and AOT. Never collects,
 * calls guest code, retains pointers, or owns guest storage. */
#include "../../../runtime/native/memory/memory-views.h"
#include "../../../tools/native-crypto/include/neoclr_crypto.h"
#include <string.h>

int32_t neoclr_cryptography_slices_v1(int32_t operation, neoclr_aot_context *context,
    const neoclr_byte_view *first, const neoclr_byte_view *second,
    const neoclr_byte_view *third, int32_t *result) {
    if (!result || operation < 1 || operation > 4) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    neoclr_byte_access a = {0}, b = {0}, c = {0}, output = {0};
    int32_t status;
    if (operation != 3) {
        const neoclr_byte_view *destination = operation == 1 ? second : operation == 2 ? third : first;
        if (!destination || destination->kind != NEOCLR_VIEW_SLICE) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        status = neoclr_byte_view_access_v1(context, destination, 1, &output);
        if (status) return status;
        if (operation != 4 && output.count < 32) { *result = -1; return 0; }
    }
    if (operation != 4) {
        if (!first || first->kind != NEOCLR_VIEW_READONLY_SLICE) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        status = neoclr_byte_view_access_v1(context, first, 0, &a);
        if (status) return status;
        if (operation != 1) {
            if (!second || second->kind != NEOCLR_VIEW_READONLY_SLICE) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
            status = neoclr_byte_view_access_v1(context, second, 0, &b);
            if (status) return status;
        }
    }
    if (operation == 1) status = neoclr_crypto_sha256(a.bytes, a.count, output.bytes, output.count);
    else if (operation == 2) status = neoclr_crypto_hmac_sha256(a.bytes, a.count, b.bytes, b.count, output.bytes, output.count);
    else if (operation == 3) {
        if (!third || third->kind != NEOCLR_VIEW_READONLY_SLICE) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        status = neoclr_byte_view_access_v1(context, third, 0, &c);
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
