#include "native-gc.h"
#include "console.h"
#include "task-queue.h"
#include "socket-listener.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>

enum { NONE, ALLOCATION, SOCKET, TASK, TERMINAL, TERMINAL_LEAVE, RENDER };
static int failure;
static unsigned frees, sockets, tasks, terminals;
static void *test_calloc(size_t count, size_t size) {
    return failure == ALLOCATION ? NULL : calloc(count, size);
}
static void test_free(void *memory) {
    uint64_t bytes[8] = {0};
    neoclr_aot_text_arena probe = {(unsigned char *)bytes, sizeof(bytes), 0};
    /* The host must release ownership before freeing its original buffer. */
    assert(neoclr_gc_heap_acquire_v1(&probe) == 0);
    assert(neoclr_gc_heap_release_v1(&probe) == 0);
    frees++;
    free(memory);
}
static int32_t test_socket_enter(neoclr_socket_scope *scope, neoclr_aot_context *context) {
    if (failure == SOCKET) return 3;
    int32_t status = neoclr_socket_scope_enter_v1(scope, context);
    if (!status) sockets++;
    return status;
}
static int32_t test_socket_leave(neoclr_socket_scope *scope) {
    assert(sockets == 1);
    sockets--;
    return neoclr_socket_scope_leave_v1(scope);
}
static int32_t test_task_enter(neoclr_task_scope *scope, neoclr_aot_context *context) {
    if (failure == TASK) return 3;
    int32_t status = neoclr_task_scope_enter_v1(scope, context);
    if (!status) tasks++;
    return status;
}
static int32_t test_task_leave(neoclr_task_scope *scope) {
    assert(tasks == 1);
    tasks--;
    return neoclr_task_scope_leave_v1(scope);
}
static int32_t test_terminal_enter(neoclr_terminal_scope_v1 *scope, const void *context) {
    if (failure == TERMINAL) return 3;
    int32_t status = neoclr_terminal_scope_enter_v1(scope, context);
    if (!status) terminals++;
    return status;
}
static int32_t test_terminal_leave(neoclr_terminal_scope_v1 *scope) {
    assert(terminals == 1);
    terminals--;
    int32_t status = neoclr_terminal_scope_leave_v1(scope);
    return failure == TERMINAL_LEAVE ? 3 : status;
}
static int test_render(FILE *stream, const neoclr_aot_fault *fault) {
    return failure == RENDER ? -1 : neoclr_aot_render_fault(stream, fault);
}
#define calloc test_calloc
#define free test_free
#define neoclr_socket_scope_enter_v1 test_socket_enter
#define neoclr_socket_scope_leave_v1 test_socket_leave
#define neoclr_task_scope_enter_v1 test_task_enter
#define neoclr_task_scope_leave_v1 test_task_leave
#define neoclr_terminal_scope_enter_v1 test_terminal_enter
#define neoclr_terminal_scope_leave_v1 test_terminal_leave
#define neoclr_aot_render_fault test_render
#include "../../../docs/experiments/jit-managed/host.c"
#undef calloc
#undef free

static int32_t entry(int32_t input, int32_t *result, neoclr_aot_context *context) {
    (void)input;
#ifdef NEOCLR_GC_OWNED_HEAP
    assert(neoclr_gc_heap_acquire_v1(&context->text) == 3);
#else
    assert(neoclr_gc_heap_acquire_v1(&context->text) == 0);
    assert(neoclr_gc_heap_release_v1(&context->text) == 0);
#endif
    *result = 42;
    if (failure == RENDER) {
        context->fault.code = NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        return 3;
    }
    return 0;
}
int main(void) {
    const char *argv[] = {"host-failures"};
    for (int selected = ALLOCATION; selected <= RENDER; selected++) {
        failure = selected;
        frees = 0;
        assert(neoclr_jit_run_v1((const void *)entry, 0, 1, argv) == 2);
        assert(frees == (selected == ALLOCATION ? 0u : 1u));
        assert(!sockets && !tasks && !terminals);
        failure = NONE;
        frees = 0;
        assert(neoclr_jit_run_v1((const void *)entry, 0, 1, argv) == 42);
        assert(frees == 1 && !sockets && !tasks && !terminals);
    }
    puts("jit-host-failure-cleanup: PASS");
    return 0;
}
