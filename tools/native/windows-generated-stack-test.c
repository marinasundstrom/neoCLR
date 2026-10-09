#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "windows-gc-host.h"
#ifdef NDEBUG
#error "Acceptance checks must remain enabled"
#endif

static void fault_return(neoclr_aot_context *context, int32_t depth, int small) {
    int32_t output = -99;
    assert(neoclr_entry_v4(depth, &output, context) == 9 && output == -99);
    assert(context->fault.code == 9);
    const neoclr_aot_text *message = context->fault.message;
    const char *expected = "Call stack limit exceeded";
    assert(message && message->length == strlen(expected));
    assert(memcmp(message->bytes, expected, (size_t)message->length) == 0);
    assert(small ? context->fault.frame_count == 0 : context->fault.frame_count > 1);
    assert(context->fault.frame_count <= 64);
    assert(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
    assert(neoclr_gc_entry_check_v1(context) == 0);
}
static DWORD WINAPI worker(void *argument) {
    uint64_t *frames = argument;
    neoclr_windows_gc_host host = {0};
    assert(neoclr_windows_gc_host_open(&host, 4096) == 0);
    for (int pass = 0; pass < 2; ++pass) {
        int32_t output = -99;
        assert(neoclr_entry_v4(5, &output, &host.context) == 0 && output == 42);
        assert(host.context.fault.code == 0 && !neoclr_root_probe_head_v1());
        uint64_t before = neoclr_root_probe_count_v1();
        fault_return(&host.context, -1, 0);
        *frames = neoclr_root_probe_count_v1() - before;
        assert(*frames > 5 && *frames < 512);
        assert(neoclr_gc_collect_v1(&host.context, NULL) == 0 && !host.context.text.used);
    }
    int32_t output = -99;
    assert(neoclr_entry_v4(0, &output, &host.context) == 0 && output == 42);
    assert(!host.context.fault.code && !host.context.fault.frame_count);
    assert(neoclr_windows_gc_host_close(&host) == 0);
    return 0;
}
static DWORD WINAPI small_worker(void *argument) {
    (void)argument;
    /* Bypass host admission deliberately to exercise the generated entry guard.
     * No guest heap allocation is possible in this integer-only fixture. */
    neoclr_aot_context context = {0};
    fault_return(&context, 5, 1);
    return 0;
}
static void run_thread(size_t reserve, LPTHREAD_START_ROUTINE entry, void *argument) {
    HANDLE thread = CreateThread(NULL, reserve, entry, argument, STACK_SIZE_PARAM_IS_A_RESERVATION, NULL);
    assert(thread && WaitForSingleObject(thread, 30000) == WAIT_OBJECT_0);
    DWORD status = 1;
    assert(GetExitCodeThread(thread, &status) && status == 0);
    assert(CloseHandle(thread));
}
int main(void) {
    run_thread(128 * 1024, small_worker, NULL);
    uint64_t shallow = 0, deep = 0;
    run_thread(512 * 1024, worker, &shallow);
    run_thread(1024 * 1024, worker, &deep);
    assert(deep > shallow);
    printf("{\"passed\":true,\"smallStackRejected\":true,\"snapshots512KiB\":%llu,\"snapshots1MiB\":%llu,\"reusePassed\":true}\n",
           (unsigned long long)shallow, (unsigned long long)deep);
    return 0;
}
