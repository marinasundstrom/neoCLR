/* Build with -I pointing to the native-library output directory. */
#include "app.h"

int main(void) {
    int32_t *pointer = sample_allocate(42);
    if (!pointer) return 1;
    sample_write(pointer, 43);
    int32_t result = sample_read(pointer);
    sample_release(pointer);
    return result == 43 ? 0 : 2;
}
