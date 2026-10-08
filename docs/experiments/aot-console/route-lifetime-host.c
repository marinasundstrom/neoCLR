#include "text-arena.h"
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>

/* Diagnostic harness: capacity is a measurement variable, not server policy. */
int main(int argc, char **argv) {
    if (signal(SIGPIPE, SIG_IGN) == SIG_ERR) return 2;
    int32_t requests = argc > 1 ? (int32_t)strtol(argv[1], NULL, 10) : 16;
    size_t capacity = argc > 2 ? (size_t)strtoull(argv[2], NULL, 10) : 1048576;
    int audit = argc > 3 && strcmp(argv[3], "audit") == 0;
    if (requests < 0 || requests > 1024 || capacity > 16777216) return 2;
    unsigned char *storage = malloc(capacity + 8);
    if (!storage) return 2;
    memset(storage + capacity, 0xa5, 8);
    neoclr_aot_context context = {.text = {storage, capacity, 0}};
    int32_t result = -99;
    int32_t status = neoclr_entry_v4(requests, &result, &context);
    for (int i = 0; i < 8; ++i) if (storage[capacity + i] != 0xa5) return 3;
    if (context.text.used > capacity || (status && (result != -99 || context.fault.code != (uint32_t)status))) return 3;
    if (!status && (context.fault.code || context.fault.frame_count || context.fault.message)) return 3;
    if (status) neoclr_aot_render_fault(stderr, &context.fault);
    if (audit) fprintf(stderr, "AOT_MEASURE {\"requests\":%d,\"capacity\":%zu,\"used\":%" PRIu64 ",\"status\":%d,\"result\":%d}\n",
                       requests, capacity, context.text.used, status, result);
    free(storage);
    return status ? 1 : result;
}
