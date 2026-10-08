#include "root-probe.h"
#include <stdlib.h>
#include <string.h>
static _Thread_local uint64_t calls;
void neoclr_probe_stack_roots_v1(uint32_t function, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length) {
    if (function >= 512 || instruction >= 8192 || lane_count > 8192 || !lanes ||
        !plan || strlen(plan) != length || !strstr(plan, "requiredSpillLanes")) abort();
    /* Exercise the snapshot reads without interpreting addresses or retaining them. */
    volatile uint64_t value = 0;
    for (uint32_t i = 0; i < lane_count; i++) value ^= lanes[i];
    (void)value;
    calls++;
}
uint64_t neoclr_root_probe_count_v1(void) { return calls; }
