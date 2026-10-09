#include "windows-gc-host.h"
#include "../../docs/experiments/aot-console/native-stack.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

int32_t neoclr_windows_gc_host_open(neoclr_windows_gc_host *host, size_t capacity) {
    if (!host || host->thread_id || host->context.text.data) return 3;
    int32_t stack_status = neoclr_native_stack_check_v1();
    if (stack_status) return stack_status;
    if (neoclr_windows_heap_create(&host->heap, capacity)) return 3;
    host->context = (neoclr_aot_context){0};
    host->context.text.data = host->heap.data;
    host->context.text.capacity = host->heap.capacity;
    host->thread_id = GetCurrentThreadId();
    return 0;
}

int32_t neoclr_windows_gc_host_close(neoclr_windows_gc_host *host) {
    if (!host) return 3;
    if (!host->thread_id) return host->heap.reservation || host->context.text.data ? 3 : 0;
    if (host->thread_id != GetCurrentThreadId() || neoclr_gc_entry_check_v1(&host->context)) return 3;
    if (neoclr_windows_heap_release(&host->heap)) return 3;
    *host = (neoclr_windows_gc_host){0};
    return 0;
}
