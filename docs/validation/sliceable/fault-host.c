#include "text-arena.h"
#include "native-gc.h"
#include <string.h>
typedef int32_t (*entry_args)(int32_t, int32_t *, neoclr_aot_context *, const char **);
static int32_t check(entry_args entry) {
    const char *args[] = {"sliceable", "x", "x", "x", "x"};
    for (int count = 1; count <= 4; ++count) {
        uint64_t heap[8192] = {0};
        neoclr_aot_context context = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
        int32_t result = -99;
        int32_t status = entry(count + 1, &result, &context, args);
        if (status != NEOCLR_AOT_FAULT_USER_FAULT || context.fault.code != NEOCLR_AOT_FAULT_USER_FAULT || result != -99) return count;
        const char *message = "Range is outside the value";
        if (!context.fault.message || context.fault.message->length != strlen(message) ||
            memcmp(context.fault.message->bytes, message, strlen(message))) return count + 10;
        if (neoclr_root_probe_head_v1() || neoclr_gc_collect_v1(&context, NULL) || context.text.used) return 20;
    }
    return 42;
}
#ifdef JIT_HOST
int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const char **argv) {
    if (!entry || !with_args || argc < 1 || !argv) return 2;
    entry_args callable;
    _Static_assert(sizeof(callable) == sizeof(entry), "entry pointer size");
    memcpy(&callable, &entry, sizeof(callable));
    return check(callable);
}
#else
extern int32_t neoclr_entry_args_v1(int32_t, int32_t *, neoclr_aot_context *, const char **);
int main(void) { return check(neoclr_entry_args_v1); }
#endif
