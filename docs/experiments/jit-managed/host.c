/* Private JIT entry host. Managed awaits drain within this call; no state escapes. */
#include "native-gc.h"
#include "task-queue.h"
#include "socket-listener.h"
#include <stdlib.h>

typedef int32_t (*entry_v4)(int32_t, int32_t *, neoclr_aot_context *);
typedef int32_t (*entry_args)(int32_t, int32_t *, neoclr_aot_context *, const char **);

/* Timer operations currently share the native I/O table. The generated entry
   consumes each delivered callback before polling again, retaining its root. */
static int32_t poll_entry(neoclr_aot_context *context, uint64_t *callback,
    const neoclr_probe_frame *boundary, void *state) {
    neoclr_socket_scope *operations = state;
    unsigned active = 0;
    for (unsigned i = 0; i < 64; ++i) active += operations->operations[i].state != 0;
    if (!active) return 0;
    int32_t ready = neoclr_socket_poll_suspended_v1(context, callback, boundary);
    if (ready) return ready;
    neoclr_os_pause();
    return 2;
}

int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const char **argv) {
    if (!entry || argc < 1 || !argv || (with_args != 0 && with_args != 1)) return 2;
    const size_t heap_bytes = 1024 * 1024;
    unsigned char *heap = calloc(1, heap_bytes);
    if (!heap) return 2;
    neoclr_aot_context context = {.text = {heap, heap_bytes, 0}};
    neoclr_socket_scope operations;
    if (neoclr_socket_scope_enter_v1(&operations, &context)) { free(heap); return 2; }
    neoclr_task_scope tasks;
    if (neoclr_task_scope_enter_v1(&tasks, &context)) {
        if (neoclr_socket_scope_leave_v1(&operations)) abort();
        free(heap);
        return 2;
    }
    tasks.poll = poll_entry;
    tasks.poll_state = &operations;
    int32_t result = 0;
    int32_t status = with_args
        ? ((entry_args)entry)(argc, &result, &context, argv)
        : ((entry_v4)entry)(0, &result, &context);
    int render_error = status && neoclr_aot_render_fault(stderr, &context.fault);
    /* Cleanup must not return to a caller about to unload JIT code while any
       frame/handle still owns its descriptors. Treat a broken invariant as fatal. */
    if (neoclr_root_probe_head_v1() != NULL ||
        neoclr_socket_scope_leave_v1(&operations) ||
        neoclr_task_scope_leave_v1(&tasks) ||
        neoclr_gc_owned_roots_check_v1(&context, NULL, 0) ||
        neoclr_gc_collect_v1(&context, NULL)) abort();
    free(heap);
    return render_error ? 2 : status ? 1 : result;
}
