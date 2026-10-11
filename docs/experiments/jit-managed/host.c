/* Private JIT entry host. Managed awaits drain within this call; no state escapes. */
#include "native-gc.h"
#include "task-queue.h"
#include <stdlib.h>

typedef int32_t (*entry_v4)(int32_t, int32_t *, neoclr_aot_context *);
typedef int32_t (*entry_args)(int32_t, int32_t *, neoclr_aot_context *, const char **);

int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const char **argv) {
    if (!entry || argc < 1 || !argv || (with_args != 0 && with_args != 1)) return 2;
    const size_t heap_bytes = 1024 * 1024;
    unsigned char *heap = calloc(1, heap_bytes);
    if (!heap) return 2;
    neoclr_aot_context context = {.text = {heap, heap_bytes, 0}};
    neoclr_task_scope tasks;
    if (neoclr_task_scope_enter_v1(&tasks, &context)) { free(heap); return 2; }
    int32_t result = 0;
    int32_t status = with_args
        ? ((entry_args)entry)(argc, &result, &context, argv)
        : ((entry_v4)entry)(0, &result, &context);
    int render_error = status && neoclr_aot_render_fault(stderr, &context.fault);
    /* Cleanup must not return to a caller about to unload JIT code while any
       frame/handle still owns its descriptors. Treat a broken invariant as fatal. */
    if (neoclr_root_probe_head_v1() != NULL ||
        neoclr_task_scope_leave_v1(&tasks) ||
        neoclr_gc_owned_roots_check_v1(&context, NULL, 0) ||
        neoclr_gc_collect_v1(&context, NULL)) abort();
    free(heap);
    return render_error ? 2 : status ? 1 : result;
}
