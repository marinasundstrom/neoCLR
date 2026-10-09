#include <assert.h>
#include <stdio.h>
#include "windows-gc-host.h"
#ifdef NDEBUG
#error "Acceptance checks must remain enabled"
#endif
int main(void) {
    neoclr_windows_gc_host host = {0};
    assert(neoclr_windows_gc_host_open(&host, 2048) == 0);
    int statuses[] = {0, 8, 5, 0};
    for (int pass = 0; pass < 4; ++pass) {
        neoclr_gc_statistics before = neoclr_gc_statistics_v1();
        int32_t output = -99;
        int mode = pass == 3 ? 0 : pass;
        int32_t status = neoclr_entry_v4(mode, &output, &host.context);
        assert(status == statuses[pass] && output == (status ? -99 : 42));
        assert(host.context.fault.code == (uint32_t)status);
        if (status) {
            assert(host.context.fault.message && host.context.fault.frame_count > 0);
        }
        neoclr_gc_statistics after = neoclr_gc_statistics_v1();
        assert(after.allocations - before.allocations == 102);
        assert(after.collections - before.collections >= 100);
        assert(after.reclaimed_allocations - before.reclaimed_allocations >= 99);
        assert(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
        assert(neoclr_gc_collect_v1(&host.context, NULL) == 0 && !host.context.text.used);
        after = neoclr_gc_statistics_v1();
        assert(after.reclaimed_allocations - before.reclaimed_allocations == 102);
    }
    assert(neoclr_windows_gc_host_close(&host) == 0);
    puts("{\"passed\":true,\"invocations\":4,\"allocations\":408,\"heapBytes\":2048,\"liveRootsPreserved\":true,\"faultCleanup\":true,\"reusePassed\":true}");
    return 0;
}
