#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "../../experiments/aot-scalar/abi.h"

#ifdef JIT_HOST
#include "../../experiments/aot-console/text-arena.h"
#include "../../experiments/aot-console/native-gc.h"
typedef int32_t (*entry_fn)(int32_t, int32_t *, neoclr_aot_context *);
#else
typedef int32_t (*entry_fn)(int32_t, int32_t *);
#endif

static int32_t check(entry_fn entry) {
    int32_t result = -123;
#ifdef JIT_HOST
    uint64_t heap[8192] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    int32_t status = entry(0, &result, &context);
#else
    int32_t status = entry(0, &result);
#endif
#ifdef EXPECT_OVERFLOW
    int ok = status == NEOCLR_AOT_ARITHMETIC_OVERFLOW && result == -123;
#else
    int ok = status == NEOCLR_AOT_OK && result == 42;
#endif
#ifdef JIT_HOST
    if (context.fault.code != (uint32_t)status || neoclr_root_probe_head_v1() ||
        neoclr_gc_collect_v1(&context, NULL) || context.text.used) ok = 0;
#endif
    printf("checked arithmetic: status=%d result=%d\n", status, result);
    return ok ? 0 : 1;
}

#ifdef JIT_HOST
int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const char **argv) {
    if (!entry || with_args || argc < 1 || !argv) return 1;
    entry_fn callable;
    _Static_assert(sizeof(callable) == sizeof(entry), "entry pointer size");
    memcpy(&callable, &entry, sizeof(callable));
    return check(callable);
}
#else
int main(void) { return check(neoclr_entry_v2); }
#endif
