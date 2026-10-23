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
    if (!strcmp(argv[1], "state-valid")) {
        const char *plan = "{\"requiredSpillLanes\":[]}";
        neoclr_parked_roots saved = {0};
        if (neoclr_root_probe_head_v1() != &parent ||
            neoclr_root_probe_depth_v1() != 1 || neoclr_root_probe_count_v1()) return 3;
        neoclr_probe_enter_v3(&child, &outer, 1, &slot, 0, "{}", 2);
        neoclr_probe_stack_roots_v2(&child, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        neoclr_roots_park_v1(&saved);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1() ||
            neoclr_root_probe_count_v1() != 1 || neoclr_roots_parked_v1() != &saved) return 4;
        neoclr_probe_frame other;
        neoclr_probe_enter_v3(&other, &inner, 2, &slot, 0, "{}", 2);
        neoclr_probe_stack_roots_v2(&other, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        neoclr_probe_leave_v1(&other);
        neoclr_roots_resume_v1(&saved);
        if (neoclr_root_probe_head_v1() != &child ||
            neoclr_root_probe_depth_v1() != 2 || neoclr_root_probe_count_v1() != 2 ||
            neoclr_roots_parked_v1() || saved.head || saved.next) return 5;
        neoclr_probe_leave_v1(&child);
        neoclr_probe_leave_v1(&parent);
        if (neoclr_root_probe_head_v1() || neoclr_root_probe_depth_v1() ||
            neoclr_root_probe_count_v1() != 2) return 6;
        return 0;
    }
    if (!strncmp(argv[1], "context-", 8)) {
        const char *plan = "{\"requiredSpillLanes\":[]}";
        neoclr_probe_enter_v3(&child, &outer, 1, &slot, 0, "{}", 2);
        neoclr_probe_stack_roots_v2(&child, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        if (!strcmp(argv[1], "context-mutated")) {
            outer.fault.code = 3;
            outer.fault.frame_count = 65;
        } else if (!strcmp(argv[1], "context-null")) child.context = NULL;
        else if (!strcmp(argv[1], "context-ancestor-null")) parent.context = NULL;
        else if (!strcmp(argv[1], "context-valid")) {
            /* Inactive fault fields are ignored; the largest active fault has
             * 65 roots, and enumeration remains independent of validation. */
            outer.fault.frame_count = 65;
            neoclr_probe_stack_roots_v2(&child, 0, lanes, 0, plan, (uint32_t)strlen(plan));
            outer.fault.code = 3;
            outer.fault.frame_count = 64;
            neoclr_probe_storage roots[65];
            if (neoclr_probe_fault_roots_v1(&outer, roots, 65) != 65 ||
                neoclr_probe_fault_roots_v1(&outer, roots, 64) != -1) return 3;
            if (roots[0].address != &outer.fault.message) return 4;
            for (uint32_t i = 0; i < 64; i++)
                if (roots[i + 1].address != &outer.fault.frames[i].function ||
                    roots[i + 1].read_bytes != 8 || roots[i + 1].flags) return 5;
            neoclr_probe_stack_roots_v2(&child, 0, lanes, 0, plan, (uint32_t)strlen(plan));
            neoclr_probe_leave_v1(&child);
            neoclr_probe_leave_v1(&parent);
            return 0;
        } else return 2;
        neoclr_probe_stack_roots_v2(&child, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        return 0;
    }
    if (!strncmp(argv[1], "batch-", 6)) {
        uint64_t words[33] = {0};
        neoclr_probe_storage slots[33];
        for (uint32_t i = 0; i < 33; i++)
            slots[i] = (neoclr_probe_storage){&words[i], i % 2 ? 4 : 8, i % 3};
        parent.storage = slots;
        parent.storage_count = 33;
        const char *plan = "{\"requiredSpillLanes\":[]}";
        neoclr_probe_stack_roots_v2(&parent, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        /* Mutate a previously accepted ancestor, covering a vector group and
         * the final scalar remainder without dereferencing malformed slots. */
        if (!strcmp(argv[1], "batch-address")) slots[32].address = NULL;
        else if (!strcmp(argv[1], "batch-width")) slots[15].read_bytes = 12;
        else if (!strcmp(argv[1], "batch-flags")) slots[31].flags = 3;
        else if (!strcmp(argv[1], "batch-transient")) {
            neoclr_probe_transient_v2(&parent, 1, slots, 33, "{}", 2);
            parent.storage_count = 0;
            slots[32].read_bytes = 0;
        } else if (!strcmp(argv[1], "batch-valid")) {
            for (uint32_t count = 0; count <= 33; count++) {
                parent.storage_count = count;
                neoclr_probe_stack_roots_v2(&parent, 0, lanes, 0, plan, (uint32_t)strlen(plan));
            }
            neoclr_probe_leave_v1(&parent);
            return 0;
        } else return 2;
        neoclr_probe_enter_v3(&child, &inner, 1, &slot, 0, "{}", 2);
        neoclr_probe_stack_roots_v2(&child, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        return 0;
    }
    if (!strncmp(argv[1], "plan-", 5)) {
        char plan[128] = "{\"requiredSpillLanes\":[]}";
        /* Warm the search hint, then reuse the same address with changed bytes. */
        neoclr_probe_stack_roots_v2(&parent, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        if (!strcmp(argv[1], "plan-missing")) plan[2] = 'X';
        else if (!strcmp(argv[1], "plan-length")) {
            neoclr_probe_stack_roots_v2(&parent, 0, lanes, 0, plan, (uint32_t)strlen(plan) - 1);
            return 0;
        } else if (!strcmp(argv[1], "plan-short")) memcpy(plan, "{}", sizeof("{}"));
        else if (!strcmp(argv[1], "plan-moved")) {
            memcpy(plan, "{\"other\":0,\"requiredSpillLanes\":[]}", sizeof("{\"other\":0,\"requiredSpillLanes\":[]}"));
            neoclr_probe_stack_roots_v2(&parent, 0, lanes, 0, plan, (uint32_t)strlen(plan));
            memcpy(plan, "requiredSpillLanes", sizeof("requiredSpillLanes")); /* Marker at both bounds. */
            neoclr_probe_stack_roots_v2(&parent, 0, lanes, 0, plan, (uint32_t)strlen(plan));
            neoclr_probe_leave_v1(&parent);
            return 0;
        } else return 2;
        neoclr_probe_stack_roots_v2(&parent, 0, lanes, 0, plan, (uint32_t)strlen(plan));
        return 0;
    }
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
