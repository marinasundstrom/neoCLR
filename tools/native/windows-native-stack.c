/* Private Windows x64 host probe. Generated Windows managed code is not yet
 * admitted. The helper requires an initially usable native host stack. */
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <intrin.h>
#include "../../docs/experiments/aot-console/native-stack.h"
#if !defined(_M_X64)
#error "Windows stack probe requires x64 MSVC"
#endif

__declspec(noinline) int32_t neoclr_native_stack_check_v1(void) {
    /* This slot is just below the caller's stack position after we return.
     * We inspect its address, never its contents or adjacent arguments. */
    uintptr_t position = (uintptr_t)_AddressOfReturnAddress();
    ULONG_PTR low, high;
    MEMORY_BASIC_INFORMATION region;
    SYSTEM_INFO system;
    if (IsThreadAFiber()) return 9;
    GetCurrentThreadStackLimits(&low, &high);
    if (!low || low >= high || position < low || position >= high) return 9;
    if (VirtualQuery((const void *)position, &region, sizeof(region)) != sizeof(region) ||
        region.State != MEM_COMMIT || region.Type != MEM_PRIVATE ||
        region.Protect != PAGE_READWRITE) return 9;
    /* Stack pages commit on demand. AllocationBase identifies the reservation;
     * using the current committed boundary would reject normal fresh stacks. */
    uintptr_t base = (uintptr_t)region.AllocationBase;
    if (!base || base > low || position < base) return 9;
    GetSystemInfo(&system);
    if (!system.dwPageSize || system.dwPageSize > NEOCLR_NATIVE_STACK_RESERVE) return 9;
    /* Leave the existing 256 KiB budget plus a bottom guard-page margin. This
     * is an address-space budget, not a guarantee of successful future commit. */
    return position - base >= NEOCLR_NATIVE_STACK_RESERVE + (size_t)system.dwPageSize ? 0 : 9;
}
