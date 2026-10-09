#include "../aot-console/text-arena.h"
#include <stdio.h>

int main(void) {
    uint64_t storage[8192];
    neoclr_aot_context context = { .text = { (unsigned char *)storage, sizeof(storage), 0 } };
    int32_t result = -1;
    int32_t status = neoclr_entry_v4(0, &result, &context);
    if (status != 0 || result != 42 || context.fault.code ||
        context.fault.frame_count || context.fault.message) {
        fprintf(stderr, "status=%d result=%d\n", status, result);
        return 1;
    }
    printf("%d\n", result);
    return 0;
}
