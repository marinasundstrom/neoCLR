/* Private retained-state HTTP consumer: three sequential dispatches on one
 * listener/object graph. Not a general scheduler or public hosting ABI. */
#include "native-session.h"
#include "task-queue.h"
#include "socket-listener.h"
#include <stdlib.h>
#include <stdio.h>
#ifdef _WIN32
#include "../../tools/native/windows-gc-host.h"
#include <fcntl.h>
#include <io.h>
#endif
static uint64_t milliseconds(void) {
    uint64_t now = neoclr_os_monotonic_ns();
    return now ? now / UINT64_C(1000000) : UINT64_MAX;
}
static int invoke_session(neoclr_aot_context *context, int *failed) {
    neoclr_socket_scope sockets;
    neoclr_task_scope tasks;
    neoclr_native_session session = {0};
    uint64_t callback = 0;
    int host_error = 0, status = 0;
    if (neoclr_socket_scope_enter_v1(&sockets, context)) { *failed = 1; return 2; }
    if (neoclr_task_scope_enter_v1(&tasks, context)) {
        neoclr_socket_scope_leave_v1(&sockets); *failed = 1; return 2;
    }
    status = neoclr_bootstrap_callback_v1(0, &callback, context);
    if (!status && neoclr_session_adopt_v1(&session, context, callback)) host_error = 1;
    for (unsigned exchange = 0; exchange < 3 && !status && !host_error; exchange++) {
        status = neoclr_session_invoke_v1(&session, callback);
        uint64_t started = milliseconds();
        while (!status && !host_error) {
            status = neoclr_drain_default_queue_v1(context);
            if (status) break;
            unsigned active = 0;
            for (unsigned i = 0; i < 64; i++) active += sockets.operations[i].state != 0;
            if (!active) break;
            uint64_t completion = 0;
            int32_t ready = neoclr_socket_poll_v1(context, &completion);
            if (ready == 1) status = neoclr_invoke_void_callback_v1(completion, context);
            else if (ready < 0) host_error = 1;
            else neoclr_os_pause();
            uint64_t now = milliseconds();
            if (now == UINT64_MAX || started == UINT64_MAX || now - started > 15000) host_error = 1;
        }
        /* A service callback fault is terminal for this host too: never dispatch
         * again or clear diagnostics. Tear down services before session close. */
        if (!status && !host_error && neoclr_session_collect_v1(&session)) host_error = 1;
    }
    if (neoclr_root_probe_head_v1()) host_error = 1;
    if (status && neoclr_aot_render_fault(stderr, &context->fault)) host_error = 1;
    if (neoclr_socket_scope_leave_v1(&sockets)) host_error = 1;
    if (neoclr_task_scope_leave_v1(&tasks)) host_error = 1;
    if (session.owner) {
        if (neoclr_session_close_v1(&session)) host_error = 1;
    } else {
        if (callback && neoclr_gc_host_root_release_v1(context, callback)) host_error = 1;
        if (neoclr_gc_entry_check_v1(context)) host_error = 1;
        /* Failed bootstrap/adoption owns no live work after service teardown. */
        if (!host_error) context->text.used = 0;
    }
    if (neoclr_gc_entry_check_v1(context) || context->text.used) host_error = 1;
    if (fflush(stdout) || ferror(stdout) || fflush(stderr) || ferror(stderr)) host_error = 1;
    *failed = host_error;
    return status ? 1 : 0;
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
    result = invoke_session(context, &host_error);
#ifndef _WIN32
    if (buffer[words] != UINT64_C(0x1122334455667788)) host_error = 1;
#endif

#ifdef _WIN32
    if (neoclr_windows_gc_host_close(&host)) host_error = 1;
#else
    free(buffer);
#endif
    if (host_error) fputs("Native HTTP host failed or exceeded its completion timeout\n", stderr);
    return host_error ? 2 : result;
}
