#include "fault-details.h"
#include <stdlib.h>
#include <inttypes.h>
int main(int argc, char **argv) {
    int32_t result = 12345;
    neoclr_aot_fault fault;
    int32_t input = argc > 1 ? (int32_t)strtol(argv[1], NULL, 10) : 0;
    int32_t status = neoclr_entry_v3(input, &result, &fault);
    if (status) {
        if (result != 12345 || fault.code != (uint32_t)status) return 3;
        return neoclr_aot_render_fault(stderr, &fault) ? 2 : 1;
    }
    if (fault.code || fault.frame_count || fault.message) return 4;
    return result;
}
