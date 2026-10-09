/* Experimental synchronous console profile; private ABI, not a hosting API. */
#include "native-gc.h"
#include <signal.h>
#include <stdlib.h>

int main(void) {
    if (signal(SIGPIPE, SIG_IGN) == SIG_ERR) return 2;
    const size_t heap_bytes = 1024 * 1024;
    unsigned char *heap = calloc(1, heap_bytes);
    if (!heap) return 2;
    neoclr_aot_context context = {.text = {heap, heap_bytes, 0}};
    int32_t result = 0;
    int32_t status = neoclr_entry_v4(0, &result, &context);
    int host_error = neoclr_root_probe_head_v1() != NULL;
    if (status && neoclr_aot_render_fault(stderr, &context.fault)) host_error = 1;
    if (!host_error && neoclr_gc_collect_v1(&context, NULL)) host_error = 1;
    if (host_error) fputs("neoCLR native console cleanup failed\n", stderr);
    free(heap);
    return host_error ? 2 : status ? 1 : result;
}
