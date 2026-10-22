/* Development scalar native-interop host. No managed heap or service bindings. */
#include "abi.h"
#include <stdio.h>

int main(void) {
    int32_t result = 0;
    const int32_t status = neoclr_entry_v2(0, &result);
    if (status != NEOCLR_AOT_OK) {
        fprintf(stderr, "neoCLR native interop fault: %d\n", (int)status);
        return 1;
    }
    return result;
}
