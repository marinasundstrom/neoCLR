#include "windows-host-memory.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef NDEBUG
#error "Acceptance checks must remain enabled"
#endif

/* SEH is used only by the C acceptance test to observe guard faults. It does not
 * establish guest SEH interoperability or generated-code unwind support. */
static int guard_rejects_write(volatile unsigned char *address) {
    __try {
        *address = 1;
    } __except(GetExceptionCode() == EXCEPTION_ACCESS_VIOLATION ?
               EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH) {
        return 1;
    }
    return 0;
}
static void check_capacity(size_t capacity) {
    neoclr_windows_heap heap = {0};
    assert(neoclr_windows_heap_create(&heap, capacity) == 0);
    assert(heap.capacity == capacity && heap.committed >= capacity);
    assert(heap.committed % heap.page_size == 0);
    assert((uintptr_t)heap.data % 8 == 0);
    for (size_t i = 0; i < heap.committed; ++i) assert(heap.data[i] == 0);
    MEMORY_BASIC_INFORMATION info;
    assert(VirtualQuery(heap.data, &info, sizeof(info)) == sizeof(info));
    assert(info.State == MEM_COMMIT && info.Protect == PAGE_READWRITE);
    assert(VirtualQuery(heap.reservation, &info, sizeof(info)) == sizeof(info));
    assert(info.State == MEM_RESERVE);
    assert(VirtualQuery(heap.data + heap.committed, &info, sizeof(info)) == sizeof(info));
    assert(info.State == MEM_RESERVE);
    assert(guard_rejects_write(heap.data - 1));
    assert(guard_rejects_write(heap.data + heap.committed));
    memset(heap.data, 0x5a, capacity);
    neoclr_windows_heap original = heap; /* Snapshot for comparison, never released. */
    assert(neoclr_windows_heap_create(&heap, capacity) == 1);
    assert(heap.reservation == original.reservation && heap.data == original.data);
    assert(heap.capacity == original.capacity && heap.committed == original.committed);
    assert(heap.data[0] == 0x5a && heap.data[capacity - 1] == 0x5a);
    assert(neoclr_windows_heap_release(&heap) == 0);
    assert(VirtualQuery(original.reservation, &info, sizeof(info)) == sizeof(info));
    assert(info.State == MEM_FREE);
    assert(!heap.reservation && !heap.data && !heap.capacity && !heap.committed && !heap.page_size);
    assert(neoclr_windows_heap_release(&heap) == 0);
}
int main(void) {
    neoclr_windows_heap heap = {0};
    assert(neoclr_windows_heap_create(NULL, 1) == 1);
    assert(neoclr_windows_heap_release(NULL) == 1);
    const size_t invalid[] = {0, NEOCLR_WINDOWS_HEAP_LIMIT + 1, SIZE_MAX};
    for (size_t i = 0; i < sizeof(invalid) / sizeof(invalid[0]); ++i) {
        assert(neoclr_windows_heap_create(&heap, invalid[i]) == 1);
        assert(!heap.reservation && !heap.data && !heap.capacity && !heap.committed && !heap.page_size);
    }
    SYSTEM_INFO info;
    GetSystemInfo(&info);
    for (unsigned repeat = 0; repeat < 3; ++repeat) {
        check_capacity(1);
        check_capacity(info.dwPageSize);
        check_capacity((size_t)info.dwPageSize + 1);
        check_capacity(NEOCLR_WINDOWS_HEAP_LIMIT);
    }
    puts("Windows guarded heap: 12 allocation lifecycles passed");
    return 0;
}
