#include "native-stack.h"
#include <pthread.h>
#include <stddef.h>
#if !defined(__APPLE__) || !defined(__aarch64__)
#error "native stack probe requires macOS ARM64"
#endif

/* Do not cache OS-thread bounds: a future activation/stack owner must provide
 * its own bounds. Reject pointers outside this pthread's reported stack. */
__attribute__((noinline)) int32_t neoclr_native_stack_check_v1(void) {
    uintptr_t stack_pointer;
    __asm__ volatile("mov %0, sp" : "=r" (stack_pointer));
    pthread_t thread = pthread_self();
    uintptr_t upper = (uintptr_t)pthread_get_stackaddr_np(thread);
    size_t size = pthread_get_stacksize_np(thread);
    if (!upper || !size || size > upper) return 9;
    uintptr_t lower = upper - size;
    if (stack_pointer < lower || stack_pointer >= upper) return 9;
    return stack_pointer - lower >= NEOCLR_NATIVE_STACK_RESERVE ? 0 : 9;
}
