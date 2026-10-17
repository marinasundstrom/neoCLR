
#include "text-arena.h"
#include "native-gc.h"
#include "task-queue.h"
#include "socket-listener.h"
#include "../../../runtime/native/async/native-runtime.h"
#include <string.h>
#include <stdio.h>
extern int neoclr_aot_render_fault(FILE *, const neoclr_aot_fault *);
typedef int32_t (*entry_v4)(int32_t, int32_t *, neoclr_aot_context *);
static int32_t poll_entry(neoclr_aot_context *context, uint64_t *callback,
    const neoclr_probe_frame *boundary, void *state) {
    (void)state;
    int32_t progress = neoclr_scheduler_acquire_v1(context, callback, boundary);
    if (progress == NEOCLR_SCHEDULER_PENDING) neoclr_scheduler_wait_v1(context);
    return progress;
}
static int32_t check(entry_v4 entry) {
    uint64_t heap[8192] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_socket_scope sockets;
    if (neoclr_socket_scope_enter_v1(&sockets, &context)) return 2;
    neoclr_task_scope tasks;
    if (neoclr_task_scope_enter_v1(&tasks, &context)) return 2;
    tasks.poll = poll_entry;
    tasks.complete = neoclr_scheduler_complete_v1;
    int32_t result = -1;
    int32_t status = entry(0, &result, &context);
    if (status || context.fault.code) { neoclr_aot_render_fault(stderr, &context.fault); fprintf(stderr, "status=%d fault=%d result=%d\n", status, context.fault.code, result); return 2; }

    if (neoclr_runtime_async_shutdown_v1(&context)) return 2;
    if (neoclr_socket_scope_leave_v1(&sockets)) return 2;
    if (neoclr_task_scope_leave_v1(&tasks)) return 2;
    if (neoclr_root_probe_head_v1() || neoclr_gc_collect_v1(&context, NULL) || context.text.used) { fprintf(stderr, "roots/collection failed; used=%llu result=%d\n", (unsigned long long)context.text.used, result); return 2; }

    return result;
}
#ifdef JIT_HOST
int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const char **argv) {
    if (!entry || with_args || argc < 1 || !argv) return 2;
    entry_v4 callable;
    _Static_assert(sizeof(callable) == sizeof(entry), "entry pointer size");
    memcpy(&callable, &entry, sizeof(callable));
    return check(callable);
}
#else
int main(void) { return check(neoclr_entry_v4); }
#endif
