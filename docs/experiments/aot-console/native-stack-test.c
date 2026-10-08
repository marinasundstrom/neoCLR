#include "native-stack.h"
#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>

/* Volatile storage and work after the recursive call prevent tail-call/frame
 * elimination at -O2. Stop well before the OS guard page, then validate unwind. */
__attribute__((noinline)) static unsigned descend(unsigned depth) {
    volatile unsigned char storage[16384];
    storage[0] = (unsigned char)depth;
    storage[sizeof(storage) - 1] = (unsigned char)(depth + 1);
    assert(depth < 1024);
    unsigned result = neoclr_native_stack_check_v1() == 0 ? descend(depth + 1) : depth;
    assert(storage[0] == (unsigned char)depth);
    assert(storage[sizeof(storage) - 1] == (unsigned char)(depth + 1));
    return result;
}
static void *worker(void *data) {
    unsigned *depth = data;
    assert(neoclr_native_stack_check_v1() == 0);
    *depth = descend(0);
    assert(*depth > 1 && *depth < 32);
    assert(neoclr_native_stack_check_v1() == 0);
    return NULL;
}
static void *small_worker(void *data) {
    (void)data;
    assert(neoclr_native_stack_check_v1() == 9);
    return NULL;
}
int main(void) {
    assert(neoclr_native_stack_check_v1() == 0);
    unsigned depth = 0;
    pthread_attr_t attr;
    pthread_t thread;
    assert(pthread_attr_init(&attr) == 0);
    assert(pthread_attr_setstacksize(&attr, 512 * 1024) == 0);
    assert(pthread_create(&thread, &attr, worker, &depth) == 0);
    assert(pthread_join(thread, NULL) == 0);
    assert(depth > 1);
    assert(pthread_attr_setstacksize(&attr, 128 * 1024) == 0);
    assert(pthread_create(&thread, &attr, small_worker, NULL) == 0);
    assert(pthread_join(thread, NULL) == 0);
    assert(pthread_attr_destroy(&attr) == 0);
    assert(neoclr_native_stack_check_v1() == 0);
    puts("native stack boundary and unwind passed");
}
