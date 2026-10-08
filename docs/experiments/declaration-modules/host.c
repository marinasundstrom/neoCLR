#include <stdint.h>
#include <stdio.h>
extern int32_t neoclr_entry_v2(int32_t argument, int32_t *result);
int main(void) { int32_t value = 0; int32_t status = neoclr_entry_v2(0, &value); if (status) return status; printf("%d\n", value); return value == 42 ? 0 : 1; }
