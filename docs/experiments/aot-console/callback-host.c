#include "native-gc.h"
int main(void) {
    uint64_t buffer[8193];
    buffer[8192] = UINT64_C(0x1122334455667788);
    neoclr_aot_context context = {.text = {(unsigned char *)buffer, 65536, 0}};
    int32_t result = -99;
    int32_t status = neoclr_entry_v4(0, &result, &context);
    if (neoclr_root_probe_head_v1() || buffer[8192] != UINT64_C(0x1122334455667788)) return 2;
    if (status) {
        if (result != -99 || context.fault.code != (uint32_t)status) return 2;
        neoclr_aot_render_fault(stderr, &context.fault);
        return 1;
    }
    if (neoclr_gc_collect_v1(&context, NULL) || context.text.used) return 2;
    return result;
}
