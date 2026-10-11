#include "native-stack.h"
#include <pthread.h>
#include <stddef.h>
#if !defined(__APPLE__) || !defined(__aarch64__)
#error "native stack probe requires macOS ARM64"
#endif

static _Thread_local uintptr_t retained_lower, retained_upper;
void neoclr_native_stack_bounds_v1(uintptr_t lower, uintptr_t upper) {
    retained_lower = lower;
    retained_upper = upper;
}
/* Do not cache OS-thread bounds: a future activation/stack owner must provide
 * its own bounds. Reject pointers outside this pthread's reported stack. */
__attribute__((noinline)) int32_t neoclr_native_stack_check_v1(void) {
    uintptr_t stack_pointer;
    __asm__ volatile("mov %0, sp" : "=r" (stack_pointer));
    if (retained_lower || retained_upper) {
        if (stack_pointer < retained_lower || stack_pointer >= retained_upper) return 9;
        return stack_pointer - retained_lower >= NEOCLR_NATIVE_STACK_RESERVE ? 0 : 9;
    }
    pthread_t thread = pthread_self();
    uintptr_t upper = (uintptr_t)pthread_get_stackaddr_np(thread);
    size_t size = pthread_get_stacksize_np(thread);
    if (!upper || !size || size > upper) return 9;
    uintptr_t lower = upper - size;
    if (stack_pointer < lower || stack_pointer >= upper) return 9;
    return stack_pointer - lower >= NEOCLR_NATIVE_STACK_RESERVE ? 0 : 9;
}
