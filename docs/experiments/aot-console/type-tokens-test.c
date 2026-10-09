#include "native-gc.h"
#include <stdio.h>
int main(void) {
    uint64_t buffer[257] = {0};
    buffer[256] = 1234567;
    neoclr_aot_context context = {.text = {(unsigned char *)buffer, 2048, 0}};
    int32_t result = 0;
    int32_t status = neoclr_entry_v4(0, &result, &context);
    if (status || result != 42 || context.fault.code || neoclr_root_probe_depth_v1()) return 1;
    if (neoclr_gc_collect_v1(&context, NULL) || context.text.used || buffer[256] != 1234567) return 2;
    puts("Type tokens: 42");
    return 0;
}
