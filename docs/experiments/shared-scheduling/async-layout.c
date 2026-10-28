/* Read actual private carrier definitions without exporting a runtime ABI. */
#include "../../../runtime/native/async/native-runtime.c"
#include <stdio.h>
void neoclr_scheduler_wait_v1(neoclr_aot_context *c) { (void)c; abort(); }
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *c, uint64_t *out,
                                   const neoclr_probe_frame *f) {
    (void)c; (void)out; (void)f; abort();
}
int32_t neoclr_scheduler_complete_v1(neoclr_aot_context *c, uint64_t id,
                                    const neoclr_probe_frame *f) {
    (void)c; (void)id; (void)f; abort();
}
int main(void) {
    printf("{\"carrierRecordBytes\":%zu,\"machineContextBytes\":%zu,"
           "\"ownerStateBytes\":%zu,\"operationBytes\":%zu,"
           "\"carrierCapacity\":%u,\"stackUsableBytes\":%u,"
           "\"guardBytesPerMapping\":%ld,\"stackCacheLimit\":%u}\n",
           sizeof(fiber), sizeof(ucontext_t), sizeof(state), sizeof(operation),
           LIMIT, STACK_BYTES, 2 * sysconf(_SC_PAGESIZE), NEOCLR_ASYNC_STACK_CACHE_LIMIT);
    return 0;
}
