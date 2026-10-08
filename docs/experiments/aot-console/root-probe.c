#include "root-probe.h"
#include <stdlib.h>
#include <string.h>
static _Thread_local uint64_t calls;
static _Thread_local uint32_t depth;
static _Thread_local neoclr_probe_frame *head;
void neoclr_probe_enter_v3(neoclr_probe_frame *frame, const void *context, uint32_t function,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (!frame || !context || function >= 512) abort();
    memset(frame, 0, sizeof(*frame));
    frame->previous = head;
    frame->context = context;
    frame->function = function;
    if (!storage || !plan || strlen(plan) != length) abort();
    frame->storage = storage; frame->storage_count = count;
    frame->storage_plan = plan; frame->storage_length = length;
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
    frame->transient = NULL; frame->transient_plan = NULL;
    frame->transient_count = frame->transient_length = frame->transient_phase = 0;
    frame->instruction = instruction;
    frame->lanes = lanes;
    frame->lane_count = lane_count;
    frame->plan = plan;
    frame->length = length;
    volatile uint64_t value = 0;
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        for (uint32_t i = 0; i < f->storage_count; i++) {
            const neoclr_probe_storage *s = &f->storage[i];
            if (!s->address || (s->read_bytes != 4 && s->read_bytes != 8) || s->flags > 2) abort();
            uint64_t word = 0;
            memcpy(&word, s->address, s->read_bytes);
            value ^= word; /* Observe the slot only; never follow a borrowed pointee. */
        }
        for (uint32_t i = 0; i < f->transient_count; i++) {
            const neoclr_probe_storage *s = &f->transient[i];
            if (!s->address || (s->read_bytes != 4 && s->read_bytes != 8)) abort();
            uint64_t word = 0; memcpy(&word, s->address, s->read_bytes); value ^= word;
        }
        for (uint32_t i = 0; i < f->lane_count; i++) value ^= f->lanes[i];
    }
    (void)value;
    calls++;
}
void neoclr_probe_transient_v1(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (head != frame || (phase != 1 && phase != 2) || !storage || !plan || strlen(plan) != length) abort();
    frame->transient = storage; frame->transient_count = count;
    frame->transient_plan = plan; frame->transient_length = length; frame->transient_phase = phase;
    volatile uint64_t observed = 0;
    for (uint32_t i = 0; i < count; i++) {
        if (!storage[i].address || (storage[i].read_bytes != 4 && storage[i].read_bytes != 8)) abort();
        uint64_t word = 0; memcpy(&word, storage[i].address, storage[i].read_bytes);
        observed ^= word;
    }
    (void)observed;
}
const neoclr_probe_frame *neoclr_root_probe_head_v1(void) { return head; }
uint64_t neoclr_root_probe_count_v1(void) { return calls; }
uint32_t neoclr_root_probe_depth_v1(void) { return depth; }
