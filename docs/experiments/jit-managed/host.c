#ifdef NEOCLR_MANAGED_TLS
#include "tls-managed.h"
#endif
/* Private JIT entry host. Managed awaits drain within this call; no state escapes. */
#include "native-gc.h"
#ifdef NEOCLR_RUNTIME_ASYNC
#include "native-runtime.h"
#endif
#include "task-queue.h"
#include "socket-listener.h"
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
typedef wchar_t entry_char;
#else
typedef char entry_char;
#endif

#ifndef NEOCLR_HTTP_HEAP_BYTES
#define NEOCLR_HTTP_HEAP_BYTES (1024 * 1024)
#endif
_Static_assert(NEOCLR_HTTP_HEAP_BYTES > 0, "positive JIT heap budget");

typedef int32_t (*entry_v4)(int32_t, int32_t *, neoclr_aot_context *);
typedef int32_t (*entry_args)(int32_t, int32_t *, neoclr_aot_context *, const entry_char **);

/* Timer operations currently share the native I/O table. The generated entry
   consumes each delivered callback before polling again, retaining its root. */
static int32_t poll_entry(neoclr_aot_context *context, uint64_t *callback,
    const neoclr_probe_frame *boundary, void *state) {
    (void)state;
    int32_t progress = neoclr_scheduler_acquire_v1(context, callback, boundary);
    if (progress == NEOCLR_SCHEDULER_PENDING) neoclr_scheduler_wait_v1(context);
    return progress;
}

static int32_t run_entry(const void *entry, int32_t with_args, int32_t argc, const entry_char **argv) {
    if (!entry || argc < 1 || !argv || (with_args != 0 && with_args != 1)) return 2;
    const size_t heap_bytes = NEOCLR_HTTP_HEAP_BYTES;
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
    tasks.complete = neoclr_scheduler_complete_v1;
    int32_t result = 0;
    entry_args args_function;
    entry_v4 plain_function;
    _Static_assert(sizeof(args_function) == sizeof(entry), "native entry pointer size");
    _Static_assert(sizeof(plain_function) == sizeof(entry), "native entry pointer size");
    memcpy(&args_function, &entry, sizeof(entry));
    memcpy(&plain_function, &entry, sizeof(entry));
    int32_t status = with_args
        ? args_function(argc, &result, &context, argv)
        : plain_function(0, &result, &context);
#ifdef NEOCLR_RUNTIME_ASYNC
    if (neoclr_runtime_async_shutdown_v1(&context)) abort();
#endif
    int render_error = status && neoclr_aot_render_fault(stderr, &context.fault);
    /* Cleanup must not return to a caller about to unload JIT code while any
       frame/handle still owns its descriptors. Treat a broken invariant as fatal. */
    if (neoclr_root_probe_head_v1() != NULL ||
        neoclr_socket_scope_leave_v1(&operations) ||
        neoclr_task_scope_leave_v1(&tasks)) abort();
#ifdef NEOCLR_MANAGED_TLS
    neoclr_tls_managed_dispose_v1(&context);
#endif
    if (neoclr_gc_owned_roots_check_v1(&context, NULL, 0) ||
        neoclr_gc_collect_v1(&context, NULL)) abort();
#ifdef NEOCLR_RUNTIME_ASYNC
    memset(&context.fault, 0, sizeof(context.fault));
    if (neoclr_gc_collect_v1(&context, NULL) || context.text.used) abort();
#endif
    free(heap);
    return render_error ? 2 : status ? 1 : result;
}

/* Windows CRT translation must not change the platform's UTF-8 byte contract.
   Restore the caller's modes after this invocation, including admission failures. */
int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const entry_char **argv) {
#ifdef _WIN32
    FILE *streams[] = {stdin, stdout, stderr};
    int modes[3];
    for (int i = 0; i < 3; ++i) {
        modes[i] = _setmode(_fileno(streams[i]), _O_BINARY);
        if (modes[i] == -1) {
            while (i > 0) { --i; (void)_setmode(_fileno(streams[i]), modes[i]); }
            return 2;
        }
    }
#endif
    int32_t status = run_entry(entry, with_args, argc, argv);
#ifdef _WIN32
    for (int i = 0; i < 3; ++i) {
        if (i && fflush(streams[i])) status = 2;
        if (_setmode(_fileno(streams[i]), modes[i]) == -1) status = 2;
    }
#endif
    return status;
}
