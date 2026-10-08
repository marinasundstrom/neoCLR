#include <stdint.h>
#include <stdio.h>

extern int32_t neoclr_entry_v2(int32_t argument, int32_t *result);

int main(void) {
    int32_t result = -1;
    int32_t status = neoclr_entry_v2(0, &result);
    if (status != 0 || result != 42) {
        fprintf(stderr, "status=%d result=%d\n", status, result);
        return 1;
    }
    printf("%d\n", result);
    return 0;
}
