#include "native-gc.h"
#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

extern int32_t neoclr_jit_run_v1(const void *, int32_t, int32_t, const char **);
static const char *arguments[] = {"ownership-lifecycle"};
static int32_t success(int32_t input, int32_t *result, neoclr_aot_context *context) {
    (void)input;
    void *payload = NULL;
    assert(neoclr_gc_heap_acquire_v1(&context->text) == 3); /* Already owned. */
    assert(neoclr_gc_allocate_v1(&context->text, 24, NEOCLR_GC_BYTES, &payload) == 0);
    assert(payload != NULL);
    for (unsigned i = 0; i < 24; i++) assert(((unsigned char *)payload)[i] == 0);
    memset(payload, 42, 24);
    *result = 42;
    return 0;
}
static int32_t fault(int32_t input, int32_t *result, neoclr_aot_context *context) {
    (void)success(input, result, context);
    context->fault.code = NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return 3;
}
static int32_t nested(int32_t input, int32_t *result, neoclr_aot_context *context) {
    assert(neoclr_jit_run_v1((const void *)success, 0, 1, arguments) == 2);
    return success(input, result, context);
}
static void *thread_main(void *unused) {
    (void)unused;
    for (int i = 0; i < 3; i++)
        assert(neoclr_jit_run_v1((const void *)success, 0, 1, arguments) == 42);
    return NULL;
}
int main(void) {
    assert(neoclr_jit_run_v1(NULL, 0, 1, arguments) == 2);
    assert(neoclr_jit_run_v1((const void *)success, 2, 1, arguments) == 2);
    uint64_t storage[64] = {0};
    neoclr_aot_text_arena incumbent = {(unsigned char *)storage, sizeof(storage), 0};
    assert(neoclr_gc_heap_acquire_v1(&incumbent) == 0);
    assert(neoclr_jit_run_v1((const void *)success, 0, 1, arguments) == 2);
    assert(neoclr_gc_heap_release_v1(&incumbent) == 0);
    assert(neoclr_jit_run_v1((const void *)success, 0, 1, arguments) == 42);
    assert(neoclr_jit_run_v1((const void *)fault, 0, 1, arguments) == 1);
    assert(neoclr_jit_run_v1((const void *)nested, 0, 1, arguments) == 42);
    pthread_t threads[2];
    for (int i = 0; i < 2; i++) assert(pthread_create(&threads[i], NULL, thread_main, NULL) == 0);
    for (int i = 0; i < 2; i++) assert(pthread_join(threads[i], NULL) == 0);
    puts("owned-host-lifecycle: PASS");
    return 0;
}
