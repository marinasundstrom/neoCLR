#include "windows-host-memory.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#if !defined(_WIN32) || !defined(_M_X64)
#error "Windows x64 host experiment only"
#endif

int neoclr_windows_heap_create(neoclr_windows_heap *heap, size_t capacity) {
    if (!heap || heap->reservation || heap->data || heap->capacity ||
        heap->committed || heap->page_size || !capacity ||
        capacity > NEOCLR_WINDOWS_HEAP_LIMIT) return 1;
    SYSTEM_INFO info;
    GetSystemInfo(&info);
    size_t page = info.dwPageSize;
    if (!page || page > NEOCLR_WINDOWS_HEAP_LIMIT) return 1;
    /* Inputs are bounded to 1 MiB, so rounding and two guards cannot overflow. */
    size_t committed = ((capacity + page - 1) / page) * page;
    unsigned char *base = VirtualAlloc(NULL, committed + 2 * page,
                                      MEM_RESERVE, PAGE_NOACCESS);
    if (!base) return 1;
    unsigned char *data = VirtualAlloc(base + page, committed, MEM_COMMIT, PAGE_READWRITE);
    if (!data) {
        VirtualFree(base, 0, MEM_RELEASE);
        return 1;
    }
    *heap = (neoclr_windows_heap){base, data, capacity, committed, page};
    return 0;
}

int neoclr_windows_heap_release(neoclr_windows_heap *heap) {
    if (!heap) return 1;
    if (!heap->reservation) {
        return heap->data || heap->capacity || heap->committed || heap->page_size ? 1 : 0;
    }
    if (!VirtualFree(heap->reservation, 0, MEM_RELEASE)) return 1;
    *heap = (neoclr_windows_heap){0};
    return 0;
}
