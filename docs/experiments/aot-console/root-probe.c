#include "root-probe.h"
#include <stdlib.h>
#include <string.h>
static _Thread_local uint64_t calls;
static _Thread_local uint32_t depth;
static _Thread_local neoclr_probe_frame *head;
void neoclr_probe_enter_v1(neoclr_probe_frame *frame, const void *context, uint32_t function) {
    if (!frame || !context || function >= 512) abort();
    memset(frame, 0, sizeof(*frame));
    frame->previous = head;
    frame->context = context;
    frame->function = function;
    head = frame;
    depth++;
}
void neoclr_probe_leave_v1(neoclr_probe_frame *frame) {
    if (!frame || head != frame || !depth) abort();
    head = frame->previous;
    depth--;
    memset(frame, 0, sizeof(*frame));
}
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length) {
    if (!frame || head != frame || instruction >= 8192 || lane_count > 8192 || !lanes ||
        !plan || strlen(plan) != length || !strstr(plan, "requiredSpillLanes")) abort();
    frame->instruction = instruction;
    frame->lanes = lanes;
    frame->lane_count = lane_count;
    frame->plan = plan;
    frame->length = length;
    volatile uint64_t value = 0;
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        for (uint32_t i = 0; i < f->lane_count; i++) value ^= f->lanes[i];
    }
    (void)value;
    calls++;
}
const neoclr_probe_frame *neoclr_root_probe_head_v1(void) { return head; }
uint64_t neoclr_root_probe_count_v1(void) { return calls; }
uint32_t neoclr_root_probe_depth_v1(void) { return depth; }
