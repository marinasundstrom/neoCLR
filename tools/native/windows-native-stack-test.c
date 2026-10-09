#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>
#include <stdio.h>
#include "windows-gc-host.h"
#include "../../docs/experiments/aot-console/native-stack.h"
#ifdef NDEBUG
#error "Acceptance checks must remain enabled"
#endif

/* Passing the full buffer to a non-inlined function, page-stride writes and
 * reads after recursion prevent frame elision and tail-call conversion at /O2. */
__declspec(noinline) static void touch(volatile unsigned char *bytes, unsigned char value) {
    for (size_t i = 0; i < 16384; i += 4096) bytes[i] = value;
    bytes[16383] = value;
}
__declspec(noinline) static int32_t descend(neoclr_aot_context *context, uint64_t *object,
                                           unsigned depth, unsigned *stopped) {
    volatile unsigned char buffer[16384];
    touch(buffer, (unsigned char)depth);
    assert(depth < 128);
    uintptr_t root = (uintptr_t)object;
    neoclr_probe_storage storage = {&root, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, context, depth, &storage, 1, "", 0);
    int32_t status = neoclr_native_stack_check_v1();
    if (!status) {
        status = descend(context, object, depth + 1, stopped);
    } else {
        assert(status == 9);
        *stopped = depth;
        /* Exercise collector headroom and rooted retention at the limit. */
        assert(neoclr_gc_collect_v1(context, neoclr_root_probe_head_v1()) == 0);
        assert(object[2] == 42);
    }
    for (size_t i = 0; i < sizeof(buffer); i += 4096) assert(buffer[i] == (unsigned char)depth);
    assert(buffer[sizeof(buffer) - 1] == (unsigned char)depth);
    neoclr_probe_leave_v1(&frame);
    return status;
}
static DWORD WINAPI worker(void *argument) {
    unsigned *stopped = argument;
    assert(neoclr_native_stack_check_v1() == 0);
    neoclr_windows_gc_host host = {0};
    assert(neoclr_windows_gc_host_open(&host, 4096) == 0);
    void *allocation = NULL;
    assert(neoclr_gc_allocate_v1(&host.context.text, 24, NEOCLR_GC_OBJECT, &allocation) == 0);
    uint64_t *object = allocation;
    object[2] = 42;
    assert(descend(&host.context, object, 0, stopped) == 9);
    assert(*stopped > 1 && *stopped < 64);
    assert(!neoclr_root_probe_head_v1() && neoclr_root_probe_depth_v1() == 0);
    assert(neoclr_native_stack_check_v1() == 0);
    assert(neoclr_gc_collect_v1(&host.context, NULL) == 0 && !host.context.text.used);
    assert(neoclr_windows_gc_host_close(&host) == 0);
    return 0;
}
__declspec(noinline) static int32_t guarded_entry(int32_t *output) {
    int32_t status = neoclr_native_stack_check_v1();
    if (status) return status;
    *output = 42;
    return 0;
}
static DWORD WINAPI small_worker(void *argument) {
    (void)argument;
    int32_t output = 123;
    assert(guarded_entry(&output) == 9 && output == 123);
    assert(!neoclr_root_probe_head_v1());
    return 0;
}
static void run_thread(size_t reserve, LPTHREAD_START_ROUTINE entry, void *argument) {
    HANDLE thread = CreateThread(NULL, reserve, entry, argument, STACK_SIZE_PARAM_IS_A_RESERVATION, NULL);
    assert(thread);
    assert(WaitForSingleObject(thread, 30000) == WAIT_OBJECT_0);
    DWORD status = 1;
    assert(GetExitCodeThread(thread, &status) && status == 0);
    assert(CloseHandle(thread));
}
int main(void) {
    assert(neoclr_native_stack_check_v1() == 0);
    run_thread(128 * 1024, small_worker, NULL);
    unsigned shallow = 0, deep = 0;
    run_thread(512 * 1024, worker, &shallow);
    run_thread(1024 * 1024, worker, &deep);
    assert(deep > shallow);
    /* Explicitly reject even a converted thread: fiber/activation ownership is
     * not qualified by this system-thread stack contract. */
    assert(ConvertThreadToFiber(NULL) != NULL);
    assert(neoclr_native_stack_check_v1() == 9);
    assert(ConvertFiberToThread());
    assert(neoclr_native_stack_check_v1() == 0);
    printf("{\"passed\":true,\"smallStackRejected\":true,\"fiberRejected\":true,\"depth512KiB\":%u,\"depth1MiB\":%u}\n", shallow, deep);
    return 0;
}
