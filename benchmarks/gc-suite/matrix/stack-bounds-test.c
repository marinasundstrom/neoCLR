/* Boundaries and owner isolation for the macOS ARM64 stack probe. */
#include "native-stack.h"
#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>

static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t changed = PTHREAD_COND_INITIALIZER;
static unsigned arrived;
static void meet(void) {
    pthread_mutex_lock(&lock);
    if (++arrived == 2) pthread_cond_broadcast(&changed);
    while (arrived < 2) pthread_cond_wait(&changed, &lock);
    pthread_mutex_unlock(&lock);
}
static void valid_bounds(void) {
    pthread_t thread = pthread_self();
    uintptr_t upper = (uintptr_t)pthread_get_stackaddr_np(thread);
    size_t size = pthread_get_stacksize_np(thread);
    neoclr_native_stack_bounds_v1(upper-size, upper);
}
static void *other_owner(void *unused) {
    (void)unused;
    assert(neoclr_native_stack_check_v1() == 0);
    neoclr_native_stack_bounds_v1(1, 2);
    meet();
    for (unsigned i=0; i<10000; ++i)
        assert(neoclr_native_stack_check_v1() == 9);
    neoclr_native_stack_bounds_v1(0, 0);
    assert(neoclr_native_stack_check_v1() == 0);
    return NULL;
}
int main(void) {
    assert(neoclr_native_stack_check_v1() == 0);
    valid_bounds();
    assert(neoclr_native_stack_check_v1() == 0);
    neoclr_native_stack_bounds_v1(UINTPTR_MAX, UINTPTR_MAX);
    assert(neoclr_native_stack_check_v1() == 9);
    neoclr_native_stack_bounds_v1(1, 0);
    assert(neoclr_native_stack_check_v1() == 9);
    neoclr_native_stack_bounds_v1(0, 1);
    assert(neoclr_native_stack_check_v1() == 9);
    uintptr_t sp;
    __asm__ volatile("mov %0, sp" : "=r" (sp));
    neoclr_native_stack_bounds_v1(sp-1024, sp+1024);
    assert(neoclr_native_stack_check_v1() == 9); /* reserve exhausted */
    for (unsigned i=0; i<100; ++i) {
        valid_bounds();
        assert(neoclr_native_stack_check_v1() == 0);
        neoclr_native_stack_bounds_v1(1, 2);
        assert(neoclr_native_stack_check_v1() == 9);
        neoclr_native_stack_bounds_v1(0, 0);
        assert(neoclr_native_stack_check_v1() == 0);
    }
    valid_bounds();
    pthread_t thread;
    assert(!pthread_create(&thread, NULL, other_owner, NULL));
    meet();
    for (unsigned i=0; i<10000; ++i)
        assert(neoclr_native_stack_check_v1() == 0);
    assert(!pthread_join(thread, NULL));
    puts("stack bounds, replacement, reset and thread isolation passed");
}
