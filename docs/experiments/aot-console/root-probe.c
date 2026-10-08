#include "root-probe.h"
#include "../aot-fault-details/fault-details.h"
#include <stdlib.h>
#include <string.h>
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
static _Thread_local uint64_t calls;
static _Thread_local uint32_t depth;
static _Thread_local neoclr_probe_frame *head;
int32_t neoclr_probe_fault_roots_v1(const void *context, neoclr_probe_storage *output,
                                   uint32_t capacity) {
    if (!context) return -1;
    const neoclr_aot_fault *fault = context;
    if (!fault->code) return 0;
    if (fault->frame_count > 64 || !output || capacity < 1 + fault->frame_count) return -1;
    output[0] = (neoclr_probe_storage){&fault->message, 8, 0};
    for (uint32_t i = 0; i < fault->frame_count; i++)
        output[i + 1] = (neoclr_probe_storage){&fault->frames[i].function, 8, 0};
    return (int32_t)(1 + fault->frame_count);
}
static void observe_fault(const void *context) {
    neoclr_probe_storage slots[65];
    int32_t count = neoclr_probe_fault_roots_v1(context, slots, 65);
    if (count < 0) abort();
    volatile uint64_t observed = 0;
    for (int32_t i = 0; i < count; i++) {
        uint64_t word;
        memcpy(&word, slots[i].address, sizeof(word));
        observed ^= word;
    }
    (void)observed;
}
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
    observe_fault(frame->context);
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
        observe_fault(f->context);
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
#ifdef NEOCLR_NATIVE_GC
    /* Full initialized snapshot is published. Allocation services never collect. */
    if (neoclr_gc_collect_v1((neoclr_aot_context *)frame->context, head)) abort();
#endif
}
void neoclr_probe_transient_v2(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (head != frame || (phase != 1 && phase != 2 && phase != 3) || !storage || !plan || strlen(plan) != length) abort();
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
