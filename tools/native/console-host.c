#ifdef NEOCLR_MANAGED_TLS
#include "tls-managed.h"
#endif
#include "entry-arguments.h"
/* Experimental synchronous console profile; private ABI, not a hosting API. */
#include "native-gc.h"
#include "console.h"
#include <signal.h>
#include <stdlib.h>

int NEOCLR_PROCESS_MAIN(int argc, neoclr_process_char **argv) {
    if (signal(SIGPIPE, SIG_IGN) == SIG_ERR) return 2;
    const size_t heap_bytes = 1024 * 1024;
    unsigned char *heap = calloc(1, heap_bytes);
    if (!heap) return 2;
    neoclr_aot_context context = {.text = {heap, heap_bytes, 0}};
    neoclr_terminal_scope_v1 terminal = {0};
    if (neoclr_terminal_scope_enter_v1(&terminal, &context)) { free(heap); return 2; }
    int32_t result = 0;
    int32_t status = neoclr_process_entry(argc, argv, &result, &context);
    int host_error = neoclr_terminal_scope_leave_v1(&terminal) != 0;
    if (neoclr_root_probe_head_v1() != NULL) host_error = 1;
    if (status && neoclr_aot_render_fault(stderr, &context.fault)) host_error = 1;
    if (!host_error && neoclr_gc_collect_v1(&context, NULL)) host_error = 1;
    if (host_error) fputs("neoCLR native console cleanup failed\n", stderr);
#ifdef NEOCLR_MANAGED_TLS
    neoclr_tls_managed_dispose_v1(&context);
#endif
    free(heap);
    return host_error ? 2 : status ? 1 : result;
}
