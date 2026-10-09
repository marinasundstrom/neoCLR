#include "windows-gc-host.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>
#include <stdio.h>
#ifdef NDEBUG
#error "Acceptance checks must remain enabled"
#endif

static uint64_t *allocate(neoclr_aot_context *context) {
    void *value = NULL;
    assert(neoclr_gc_allocate_v1(&context->text, 24, NEOCLR_GC_OBJECT, &value) == 0);
    return value;
}
typedef struct {
    neoclr_windows_gc_host *host;
    uint64_t handle, other_handle;
} thread_input;
static DWORD WINAPI foreign_thread(void *argument) {
    thread_input *input = argument;
    void *value = (void *)(uintptr_t)123;
    assert(neoclr_windows_gc_host_close(input->host) == 3);
    assert(neoclr_gc_host_root_read_v1(&input->host->context, input->handle, &value) == 3);
    assert(value == (void *)(uintptr_t)123);
    neoclr_windows_gc_host own = {0};
    assert(neoclr_windows_gc_host_open(&own, 4096) == 0);
    assert(neoclr_gc_host_root_create_v1(&own.context, NULL, &input->other_handle) == 0);
    assert(input->other_handle != input->handle);
    assert(neoclr_windows_gc_host_close(&own) == 3);
    assert(neoclr_gc_host_root_release_v1(&own.context, input->other_handle) == 0);
    assert(neoclr_windows_gc_host_close(&own) == 0);
    return 0;
}
int main(void) {
    neoclr_windows_gc_host host = {0};
    assert(neoclr_windows_gc_host_open(NULL, 4096) == 3);
    assert(neoclr_windows_gc_host_close(NULL) == 3);
    assert(neoclr_windows_gc_host_open(&host, 0) == 3);
    assert(neoclr_windows_gc_host_open(&host, 4096) == 0);
    assert(neoclr_windows_gc_host_open(&host, 4096) == 3);
    neoclr_aot_context *context = &host.context;
    uint64_t *parent = allocate(context), *child = allocate(context);
    (void)allocate(context); /* Unrooted garbage must be reclaimed. */
    parent[1] = (uintptr_t)child;
    child[1] = (uintptr_t)parent; /* Cyclic graph. */
    child[2] = 42;
    uint64_t handle = 0;
    assert(neoclr_gc_host_root_create_v1(context, parent, &handle) == 0);
    assert(neoclr_gc_collect_v1(context, NULL) == 0);
    assert(child[2] == 42 && parent[1] == (uintptr_t)child);
    assert(neoclr_gc_statistics_v1().reclaimed_allocations == 1);
    assert(neoclr_windows_gc_host_close(&host) == 3);
    assert(context->text.data == host.heap.data && child[2] == 42);
    thread_input input = {&host, handle, 0};
    HANDLE thread = CreateThread(NULL, 0, foreign_thread, &input, 0, NULL);
    assert(thread != NULL);
    assert(WaitForSingleObject(thread, 30000) == WAIT_OBJECT_0);
    DWORD exit_code = 1;
    assert(GetExitCodeThread(thread, &exit_code) && exit_code == 0);
    assert(CloseHandle(thread));
    assert(input.other_handle != handle);
    void *value = NULL;
    assert(neoclr_gc_host_root_read_v1(context, input.other_handle, &value) == 3 && !value);
    assert(neoclr_gc_host_root_read_v1(context, handle, &value) == 0 && value == parent);
    /* Published frame roots take over ownership before the host root is released. */
    uint64_t root = (uintptr_t)parent;
    neoclr_probe_storage storage = {&root, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, context, 0, &storage, 1, "", 0);
    assert(neoclr_gc_host_root_release_v1(context, handle) == 0);
    assert(neoclr_gc_host_root_release_v1(context, handle) == 3);
    assert(neoclr_gc_collect_v1(context, neoclr_root_probe_head_v1()) == 0);
    assert(child[2] == 42 && parent[1] == (uintptr_t)child);
    assert(neoclr_windows_gc_host_close(&host) == 3);
    neoclr_probe_leave_v1(&frame);
    assert(neoclr_gc_collect_v1(context, NULL) == 0 && context->text.used == 0);
    assert(neoclr_gc_statistics_v1().reclaimed_allocations == 3);
    /* Exhaustion preserves the result slot; releasing its root allows reuse. */
    void *large = NULL;
    assert(neoclr_gc_allocate_v1(&context->text, 4064, NEOCLR_GC_BYTES, &large) == 0);
    assert(neoclr_gc_host_root_create_v1(context, large, &handle) == 0);
    void *unchanged = (void *)(uintptr_t)123;
    assert(neoclr_gc_allocate_v1(&context->text, 8, NEOCLR_GC_BYTES, &unchanged) == 5);
    assert(unchanged == (void *)(uintptr_t)123);
    assert(neoclr_gc_collect_v1(context, NULL) == 0 && context->text.used == 4096);
    assert(neoclr_gc_host_root_release_v1(context, handle) == 0);
    assert(neoclr_gc_collect_v1(context, NULL) == 0 && context->text.used == 0);
    assert(neoclr_windows_gc_host_close(&host) == 0);
    assert(!host.heap.reservation && !host.context.text.data && !host.thread_id);
    assert(neoclr_windows_gc_host_close(&host) == 0);
    puts("Windows collector: rooted graph, frame handoff, thread isolation, exhaustion and cleanup passed");
    return 0;
}
