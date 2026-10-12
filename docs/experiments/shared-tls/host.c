/* Focused synchronous qualification host: real GC/stack helpers, no networking. */
#include "tls-managed.h"
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
typedef wchar_t entry_char;
#else
typedef char entry_char;
#endif
typedef int32_t (*entry_fn)(int32_t, int32_t *, neoclr_aot_context *);
int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const entry_char **argv) {
    (void)argc; (void)argv;
    if (with_args || !entry) return 99;
    unsigned char *heap = calloc(1, 1024 * 1024);
    if (!heap) return 98;
    neoclr_aot_context context = {.text = {heap, 1024 * 1024, 0}};
    entry_fn run;
    memcpy(&run, &entry, sizeof(run));
    int32_t value = -1;
    int32_t status = run(0, &value, &context);
    if (neoclr_root_probe_head_v1() || neoclr_gc_collect_v1(&context, NULL)) abort();
    neoclr_tls_managed_dispose_v1(&context);
    if (neoclr_gc_entry_check_v1(&context)) abort();
    free(heap);
    return status ? 100 + status : value;
}
#ifdef NEOCLR_TLS_AOT
int main(void) {
    entry_fn entry = neoclr_entry_v4;
    const void *pointer;
    memcpy(&pointer, &entry, sizeof(pointer));
    return neoclr_jit_run_v1(pointer, 0, 0, NULL);
}
#endif
