/* Diagnostic translation unit: inspect the actual private definitions, not copies.
 * Never link this alongside a separate native-workers.c. */
#include "../../../runtime/native/workers/native-workers.c"
#include "../aot-console/socket-listener.h"
#include <stdio.h>

int main(void) {
    printf("{\"contextHeaderBytes\":%zu,\"workerJobBytes\":%zu,"
           "\"workerPoolBytes\":%zu,\"ownerScopeBytes\":%zu,"
           "\"parkedRootsRecordBytes\":%zu,\"probeFrameBytes\":%zu,"
           "\"workerJobCapacity\":%u,\"workerThreadCapacity\":%u",
           sizeof(neoclr_aot_context), sizeof(neoclr_worker_job),
           sizeof(neoclr_workers), sizeof(neoclr_socket_scope),
           sizeof(neoclr_parked_roots), sizeof(neoclr_probe_frame), JOBS, THREADS);
#ifdef NEOCLR_SHARED_READY_QUEUE
    printf(",\"readyItemBytes\":%zu,\"readyQueueHeaderBytes\":%zu,"
           "\"ownerReadyStorageBytes\":%zu",
           sizeof(neoclr_ready_item), sizeof(neoclr_ready_queue),
           sizeof(((neoclr_socket_scope *)0)->runnable_items));
#endif
    puts("}");
    return 0;
}
