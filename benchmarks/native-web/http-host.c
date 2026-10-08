/* One-request HTTP correctness host, not a production scheduler or load benchmark. */
#include "native-gc.h"
#include "task-queue.h"
#include "socket-listener.h"
#include <stdlib.h>
#include <stdio.h>
#include <time.h>

static uint64_t milliseconds(void) {
    struct timespec time;
    if (clock_gettime(CLOCK_MONOTONIC, &time)) return UINT64_MAX;
    return (uint64_t)time.tv_sec * 1000 + (uint64_t)time.tv_nsec / 1000000;
}
int main(void) {
    const size_t words = 131072; /* Explicit 1 MiB managed heap for this consumer. */
    uint64_t *buffer = calloc(words + 1, sizeof(uint64_t));
    if (!buffer) return 2;
    buffer[words] = UINT64_C(0x1122334455667788);
    neoclr_aot_context context = {.text = {(unsigned char *)buffer, words * 8, 0}};
    neoclr_socket_scope sockets;
    neoclr_task_scope tasks;
    if (neoclr_socket_scope_enter_v1(&sockets, &context)) { free(buffer); return 2; }
    if (neoclr_task_scope_enter_v1(&tasks, &context)) {
        neoclr_socket_scope_leave_v1(&sockets); free(buffer); return 2;
    }
    int32_t result = -99;
    int32_t status = neoclr_entry_v4(0, &result, &context);
    int host_error = 0;
    uint64_t started = milliseconds();
    while (!status && !host_error) {
        status = neoclr_drain_default_queue_v1(&context);
        if (status) break;
        unsigned active = 0;
        for (unsigned i = 0; i < 64; i++) active += sockets.operations[i].state != 0;
        if (!active) break;
        uint64_t callback = 0;
        int32_t ready = neoclr_socket_poll_v1(&context, &callback);
        if (ready == 1) status = neoclr_invoke_void_callback_v1(callback, &context);
        else if (ready < 0) host_error = 1;
        else { const struct timespec pause = {0, 1000000}; nanosleep(&pause, NULL); }
        uint64_t now = milliseconds();
        if (now == UINT64_MAX || started == UINT64_MAX || now - started > 15000) host_error = 1;
    }
    if (neoclr_root_probe_head_v1() || buffer[words] != UINT64_C(0x1122334455667788)) host_error = 1;
    if (neoclr_socket_scope_leave_v1(&sockets)) host_error = 1;
    if (neoclr_task_scope_leave_v1(&tasks)) host_error = 1;
    if (status && neoclr_aot_render_fault(stderr, &context.fault)) host_error = 1;
    if (neoclr_gc_collect_v1(&context, NULL) || (!status && context.text.used)) host_error = 1;
    if (host_error) fputs("Native HTTP host failed or exceeded its completion timeout\n", stderr);
    free(buffer);
    return host_error ? 2 : status ? 1 : result;
}
