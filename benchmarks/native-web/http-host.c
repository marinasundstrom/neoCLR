/* One-request HTTP correctness host, not a production scheduler or load benchmark. */
#include "native-gc.h"
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

static uint64_t milliseconds(void) {
    uint64_t now = neoclr_os_monotonic_ns();
    return now ? now / UINT64_C(1000000) : UINT64_MAX;
}
typedef struct entry_io { neoclr_socket_scope *sockets; uint64_t started; } entry_io;
static int32_t poll_entry(neoclr_aot_context *context, uint64_t *callback,
    const neoclr_probe_frame *boundary, void *state) {
    entry_io *io = state;
    uint64_t now = milliseconds();
    if (now == UINT64_MAX || io->started == UINT64_MAX || now - io->started > 15000) return -3;
    unsigned active = 0;
    for (unsigned i = 0; i < 64; i++) active += io->sockets->operations[i].state != 0;
    if (!active) return 0;
    int32_t ready = neoclr_socket_poll_suspended_v1(context, callback, boundary);
    if (ready) return ready;
    neoclr_os_pause();
    return 2;
}
/* Serial reuse proof; not persistent guest state or a public hosting ABI. */
static int invoke(neoclr_aot_context *context, int *failed) {
    neoclr_socket_scope sockets;
    neoclr_task_scope tasks;
    int host_error = 0, status = 0, result = 0;
    if (neoclr_socket_scope_enter_v1(&sockets, context)) { *failed = 1; return 2; }
    if (neoclr_task_scope_enter_v1(&tasks, context)) {
        neoclr_socket_scope_leave_v1(&sockets); *failed = 1; return 2;
    }
    entry_io io = {&sockets, milliseconds()};
    tasks.poll = poll_entry;
    tasks.poll_state = &io;
    status = neoclr_entry_v4(0, &result, context);
    uint64_t started = milliseconds();
    while (!status && !host_error) {
        status = neoclr_drain_default_queue_v1(context);
        if (status) break;
        unsigned active = 0;
        for (unsigned i = 0; i < 64; i++) active += sockets.operations[i].state != 0;
        if (!active) break;
        uint64_t callback = 0;
        int32_t ready = neoclr_socket_poll_v1(context, &callback);
        if (ready == 1) status = neoclr_invoke_void_callback_v1(callback, context);
        else if (ready < 0) host_error = 1;
        else neoclr_os_pause();
        uint64_t now = milliseconds();
        if (now == UINT64_MAX || started == UINT64_MAX || now - started > 15000) host_error = 1;
    }
    if (neoclr_root_probe_head_v1()) host_error = 1;
    if (neoclr_socket_scope_leave_v1(&sockets)) host_error = 1;
    if (neoclr_task_scope_leave_v1(&tasks)) host_error = 1;
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

int main(void) {
#ifdef _WIN32
    if (_setmode(_fileno(stdin), _O_BINARY) == -1 ||
        _setmode(_fileno(stdout), _O_BINARY) == -1 ||
        _setmode(_fileno(stderr), _O_BINARY) == -1) return 2;
    neoclr_windows_gc_host host = {0};
    if (neoclr_windows_gc_host_open(&host, 1024 * 1024)) return 2;
    neoclr_aot_context *context = &host.context;
#else
    const size_t words = 131072;
    uint64_t *buffer = calloc(words + 1, sizeof(uint64_t));
    if (!buffer) return 2;
    buffer[words] = UINT64_C(0x1122334455667788);
    neoclr_aot_context storage = {.text = {(unsigned char *)buffer, words * 8, 0}};
    neoclr_aot_context *context = &storage;
#endif
    int host_error = 0, result = 0;
    for (unsigned invocation = 0; invocation < NEOCLR_HTTP_INVOCATIONS; invocation++) {
        int code = invoke(context, &host_error);
        if (!result && code) result = code;
#ifndef _WIN32
        if (buffer[words] != UINT64_C(0x1122334455667788)) host_error = 1;
#endif
        if (host_error) break;
    }
#ifdef _WIN32
    if (neoclr_windows_gc_host_close(&host)) host_error = 1;
#else
    free(buffer);
#endif
    if (host_error) fputs("Native HTTP host failed or exceeded its completion timeout\n", stderr);
    return host_error ? 2 : result;
}
