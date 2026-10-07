#include "../aot-scalar/abi.h"
#include <stdio.h>

/* Temporary native startup adapter; generated program returns the exit status. */
int main(void) {
    int32_t result = 1;
    int32_t status = neoclr_entry_v2(0, &result);
    if (status != NEOCLR_AOT_OK) {
        fprintf(stderr, "neoCLR AOT invocation failed: %d\n", status);
        return 1;
    }
    return result;
}
