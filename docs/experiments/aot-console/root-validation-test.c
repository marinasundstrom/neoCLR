/* Invoke each malformed-descriptor case separately: validation must abort in
 * both probe-only and GC builds, including ancestors belonging to another heap. */
#include "root-probe.h"
#include "text-arena.h"
#include <string.h>
#ifdef _WIN32
#include <stdlib.h>
#endif
int main(int argc, char **argv) {
#ifdef _WIN32
    /* Expected aborts must not open Windows crash-reporting dialogs in CI. */
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    if (argc != 2) return 2;
    neoclr_aot_context outer = {0}, inner = {0};
    uint64_t word = 0, lanes[1] = {0};
    neoclr_probe_storage slot = {&word, 8, 0};
    neoclr_probe_frame parent, child;
    neoclr_probe_enter_v3(&parent, &outer, 0, &slot, 1, "{}", 2);
    if (!strcmp(argv[1], "address")) slot.address = NULL;
    else if (!strcmp(argv[1], "width")) slot.read_bytes = 3;
    else if (!strcmp(argv[1], "flags")) slot.flags = 3;
    else if (!strcmp(argv[1], "fault")) {
        outer.fault.code = 3;
        outer.fault.frame_count = 65;
    } else if (!strcmp(argv[1], "transient")) {
        neoclr_probe_transient_v2(&parent, 1, &slot, 1, "{}", 2);
        slot.read_bytes = 3;
        parent.storage_count = 0;
    } else return 2;
    neoclr_probe_storage empty = {&word, 8, 0};
    neoclr_probe_enter_v3(&child, &inner, 1, &empty, 0, "{}", 2);
    const char *plan = "{\"requiredSpillLanes\":[]}";
    neoclr_probe_stack_roots_v2(&child, 0, lanes, 0, plan, (uint32_t)strlen(plan));
    return 0; /* Reaching here means a contract violation escaped validation. */
}
