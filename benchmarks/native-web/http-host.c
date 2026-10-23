#ifdef NEOCLR_MANAGED_TLS
#include "tls-managed.h"
#endif
#include "entry-arguments.h"
/* One-request HTTP correctness host, not a production scheduler or load benchmark. */
#include "native-gc.h"
#include "console.h"
#ifdef NEOCLR_RUNTIME_ASYNC
#include "native-runtime.h"
#endif
#include "task-queue.h"
#include "socket-listener.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#ifndef NEOCLR_HTTP_INVOCATIONS
#define NEOCLR_HTTP_INVOCATIONS 1
#endif
#if NEOCLR_HTTP_INVOCATIONS < 1 || NEOCLR_HTTP_INVOCATIONS > 8
#error HTTP invocation qualification supports one to eight serial entries
#endif
#ifdef _WIN32
#include "../../tools/native/windows-gc-host.h"
#include <fcntl.h>
#include <io.h>
#endif

static int32_t poll_entry(neoclr_aot_context *context, uint64_t *callback,
    const neoclr_probe_frame *boundary, void *state) {
    (void)state;
    int32_t progress = neoclr_scheduler_acquire_v1(context, callback, boundary);
    if (progress == NEOCLR_SCHEDULER_PENDING) neoclr_scheduler_wait_v1(context);
    return progress;
}
/* Serial reuse proof; not persistent guest state or a public hosting ABI. */
#ifndef NEOCLR_HTTP_HEAP_BYTES
#define NEOCLR_HTTP_HEAP_BYTES 1048576
#endif
#if NEOCLR_HTTP_HEAP_BYTES < 8 || NEOCLR_HTTP_HEAP_BYTES % 8 != 0
#error "HTTP heap capacity must be a positive multiple of eight"
#endif

static int invoke(neoclr_aot_context *context, int *failed, int argc, neoclr_process_char **argv) {
    neoclr_socket_scope sockets;
    neoclr_task_scope tasks;
    int host_error = 0, status = 0, result = 0;
    if (neoclr_socket_scope_enter_v1(&sockets, context)) { *failed = 1; return 2; }
    if (neoclr_task_scope_enter_v1(&tasks, context)) {
        neoclr_socket_scope_leave_v1(&sockets); *failed = 1; return 2;
    }
    neoclr_terminal_scope_v1 terminal = {0};
    if (neoclr_terminal_scope_enter_v1(&terminal, context)) {
        neoclr_socket_scope_leave_v1(&sockets);
        neoclr_task_scope_leave_v1(&tasks);
        *failed = 1;
        return 2;
    }
    tasks.poll = poll_entry;
    tasks.complete = neoclr_scheduler_complete_v1;
    status = neoclr_process_entry(argc, argv, &result, context);
    while (!status && !host_error) {
#ifndef NEOCLR_NO_DEFAULT_QUEUE
        status = neoclr_drain_default_queue_v1(context);
        if (status) break;
#endif
        uint64_t callback = 0;
        int32_t progress = neoclr_scheduler_acquire_v1(context, &callback, NULL);
        if (progress == NEOCLR_SCHEDULER_IDLE) break;
        if (progress == NEOCLR_SCHEDULER_READY) {
            status = neoclr_invoke_void_callback_v1(callback, context);
            if (neoclr_scheduler_complete_v1(context, callback, NULL)) host_error = 1;
        }
        else if (progress < 0) host_error = 1;
        else neoclr_scheduler_wait_v1(context);
    }
#ifdef NEOCLR_RUNTIME_ASYNC
    if (neoclr_runtime_async_shutdown_v1(context)) host_error = 1;
#endif
    if (neoclr_terminal_scope_leave_v1(&terminal)) host_error = 1;
    if (neoclr_root_probe_head_v1()) host_error = 1;
    if (neoclr_socket_scope_leave_v1(&sockets)) host_error = 1;
    if (neoclr_task_scope_leave_v1(&tasks)) host_error = 1;
#ifdef NEOCLR_MANAGED_TLS
    if (!neoclr_root_probe_head_v1()) neoclr_tls_managed_dispose_v1(context);
#endif
    if (status && neoclr_aot_render_fault(stderr, &context->fault)) host_error = 1;
    /* Render before clearing fault roots. A completed invocation retains no
     * guest state: the same host heap/context can then accept another entry. */
    if (neoclr_gc_entry_check_v1(context)) host_error = 1;
    if (!host_error) {
        memset(&context->fault, 0, sizeof(context->fault));
        if (neoclr_gc_collect_v1(context, NULL) || context->text.used) host_error = 1;
    }
    if (fflush(stdout) || ferror(stdout) || fflush(stderr) || ferror(stderr)) host_error = 1;
    *failed = host_error;
    return status ? 1 : result;
}

static int run_entry(int argc, neoclr_process_char **argv) {
#ifdef _WIN32
    if (_setmode(_fileno(stdin), _O_BINARY) == -1 ||
        _setmode(_fileno(stdout), _O_BINARY) == -1 ||
        _setmode(_fileno(stderr), _O_BINARY) == -1) return 2;
    neoclr_windows_gc_host host = {0};
    int32_t opened = neoclr_windows_gc_host_open(&host, NEOCLR_HTTP_HEAP_BYTES);
    if (opened) {
        fprintf(stderr, "Native HTTP host initialization failed (status %d, heap %zu bytes)\n",
                opened, (size_t)NEOCLR_HTTP_HEAP_BYTES);
        return 2;
    }
    neoclr_aot_context *context = &host.context;
#else
    const size_t words = NEOCLR_HTTP_HEAP_BYTES / 8;
    uint64_t *buffer = calloc(words + 1, sizeof(uint64_t));
    if (!buffer) return 2;
    buffer[words] = UINT64_C(0x1122334455667788);
    neoclr_aot_context storage = {.text = {(unsigned char *)buffer, words * 8, 0}};
    neoclr_aot_context *context = &storage;
#endif
    int host_error = 0, result = 0;
#ifdef NEOCLR_GC_OWNED_HEAP
    if (neoclr_gc_heap_acquire_v1(&context->text)) host_error = 1;
#endif
    for (unsigned invocation = 0; !host_error && invocation < NEOCLR_HTTP_INVOCATIONS; invocation++) {
        int code = invoke(context, &host_error, argc, argv);
        if (!result && code) result = code;
#ifndef _WIN32
        if (buffer[words] != UINT64_C(0x1122334455667788)) host_error = 1;
#endif
        if (host_error) break;
    }
#ifdef NEOCLR_GC_OWNED_HEAP
    if (neoclr_gc_heap_release_v1(&context->text)) host_error = 1;
#endif
#ifdef _WIN32
    if (neoclr_windows_gc_host_close(&host)) host_error = 1;
#else
    free(buffer);
#endif
    if (host_error) fputs("Native HTTP host failed\n", stderr);
    return host_error ? 2 : result;
}

#ifdef NEOCLR_HTTP_WORKERS
#include "../../runtime/native/workers/http-workers.h"
typedef struct { int argc; neoclr_process_char **argv; } worker_arguments;
static int worker_entry(void *state) {
    worker_arguments *arguments = state;
    return run_entry(arguments->argc, arguments->argv);
}
#endif
int NEOCLR_PROCESS_MAIN(int argc, neoclr_process_char **argv) {
#ifdef NEOCLR_HTTP_WORKERS
    worker_arguments arguments = {argc, argv};
    return neoclr_http_workers_run(NEOCLR_HTTP_WORKERS, worker_entry, &arguments);
#else
    return run_entry(argc, argv);
#endif
}
