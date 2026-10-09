/* Run repeated entries to exercise pool reset and every-operation native GC. */
#include "../aot-console/native-gc.h"
#include <stdio.h>

#ifndef EXPECTED_FAULT
#define EXPECTED_FAULT 0
#endif

int main(void) {
    uint64_t storage[8192];
    neoclr_aot_context context = { .text = { (unsigned char *)storage, sizeof(storage), 0 } };
    for (int iteration = 0; iteration < 3; ++iteration) {
        int32_t result = -1;
        int32_t status = neoclr_entry_v4(0, &result, &context);
        if (EXPECTED_FAULT) {
            if (status != EXPECTED_FAULT || context.fault.code != EXPECTED_FAULT ||
                result != -1 || !context.fault.frame_count || !context.fault.message ||
                neoclr_root_probe_depth_v1()) return 3;
            if (!iteration) neoclr_aot_render_fault(stdout, &context.fault);
            continue;
        }
        if (status || result != 42 || context.fault.code || context.fault.frame_count ||
            context.fault.message || neoclr_root_probe_depth_v1()) {
            fprintf(stderr, "iteration=%d status=%d result=%d\n", iteration, status, result);
            neoclr_aot_render_fault(stderr, &context.fault);
            return 1;
        }
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (!stats.collections) return 2;
    if (EXPECTED_FAULT) return 0;
    printf("42; three entries, %llu collections, %llu reclaimed allocations\n",
           (unsigned long long)stats.collections, (unsigned long long)stats.reclaimed_allocations);
    return 0;
}
