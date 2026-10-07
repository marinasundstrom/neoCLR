#include "text-arena.h"
#include <signal.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    if (signal(SIGPIPE, SIG_IGN) == SIG_ERR) return 2;
    /* Bounded example policy, not a platform default or a managed heap. */
    uint64_t storage[8192];
    neoclr_aot_context context = { .text = { (unsigned char *)storage, sizeof(storage), 0 } };
    int32_t result = 12345;
    int32_t input = argc > 1 ? (int32_t)strtol(argv[1], NULL, 10) : 0;
    int32_t status = neoclr_entry_v4(input, &result, &context);
    if (status) {
        if (result != 12345 || context.fault.code != (uint32_t)status) return 3;
        return neoclr_aot_render_fault(stderr, &context.fault) ? 2 : 1;
    }
    if (context.fault.code || context.fault.frame_count || context.fault.message) return 4;
    return result;
}
